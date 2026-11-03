"""Isolated persistence/command smoke; never touches real app settings."""
import json
import subprocess
import tempfile
from pathlib import Path

root = Path(__file__).resolve().parents[1]
with tempfile.TemporaryDirectory(prefix="vxkey_smoke_") as data_dir:
    def run(requests):
        completed = subprocess.run(
            [str(root / "target/debug/vxkey_backend"), "--data-dir", data_dir],
            input="".join(json.dumps(r, ensure_ascii=False) + "\n" for r in requests),
            text=True, capture_output=True, check=True,
        )
        return [json.loads(line) for line in completed.stdout.splitlines()]
    first = run([{"command": "get_snapshot"}])[0]
    assert first["ok"]
    revision = first["data"]["revision"]
    assert not first["data"]["capabilities"]["native_engine"]
    result = run([
        {"command": "patch_config", "expected_revision": revision, "changes": {"ui": {"tray_icon_style": "playful"}}},
        {"command": "get_snapshot"},
        {"command": "convert_text", "text": "Tiếng Việt", "from_charset": "unicode", "to_charset": "unicode", "transforms": ["no_diacritics", "upper"], "keep_d": False, "flatten_caps": False},
        {"command": "set_launch_at_login", "enabled": True},
        {"command": "get_snapshot", "unknown": True},
    ])
    assert result[0]["ok"] and result[1]["data"]["config"]["ui"]["tray_icon_style"] == "playful"
    assert result[2]["data"]["text"] == "TIENG VIET"
    assert result[3]["error"]["code"] == "unsupported"
    assert result[4]["error"]["code"] == "invalid_argument"
    after = run([{"command": "get_snapshot"}])[0]
    assert after["data"]["revision"] == result[1]["data"]["revision"]
    assert after["data"]["config"]["ui"]["tray_icon_style"] == "playful"
    assert not after["data"]["config"]["system"]["launch_companion_at_login"]
    owner = subprocess.Popen([str(root / "target/debug/vxkey_backend"), "--data-dir", data_dir], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    try:
        owner.stdin.write('{"command":"get_snapshot"}\n'); owner.stdin.flush()
        assert json.loads(owner.stdout.readline())["ok"]
        rival = subprocess.run([str(root / "target/debug/vxkey_backend"), "--data-dir", data_dir], input="", text=True, capture_output=True, timeout=5)
        assert rival.returncode != 0 and "config_in_use" in rival.stderr
    finally:
        owner.communicate(timeout=5)
    evidence = {"status": "pass", "scope": "isolated JSON-lines developer service", "checks": ["fresh defaults", "persisted revision", "process restart readback", "real charset converter", "native action unsupported", "unknown fields rejected", "autostart remains off", "competing process writer rejected"], "native_activation": False}
    (root / "docs/qa/backend_smoke.json").write_text(json.dumps(evidence, ensure_ascii=False, indent=2) + "\n")
    print("Backend isolated smoke: PASS")
