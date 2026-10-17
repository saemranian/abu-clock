#!/usr/bin/env python3
"""
Atomic 4-part version bump and manifest synchronizer for Abu Clock (abu-clock).

Increments /VERSION (vA.B.C.D -> vA.B.C.(D+1)), synchronizes:
  - /Cargo.toml ([workspace.package] version = "A.B.D")
  - /crates/abu-clock-core/src/lib.rs (ABU_CLOCK_VERSION = "vA.B.C.D")
  - /apps/abu-clock-gui/package.json (if present)
  - /apps/abu-clock-gui/src-tauri/tauri.conf.json (if present: version, bundleVersion, versionCode)
  - /.agent-lock.json (current_version, agent lock status -> IDLE)
  - /CHANGELOG.md (appends structured audit entry)
"""

import argparse
import datetime
import json
import pathlib
import re
import sys


def parse_version(ver_str: str) -> tuple[int, int, int, int]:
    m = re.match(r"^v?(\d+)\.(\d+)\.(\d+)\.(\d+)$", ver_str.strip())
    if not m:
        raise ValueError(f"Invalid 4-part version string: {ver_str!r}")
    return int(m.group(1)), int(m.group(2)), int(m.group(3)), int(m.group(4))


def main() -> int:
    parser = argparse.ArgumentParser(description="Bump Abu Clock 4-part version by +1.")
    parser.add_argument("--agent", required=True, choices=["antigravity", "chatgpt"])
    parser.add_argument("--task", required=True, help="Task ID, e.g., P1-T1")
    parser.add_argument("--summary", required=True, help="Audit log summary of the completed task")
    parser.add_argument(
        "--set-version",
        default=None,
        help="Optional explicit 4-part version (e.g., v0.0.0.1 for bootstrap without incrementing)",
    )
    args = parser.parse_args()

    repo_root = pathlib.Path(__file__).resolve().parent.parent
    version_file = repo_root / "VERSION"

    if args.set_version:
        a, b, c, d = parse_version(args.set_version)
    else:
        current = version_file.read_text(encoding="utf-8").strip()
        a, b, c, d = parse_version(current)
        d += 1

    new_ver_4 = f"v{a}.{b}.{c}.{d}"
    new_ver_4_plain = f"{a}.{b}.{c}.{d}"
    semver_3 = f"{a}.{b}.{d}"
    now_utc = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

    # 1. Write /VERSION
    version_file.write_text(new_ver_4 + "\n", encoding="utf-8")

    # 2. Update root /Cargo.toml
    cargo_toml = repo_root / "Cargo.toml"
    if cargo_toml.exists():
        content = cargo_toml.read_text(encoding="utf-8")
        content = re.sub(
            r'(\[workspace\.package\][\s\S]*?^version\s*=\s*)"[^"]+"',
            rf'\1"{semver_3}"',
            content,
            count=1,
            flags=re.MULTILINE,
        )
        cargo_toml.write_text(content, encoding="utf-8")

    # 3. Update ABU_CLOCK_VERSION constant in crates/abu-clock-core/src/lib.rs
    core_lib = repo_root / "crates" / "abu-clock-core" / "src" / "lib.rs"
    if core_lib.exists():
        content = core_lib.read_text(encoding="utf-8")
        content = re.sub(
            r'pub const ABU_CLOCK_VERSION: &str = "v\d+\.\d+\.\d+\.\d+";',
            f'pub const ABU_CLOCK_VERSION: &str = "{new_ver_4}";',
            content,
        )
        core_lib.write_text(content, encoding="utf-8")

    # 4. Update apps/abu-clock-gui/package.json and tauri.conf.json if present
    gui_pkg = repo_root / "apps" / "abu-clock-gui" / "package.json"
    if gui_pkg.exists():
        pkg_data = json.loads(gui_pkg.read_text(encoding="utf-8"))
        pkg_data["version"] = semver_3
        gui_pkg.write_text(json.dumps(pkg_data, indent=2) + "\n", encoding="utf-8")

    tauri_conf = repo_root / "apps" / "abu-clock-gui" / "src-tauri" / "tauri.conf.json"
    if tauri_conf.exists():
        t_data = json.loads(tauri_conf.read_text(encoding="utf-8"))
        t_data["version"] = semver_3
        bundle = t_data.setdefault("bundle", {})
        ios_cfg = bundle.setdefault("iOS", {})
        ios_cfg["bundleVersion"] = new_ver_4_plain
        mac_cfg = bundle.setdefault("macOS", {})
        mac_cfg["bundleVersion"] = new_ver_4_plain
        android_cfg = bundle.setdefault("android", {})
        android_cfg["versionCode"] = d
        tauri_conf.write_text(json.dumps(t_data, indent=2) + "\n", encoding="utf-8")

    tauri_cargo = repo_root / "apps" / "abu-clock-gui" / "src-tauri" / "Cargo.toml"
    if tauri_cargo.exists():
        content = tauri_cargo.read_text(encoding="utf-8")
        content = re.sub(
            r'(^version\s*=\s*)"[^"]+"',
            rf'\1"{semver_3}"',
            content,
            count=1,
            flags=re.MULTILINE,
        )
        tauri_cargo.write_text(content, encoding="utf-8")

    # 5. Update /.agent-lock.json
    lock_file = repo_root / ".agent-lock.json"
    if lock_file.exists():
        lock_data = json.loads(lock_file.read_text(encoding="utf-8"))
    else:
        lock_data = {"current_version": new_ver_4, "active_locks": {}}
    lock_data["current_version"] = new_ver_4
    agent_entry = lock_data.setdefault("active_locks", {}).setdefault(args.agent, {})
    agent_entry["status"] = "IDLE"
    agent_entry["task_id"] = args.task
    agent_entry["locked_paths"] = []
    agent_entry["timestamp_utc"] = now_utc
    lock_file.write_text(json.dumps(lock_data, indent=2) + "\n", encoding="utf-8")

    # 6. Append to /CHANGELOG.md (if not already recorded for this exact version)
    changelog_file = repo_root / "CHANGELOG.md"
    row = f"| `{new_ver_4}` | `{now_utc}` | `{args.agent}` | `{args.task}` | {args.summary} |\n"
    if changelog_file.exists():
        cl_text = changelog_file.read_text(encoding="utf-8")
        if f"| `{new_ver_4}` |" not in cl_text:
            if not cl_text.endswith("\n"):
                cl_text += "\n"
            cl_text += row
            changelog_file.write_text(cl_text, encoding="utf-8")
    else:
        changelog_file.write_text(row, encoding="utf-8")

    print(f"Bumped version to {new_ver_4} (Cargo SemVer {semver_3}) for task {args.task} by {args.agent}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
