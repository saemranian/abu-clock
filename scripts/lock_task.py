#!/usr/bin/env python3
"""
Helper script to claim an active task lock in /.agent-lock.json prior to starting a task.
"""

import argparse
import datetime
import json
import pathlib
import sys


def main() -> int:
    parser = argparse.ArgumentParser(description="Acquire task lock in .agent-lock.json")
    parser.add_argument("--agent", required=True, choices=["antigravity", "chatgpt"])
    parser.add_argument("--task", required=True)
    parser.add_argument("--branch", required=True)
    parser.add_argument("--paths", nargs="+", required=True)
    args = parser.parse_args()

    repo_root = pathlib.Path(__file__).resolve().parent.parent
    lock_file = repo_root / ".agent-lock.json"
    data = json.loads(lock_file.read_text(encoding="utf-8"))
    now_utc = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

    data.setdefault("active_locks", {})[args.agent] = {
        "status": "WORKING",
        "branch": args.branch,
        "task_id": args.task,
        "locked_paths": args.paths,
        "timestamp_utc": now_utc,
    }
    lock_file.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    print(f"Acquired WORKING lock for {args.agent} on {args.task} ({args.branch}): {args.paths}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
