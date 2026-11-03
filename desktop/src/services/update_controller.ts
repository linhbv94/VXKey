// VX updater engine contract 1.0.0. Vendor this file; never import a sibling repo.
import type { Update } from '@tauri-apps/plugin-updater';

type UpdateHandle = Pick<Update, 'version' | 'close' | 'downloadAndInstall'>;
interface UpdaterDependencies {
  enabled: boolean;
  currentVersion?: string;
  check: (options: { timeout: number }) => Promise<UpdateHandle | null>;
  relaunch: () => Promise<void>;
  beforeInstall?: () => Promise<void>;
  installFailed?: () => Promise<void>;
}
export type UpdateStatus = 'idle' | 'checking' | 'current' | 'available' | 'downloading' | 'installing' | 'installed' | 'error' | 'disabled';
export interface UpdateState {
  status: UpdateStatus;
  version?: string;
  progress?: number;
  error?: string;
  errorCode?: 'check_failed' | 'invalid_version' | 'prepare_failed' | 'install_failed' | 'recovery_failed' | 'restart_failed';
  noticeVisible: boolean;
}

function stableVersion(value: string): number[] {
  const match = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:\+[0-9A-Za-z.-]+)?$/.exec(value);
  if (!match) throw new Error('invalid_stable_version');
  const numbers = match.slice(1, 4).map(Number);
  if (!numbers.every(Number.isSafeInteger)) throw new Error('invalid_stable_version');
  return numbers;
}

export function createUpdateController(dependencies: UpdaterDependencies) {
  const { enabled } = dependencies;
  let state: UpdateState = { status: enabled ? 'idle' : 'disabled', noticeVisible: false };
  let pendingUpdate: UpdateHandle | null = null;
  let started = false;
  let busy = false;
  let recoveryRequired = false;
  const dismissedVersions = new Set<string>();
  const listeners = new Set<() => void>();
  const snapshot = () => state;
  const subscribe = (listener: () => void) => {
    listeners.add(listener);
    return () => { listeners.delete(listener); };
  };
  function publish(next: UpdateState) {
    state = next;
    listeners.forEach((listener) => listener());
  }
  async function releaseHandle() {
    const old = pendingUpdate;
    pendingUpdate = null; // A failed close must never leave an installable stale handle.
    if (old) await old.close();
  }
  async function recover() {
    if (recoveryRequired) {
      await dependencies.installFailed?.();
      recoveryRequired = false;
    }
  }
  async function checkForUpdates(manual = true): Promise<void> {
    if (!enabled || busy || state.status === 'installed') return;
    busy = true;
    publish({ status: 'checking', noticeVisible: false });
    try {
      await recover();
      await releaseHandle();
      const next = await dependencies.check({ timeout: 15_000 });
      pendingUpdate = next;
      if (next) {
        const remote = stableVersion(next.version);
        if (dependencies.currentVersion) {
          const current = stableVersion(dependencies.currentVersion);
          const firstDifference = remote.findIndex((value, index) => value !== current[index]);
          if (firstDifference === -1 || remote[firstDifference] < current[firstDifference]) {
            await releaseHandle();
          }
        }
      }
      publish(pendingUpdate
        ? { status: 'available', version: pendingUpdate.version, noticeVisible: !dismissedVersions.has(pendingUpdate.version) }
        : { status: 'current', noticeVisible: false });
    } catch (error) {
      try { await releaseHandle(); } catch { /* state already cannot install this handle */ }
      publish({ status: 'error', error: String(error),
        errorCode: recoveryRequired ? 'recovery_failed' : String(error).includes('invalid_stable_version') ? 'invalid_version' : 'check_failed',
        noticeVisible: manual });
    } finally { busy = false; }
  }

  async function installUpdate(): Promise<void> {
    if (!enabled || busy || !pendingUpdate || state.status === 'installed') return;
    busy = true; // Set synchronously, before any guard awaits.
    const version = pendingUpdate.version;
    let downloaded = 0;
    let total = 0;
    let prepared = false;
    try {
      await recover();
      recoveryRequired = Boolean(dependencies.installFailed);
      await dependencies.beforeInstall?.();
      prepared = true;
      publish({ status: 'downloading', version, noticeVisible: true });
      await pendingUpdate.downloadAndInstall((event) => {
        if (event.event === 'Started') {
          total = event.data.contentLength ?? 0;
        } else if (event.event === 'Progress') {
          downloaded += event.data.chunkLength;
          publish({ status: 'downloading', version,
            progress: total > 0 ? Math.min(100, Math.round(downloaded / total * 100)) : undefined,
            noticeVisible: true });
        } else if (event.event === 'Finished') {
          publish({ status: 'installing', version, noticeVisible: true });
        }
      }, { timeout: 300_000 });
      recoveryRequired = false;
      // Only a resolved native install can enter installed; a restart failure stays here.
      publish({ status: 'installed', version, noticeVisible: true });
      try { await releaseHandle(); } catch { /* installed payload is not downloaded again */ }
      await restart();
    } catch (error) {
      let message = String(error);
      let errorCode: UpdateState['errorCode'] = prepared ? 'install_failed' : 'prepare_failed';
      try { await recover(); } catch (recoveryError) {
        message += `; recovery: ${String(recoveryError)}`;
        errorCode = 'recovery_failed';
      }
      publish({ status: 'error', version, error: message, errorCode, noticeVisible: true });
    } finally { busy = false; }
  }
  async function restart() {
    try { await dependencies.relaunch(); } catch (error) {
      publish({ ...state, status: 'installed', error: String(error), errorCode: 'restart_failed', noticeVisible: true });
    }
  }
  async function restartAfterUpdate(): Promise<void> {
    if (!enabled || busy || state.status !== 'installed') return;
    busy = true;
    try { await restart(); } finally { busy = false; }
  }
  function dismissUpdateNotice() {
    if (busy) return;
    if (state.version && state.status === 'available') dismissedVersions.add(state.version);
    publish({ ...state, noticeVisible: false });
  }
  function start() {
    if (!started) { started = true; void checkForUpdates(false); }
  }
  return { snapshot, subscribe, start, checkForUpdates, installUpdate, restartAfterUpdate, dismissUpdateNotice };
}
