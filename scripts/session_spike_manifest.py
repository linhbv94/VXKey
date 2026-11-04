#!/usr/bin/env python3
"""Read-only source/artifact evidence. No config, keystrokes or TCC database."""
import hashlib
import json
import platform
import plistlib
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def command(*args):
    result = subprocess.run(args, cwd=ROOT, text=True, capture_output=True, check=False)
    return {"exit_code": result.returncode, "stdout": result.stdout.strip(), "stderr": result.stderr.strip()}


def artifact(path):
    if not path.is_dir():
        return {"path": str(path), "exists": False}
    info = plistlib.loads((path / "Contents/Info.plist").read_bytes())
    executable = path / "Contents/MacOS" / info["CFBundleExecutable"]
    return {"path": str(path), "exists": True, "identifier": info["CFBundleIdentifier"],
            "version": info["CFBundleShortVersionString"],
            "executable_sha256": hashlib.sha256(executable.read_bytes()).hexdigest(),
            "info_sha256": hashlib.sha256((path / "Contents/Info.plist").read_bytes()).hexdigest(),
            "identity": command("codesign", "-dvv", "-r-", str(path)),
            "strict_verify": command("codesign", "--verify", "--deep", "--strict", str(path))}


tracked = subprocess.check_output(["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"], cwd=ROOT).decode().split("\0")
digest = hashlib.sha256()
count = 0
for name in sorted(set(tracked)):
    path = ROOT / name
    if not path.is_file() or path.is_symlink() or not (
        name.startswith(("native/", "crates/", "platforms/macos/", "scripts/"))
        and path.suffix in {".swift", ".rs", ".sh", ".py", ".toml", ".h"}
    ):
        continue
    digest.update(name.encode() + b"\0" + hashlib.sha256(path.read_bytes()).digest())
    count += 1
print(json.dumps({
    "recorded_at_utc": datetime.now(timezone.utc).isoformat(),
    "head": command("git", "rev-parse", "HEAD")["stdout"],
    "source_fingerprint": digest.hexdigest(), "source_files_hashed": count,
    "fingerprint_scope": "native/crates/platforms_macos/scripts source; includes dirty/untracked source",
    "dirty_entry_count": len([x for x in command("git", "status", "--porcelain")["stdout"].splitlines() if x]),
    "os": command("sw_vers"), "architecture": platform.machine(),
    "sdk": command("xcrun", "--show-sdk-version"),
    "artifacts": [artifact(path) for path in [
        Path("/Applications/VXKey.app"),
        ROOT / "build/compatibility_test_candidate/desktop_candidate/VXKey.app",
        ROOT / "build/session_spike_candidate/vxkey_session_spike.app",
        ROOT / "build/session_spike_candidate_v2/vxkey_session_spike.app",
        ROOT / "build/session_spike_candidate_v3/vxkey_session_spike.app",
        ROOT / "build/session_spike_candidate_v4/vxkey_session_spike.app",
        ROOT / "build/session_spike_candidate_v5/vxkey_session_spike.app",
        ROOT / "build/session_spike_candidate_v6/vxkey_session_spike.app",
    ]],
    "permission_retention_tested": False,
    "native_typing_accepted": False,
}, ensure_ascii=False, indent=2))
