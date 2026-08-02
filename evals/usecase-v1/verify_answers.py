#!/usr/bin/env python3
"""Verify coverage and timestamp integrity of the three use-case answers."""

from __future__ import annotations

import csv
import json
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
EVAL_DIR = ROOT / "evals" / "usecase-v1"
QUESTIONS = EVAL_DIR / "questions.tsv"
ANSWERS = EVAL_DIR / "answers"
CACHE = ROOT / ".oriel-cache" / "youtube"
TIMESTAMP = re.compile(
    r"\[(?P<label>\d+(?::\d{2}){1,2})\]"
    r"\(https://www\.youtube\.com/watch\?v=(?P<video>[\w-]{11})&t=(?P<seconds>\d+)s\)"
)


def normalise(text: str) -> str:
    return text.replace("\u2011", "-").replace("\u2013", "-").replace("\u2014", "-")


def label_seconds(label: str) -> int:
    parts = [int(part) for part in label.split(":")]
    if len(parts) == 2:
        minutes, seconds = parts
        return minutes * 60 + seconds
    hours, minutes, seconds = parts
    return hours * 3_600 + minutes * 60 + seconds


def source_starts(video_id: str) -> set[int]:
    versions = list((CACHE / video_id / "versions").glob("*.json"))
    if len(versions) != 1:
        raise AssertionError(f"expected one cached version for {video_id}, found {len(versions)}")
    cached = json.loads(versions[0].read_text())
    return {
        int(evidence["start_ms"]) // 1_000
        for evidence in cached["compiled"]["evidence"]
    }


def sections(answer: str, question_ids: list[str]) -> dict[str, str]:
    text = normalise(answer)
    positions: list[tuple[int, str]] = []
    for question_id in question_ids:
        match = re.search(rf"^#{{2,}}\s+{re.escape(question_id)}\b", text, re.MULTILINE)
        if match is None:
            raise AssertionError(f"answer is missing a heading for {question_id}")
        positions.append((match.start(), question_id))

    positions.sort()
    extracted: dict[str, str] = {}
    for index, (start, question_id) in enumerate(positions):
        end = positions[index + 1][0] if index + 1 < len(positions) else len(text)
        extracted[question_id] = text[start:end]
    return extracted


def main() -> int:
    with QUESTIONS.open(newline="") as handle:
        questions = list(csv.DictReader(handle, delimiter="\t"))

    grouped: dict[str, list[str]] = {}
    for question in questions:
        grouped.setdefault(question["video_id"], []).append(question["question_id"])

    total_citations = 0
    missing_citations: list[str] = []
    for video_id, question_ids in grouped.items():
        answer = (ANSWERS / f"{video_id}.md").read_text()
        answer_sections = sections(answer, question_ids)
        valid_starts = source_starts(video_id)
        video_citations = 0

        for question_id, section in answer_sections.items():
            citations = list(TIMESTAMP.finditer(section))
            if not citations:
                missing_citations.append(question_id)
                continue
            for citation in citations:
                cited_video = citation.group("video")
                seconds = int(citation.group("seconds"))
                label = citation.group("label")
                if cited_video != video_id:
                    raise AssertionError(
                        f"{question_id} cites {cited_video}, expected {video_id}"
                    )
                if seconds not in valid_starts:
                    raise AssertionError(
                        f"{question_id} cites t={seconds}s, which is not a passage start"
                    )
                if label_seconds(label) != seconds:
                    raise AssertionError(
                        f"{question_id} label {label} disagrees with t={seconds}s"
                    )
            video_citations += len(citations)

        total_citations += video_citations
        print(
            f"{video_id}: {len(question_ids)}/{len(question_ids)} questions present, "
            f"{video_citations}/{video_citations} timestamps valid"
        )

    print(f"total: {len(questions)}/{len(questions)} questions, {total_citations} valid timestamps")
    if missing_citations:
        print(f"questions without a source timestamp: {', '.join(missing_citations)}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
