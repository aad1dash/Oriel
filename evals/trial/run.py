#!/usr/bin/env python3
"""Drive Oriel's two MCP tools from an unrelated repository and record tool choice.

Each question runs in its own headless agent session with no memory of the others,
started in a repository that did not write Oriel and does not know it exists. The
only doors to the video are Oriel's two tools: web access is withheld so that
`search_source` and `read_source` compete for the same slot without a third option.

    python3 evals/trial/run.py --repo <path> [--only <id> ...]

Writes one stream-json transcript per question to evals/trial/runs/<id>.jsonl and
a summary of tool choices to stdout. Requires the sources to be cached already;
Oriel never reaches the network for a warm source.
"""

import argparse
import csv
import json
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
QUESTIONS = ROOT / "evals/trial/questions.tsv"
RUNS = ROOT / "evals/trial/runs"
DEFAULT_ORIEL = ROOT / "target/release/oriel"
DEFAULT_CACHE = ROOT / ".oriel-cache"

# The agent may read its own repository and call Oriel. Everything that could reach
# the video another way is withheld, so a tool choice is a choice between the two.
ALLOWED = [
    "mcp__oriel__search_source",
    "mcp__oriel__read_source",
    "Read",
    "Grep",
    "Glob",
]
DENIED = [
    "Edit",
    "Write",
    "NotebookEdit",
    "Bash",
    "WebFetch",
    "WebSearch",
    "Task",
    "Agent",
]


def ask(question: dict[str, str], repo: Path, mcp_config: Path) -> dict[str, object]:
    """Run one question in a fresh agent and return its tool calls and answer."""
    transcript = RUNS / f"{question['id']}.jsonl"
    command = [
        "claude",
        "-p",
        question["question"],
        "--mcp-config",
        str(mcp_config),
        "--strict-mcp-config",
        "--allowedTools",
        *ALLOWED,
        "--disallowedTools",
        *DENIED,
        "--output-format",
        "stream-json",
        "--verbose",
    ]
    completed = subprocess.run(
        command, cwd=repo, capture_output=True, text=True, timeout=900, check=False
    )
    transcript.write_text(completed.stdout)
    return summarise(question, completed.stdout, completed.stderr)


def summarise(question: dict[str, str], stdout: str, stderr: str) -> dict[str, object]:
    """Pull the ordered tool calls and the final answer out of a stream-json run."""
    calls: list[dict[str, object]] = []
    answer = ""
    model = ""
    cost = None
    for line in stdout.splitlines():
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        if event.get("type") == "assistant":
            for block in event.get("message", {}).get("content", []):
                if block.get("type") == "tool_use":
                    calls.append(
                        {"tool": block["name"], "input": block.get("input", {})}
                    )
            model = event.get("message", {}).get("model", model)
        elif event.get("type") == "result":
            answer = event.get("result", "") or ""
            cost = event.get("total_cost_usd")
    oriel_calls = [call for call in calls if call["tool"].startswith("mcp__oriel__")]
    chose = [call["tool"].removeprefix("mcp__oriel__") for call in oriel_calls]
    return {
        **question,
        "model": model,
        "cost_usd": cost,
        "tools_called": [call["tool"] for call in calls],
        "oriel_tools": chose,
        "first_oriel_tool": chose[0] if chose else "",
        "answer": answer,
        "stderr": stderr[-2000:] if not answer else "",
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--repo",
        type=Path,
        required=True,
        help="unrelated repository the read-only agent may inspect",
    )
    parser.add_argument(
        "--oriel-bin",
        type=Path,
        default=DEFAULT_ORIEL,
        help=f"Oriel executable (default: {DEFAULT_ORIEL.relative_to(ROOT)})",
    )
    parser.add_argument(
        "--cache-dir",
        type=Path,
        default=DEFAULT_CACHE,
        help="warm Oriel cache containing all trial sources",
    )
    parser.add_argument("--only", nargs="+", default=None)
    parser.add_argument("--workers", type=int, default=3)
    arguments = parser.parse_args()

    repo = arguments.repo.expanduser().resolve()
    oriel_bin = arguments.oriel_bin.expanduser().resolve()
    cache_dir = arguments.cache_dir.expanduser().resolve()
    if not repo.is_dir():
        parser.error(f"--repo is not a directory: {repo}")
    if not oriel_bin.is_file():
        parser.error(f"--oriel-bin is not a file: {oriel_bin}")
    if not cache_dir.is_dir():
        parser.error(f"--cache-dir is not a directory: {cache_dir}")
    if arguments.workers < 1:
        parser.error("--workers must be at least 1")

    RUNS.mkdir(parents=True, exist_ok=True)
    with QUESTIONS.open(newline="") as handle:
        questions = list(csv.DictReader(handle, delimiter="\t"))
    if arguments.only:
        known = {question["id"] for question in questions}
        unknown = sorted(set(arguments.only) - known)
        if unknown:
            parser.error(f"unknown question ids: {', '.join(unknown)}")
        questions = [q for q in questions if q["id"] in arguments.only]

    config = {
        "mcpServers": {
            "oriel": {
                "command": str(oriel_bin),
                "args": ["mcp", "--cache-dir", str(cache_dir)],
            }
        }
    }
    with tempfile.TemporaryDirectory(prefix="oriel-trial-") as temporary:
        mcp_config = Path(temporary) / "mcp.json"
        mcp_config.write_text(json.dumps(config), encoding="utf-8")
        with ThreadPoolExecutor(max_workers=arguments.workers) as pool:
            results = list(pool.map(lambda q: ask(q, repo, mcp_config), questions))

    (ROOT / "evals/trial/results.json").write_text(
        json.dumps(results, indent=2), encoding="utf-8"
    )
    for result in results:
        matched = (
            "ok " if result["first_oriel_tool"] == result["expected_tool"] else "OFF"
        )
        print(
            f"{matched} {result['id']:<14} expected={result['expected_tool']:<13} "
            f"chose={result['first_oriel_tool'] or '(none)':<13} calls={result['oriel_tools']}"
        )
    agreed = sum(1 for r in results if r["first_oriel_tool"] == r["expected_tool"])
    print(f"\nfirst tool matched expectation: {agreed}/{len(results)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
