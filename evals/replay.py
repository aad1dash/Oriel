#!/usr/bin/env python3
"""Replay the graded question log and report recall.

Reads evals/session-log.tsv, reruns every question against the cached source and
compares the result with the recorded verdict.

    blank correct_start_ms  -> the subject is absent; returning anything is a false positive
    a correct_start_ms      -> the passage covering that moment must be returned

A human marking `--miss 12:34` gives a time, not a passage boundary, so the recorded
time is snapped to the passage it falls in before comparing.

Requires the sources to be cached already (./ask --video ...). Never hits the network.
"""

import csv
import json
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BIN = ROOT / "target/release/oriel"
LOG = ROOT / "evals/session-log.tsv"
CACHE = ROOT / ".oriel-cache"


def passage_starts(video_id: str) -> list[int]:
    """Every passage start for a cached source, ascending."""
    for path in CACHE.glob("youtube/*/versions/*.json"):
        compiled = json.loads(path.read_text())["compiled"]
        if compiled["source"]["source_id"] == video_id:
            return sorted(e["start_ms"] for e in compiled["evidence"])
    raise SystemExit(f"{video_id} is not cached; run ./ask --video first")


def snap(starts: list[int], moment_ms: int) -> int:
    """The start of the passage covering a marked moment.

    Clock displays truncate to the second, so a mark can sit just before its own
    passage. Allow that second back before choosing.
    """
    covering = [start for start in starts if start <= moment_ms + 1000]
    return covering[-1] if covering else starts[0]


def ask(video_id: str, question: str) -> list[tuple[int, int]]:
    result = subprocess.run(
        [
            str(BIN), "search",
            "--source", f"https://www.youtube.com/watch?v={video_id}",
            "--language", "en",
            "--cache-dir", str(CACHE),
            "--query", question,
        ],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise SystemExit(f"engine failed on {question!r}: {result.stderr.strip()}")
    return [(m["start_ms"], m["end_ms"]) for m in json.loads(result.stdout)["moments"]]


def main() -> int:
    rows = list(csv.DictReader(LOG.open(), delimiter="\t"))
    verbose = "--verbose" in sys.argv

    totals: dict[str, list[int]] = defaultdict(lambda: [0, 0])
    absent_returned = 0
    absent_total = 0
    failures = []

    starts_by_video = {row["video_id"]: passage_starts(row["video_id"]) for row in rows}

    for row in rows:
        returned = ask(row["video_id"], row["question"])
        expected = row["correct_start_ms"].strip()

        if expected:
            want = snap(starts_by_video[row["video_id"]], int(expected))
            passed = any(start == want for start, _ in returned)
        else:
            absent_total += 1
            passed = not returned
            if returned:
                absent_returned += 1

        bucket = totals[row["video_id"]]
        bucket[1] += 1
        bucket[0] += passed
        if not passed:
            wanted = snap(starts_by_video[row["video_id"]], int(expected)) if expected else None
            failures.append((row["question"], wanted, returned))

    hit = sum(b[0] for b in totals.values())
    asked = sum(b[1] for b in totals.values())

    for video_id, (good, seen) in totals.items():
        print(f"  {video_id}  {good}/{seen}")
    print(f"\n  recall            {hit}/{asked}  ({hit / asked * 100:.0f}%)")
    print(f"  false positives   {absent_returned}/{absent_total} absent subjects returned evidence")

    if verbose and failures:
        print("\n  failures:")
        for question, wanted, returned in failures:
            want = f"{wanted // 1000}s" if wanted is not None else "nothing"
            got = ", ".join(f"{start // 1000}s" for start, _ in returned) or "nothing"
            print(f"    want {want:<8} got {got}")
            print(f"      {question[:96]}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
