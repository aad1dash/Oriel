use std::{
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

use oriel::{fixture::compile_fixture, store::FileSourceStore};
use serde_json::{Value, json};

const FIXTURE: &str = "schema_version\t1\n\
source_url\thttps://youtu.be/dQw4w9WgXcQ\n\
title\tMCP evidence fixture\n\
creator\tOriel\n\
duration_ms\t20000\n\
language\ten\n\
caption_provenance\tmanual\n\
cue\t0\t10000\tThe source introduces the evidence contract.\n\
cue\t10000\t20000\tTimestamped evidence remains attached to its source.\n";
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(5);

/// A running server with a cached fixture, already past initialisation.
struct Session {
    child: std::process::Child,
    stdin: std::process::ChildStdin,
    receiver: Receiver<std::io::Result<String>>,
    reader: thread::JoinHandle<()>,
    cache: tempfile::TempDir,
}

impl Session {
    fn start() -> Self {
        let cache = tempfile::tempdir().expect("cache should be created");
        let compiled = compile_fixture(FIXTURE).expect("fixture should compile");
        FileSourceStore::new(cache.path())
            .save(&compiled, true)
            .expect("fixture should be cached");

        let mut child = Command::new(env!("CARGO_BIN_EXE_oriel"))
            .args(["mcp", "--cache-dir"])
            .arg(cache.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("MCP server should start");
        let mut stdin = child.stdin.take().expect("server stdin should be piped");
        let stdout = child.stdout.take().expect("server stdout should be piped");
        let (sender, receiver) = mpsc::channel();
        let reader = thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                if sender.send(line).is_err() {
                    break;
                }
            }
        });

        send(
            &mut stdin,
            &json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2025-11-25",
                    "capabilities": {},
                    "clientInfo": {"name": "oriel-test-agent", "version": "1.0.0"}
                }
            }),
        );
        let initialised = receive_id(&receiver, 1);
        assert_eq!(initialised["result"]["serverInfo"]["name"], "oriel");
        send(
            &mut stdin,
            &json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        );

        Self {
            child,
            stdin,
            receiver,
            reader,
            cache,
        }
    }

    fn request(&mut self, id: u64, method: &str, params: &Value) -> Value {
        send(
            &mut self.stdin,
            &json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}),
        );
        receive_id(&self.receiver, id)
    }

    fn call(&mut self, id: u64, tool: &str, arguments: &Value) -> Value {
        let response = self.request(
            id,
            "tools/call",
            &json!({"name":tool,"arguments":arguments}),
        );
        assert_eq!(response["result"]["isError"], false);
        response["result"]["structuredContent"].clone()
    }

    fn shutdown(self) {
        let Self {
            mut child,
            stdin,
            reader,
            cache,
            receiver,
        } = self;
        // Closing stdin is what tells the server to stop, so it must go first.
        drop(stdin);
        let status = child.wait().expect("MCP server should stop");
        assert!(status.success());
        reader.join().expect("stdout reader should stop");
        // The cache and the channel outlive the server they served.
        drop((cache, receiver));
    }
}

#[test]
fn stdio_agent_discovers_both_source_tools() {
    let mut session = Session::start();
    let tools = session.request(2, "tools/list", &json!({}));
    let mut names = tools["result"]["tools"]
        .as_array()
        .expect("tools should be a list")
        .iter()
        .map(|tool| tool["name"].as_str().unwrap_or_default())
        .collect::<Vec<_>>();
    names.sort_unstable();

    assert_eq!(names, ["read_source", "search_source"]);
    session.shutdown();
}

#[test]
fn stdio_agent_can_search_cached_source_evidence() {
    let mut session = Session::start();
    let packet = session.call(
        2,
        "search_source",
        &json!({
            "source":"https://youtu.be/dQw4w9WgXcQ",
            "language":"en",
            "query":"timestamped evidence"
        }),
    );

    assert_eq!(packet["cache"]["status"], "hit");
    assert_eq!(packet["moments"][0]["start_ms"], 10_000);
    session.shutdown();
}

#[test]
fn stdio_agent_can_read_a_cached_source_whole() {
    let mut session = Session::start();
    let packet = session.call(
        2,
        "read_source",
        &json!({"source":"https://youtu.be/dQw4w9WgXcQ","language":"en"}),
    );

    assert_eq!(packet["cache"]["status"], "hit");
    assert_eq!(packet["passage_count"], 2);
    assert_eq!(
        packet["passages"][1]["text"],
        "Timestamped evidence remains attached to its source."
    );
    assert_eq!(
        packet["passages"][1]["timestamp_url"],
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=10s"
    );
    session.shutdown();
}

fn send(stdin: &mut impl Write, message: &Value) {
    writeln!(stdin, "{message}").expect("request should be written");
    stdin.flush().expect("request should be flushed");
}

fn receive_id(receiver: &Receiver<std::io::Result<String>>, expected_id: u64) -> Value {
    loop {
        let line = receiver
            .recv_timeout(RESPONSE_TIMEOUT)
            .expect("MCP response should arrive")
            .expect("MCP response should be readable");
        let message: Value = serde_json::from_str(&line).expect("MCP response should be JSON");
        if message["id"] == expected_id {
            return message;
        }
    }
}
