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

#[test]
fn stdio_agent_can_discover_and_search_source_evidence() {
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
    send(
        &mut stdin,
        &json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
    );
    let tools = receive_id(&receiver, 2);
    assert_eq!(tools["result"]["tools"].as_array().map(Vec::len), Some(1));
    assert_eq!(tools["result"]["tools"][0]["name"], "search_source");

    send(
        &mut stdin,
        &json!({
            "jsonrpc":"2.0",
            "id":3,
            "method":"tools/call",
            "params":{
                "name":"search_source",
                "arguments":{
                    "source":"https://youtu.be/dQw4w9WgXcQ",
                    "language":"en",
                    "query":"timestamped evidence"
                }
            }
        }),
    );
    let result = receive_id(&receiver, 3);
    assert_eq!(result["result"]["isError"], false);
    assert_eq!(
        result["result"]["structuredContent"]["cache"]["status"],
        "hit"
    );
    assert_eq!(
        result["result"]["structuredContent"]["moments"][0]["start_ms"],
        10_000
    );

    drop(stdin);
    let status = child.wait().expect("MCP server should stop");
    assert!(status.success());
    reader.join().expect("stdout reader should stop");
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
