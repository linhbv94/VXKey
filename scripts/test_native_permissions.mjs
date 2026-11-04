// Pure UI decision matrix; Node with native TypeScript stripping (local Node26).
import assert from 'node:assert/strict';
import { nativePermissions } from '../desktop/src/services/native_permissions.ts';
let checks = 0;
for (const stream of [false, true, undefined])
  for (const connected of [false, true])
    for (const ax of [false, true])
      for (const listen of [false, true])
        for (const post of [false, true]) {
          const snapshot = {
            capabilities: { native_session_stream: stream },
            runtime: { connection_status: connected ? 'connected' : 'disconnected',
              driver: { accessibility: ax, listen_access: listen, post_access: post } },
          };
          const result = nativePermissions(snapshot);
          const granted = ax && (stream === true || (listen && post));
          assert.equal(result.granted, granted);
          assert.equal(result.ready, connected && granted);
          assert.equal(result.missing, !granted);
          checks += 3;
        }
assert.equal(nativePermissions({ capabilities: {}, runtime: { connection_status: 'disconnected' } }).ready, false);
checks++;
console.log(JSON.stringify({ status: 'pass', checks,
  scope: 'shared UI permission decision matrix; no native permission claim' }));
