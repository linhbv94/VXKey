# VXKey 1.0.0 release checkpoint

User giao release ngày2026-11-03 và cho phép tạo/lưu khóa macOS+updater riêng cho linhbv94/VXKey. Repo public, chưa có release trước lượt này. Mac Silicon only; không đổi candidate/TCC.

Source production identity/updater/permission entry/pipeline đã chuẩn bị. Frontend/helper compile/backend check PASS; runtime regression, controller và86Swift checks chạy lại. Initial backend check thiếu helper resource; đã build helper thật và check lại. Giữ nguyên known typing gaps từ handoff.

CI/package/public sẽ cập nhật theo exact tagSHA; chưa claim E2E updater/permission retention/native typing fullPASS.


Published: https://github.com/linhbv94/VXKey/releases/tag/v1.0.0
App tag SHA: cb4157e28ea5c3a6b1595ce77ad970a2781c2e1c. Build run41565929429 and publish run41566944276 success. Initial run41565488567 failed before signing because macos-15 lacks SDK27; tooling-only PR6 selected verified hosted xcode-27 and retained immutable app tag. PR7 added gated publish. Independent downloaded draft signature/tamper and app/helper metadata/noQAenv/key-exclusion checks PASS; all3public files HTTP200 and hashes equal verified draft. See release_1_0_0_package_readback.json, release_1_0_0_public_checks.json and release_1_0_0_state.json.

No user app/candidate/TCC/trust changed; production private credentials stay in ignored .release_keys and authorized GitHub Secrets. Public payload scan found no private keys/tokens/home paths; maintenance/issue/internal QA logs were excluded. Remaining workspace dirty files are preserved private QA/handoff/docs. Current branch release/v1_0_0.

Known LOG_005 remains open, native typing fullPASS and N-to-N+1 permission retention are not established. Maintenance track remains open.
