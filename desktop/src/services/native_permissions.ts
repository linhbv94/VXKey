import type { Snapshot } from '../types';

// The authenticated helper negotiates its driver. Unknown/legacy helpers keep
// their existing requirements; preflight diagnostics cannot overrule stream AX.
export function nativePermissions(snapshot: Snapshot) {
  const stream = snapshot.capabilities.native_session_stream === true;
  const driver = snapshot.runtime.driver;
  const ax = Boolean(driver?.accessibility);
  const listen = Boolean(driver?.listen_access);
  const post = Boolean(driver?.post_access);
  const granted = ax && (stream || (listen && post));
  return { stream, ax, listen, post, granted,
    ready: snapshot.runtime.connection_status === 'connected' && granted,
    missing: snapshot.runtime.driver_fault === 'permission_required' ||
      (driver !== undefined && !granted) };
}
