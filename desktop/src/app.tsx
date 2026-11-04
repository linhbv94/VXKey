import { nativePermissions } from './services/native_permissions';
import { useState, useLayoutEffect, useEffect, useCallback, useRef } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { initialSurface } from './services/window_surface';
import type { Snapshot, SurfaceType } from './types';
import { invoke } from '@tauri-apps/api/core';
import { sendCommand } from './services/api';
import { PermissionSetup } from './components/permission_setup';
import { SettingsView } from './components/settings_view';
import { TrayPanel } from './components/tray_panel';
import { HistoryPanel } from './components/history_panel';
import { ConversionView } from './components/conversion_view';
import { OnboardingView } from './components/onboarding_view';

function applyTheme(theme: string) {
  const root = document.documentElement;
  root.classList.remove('dark', 'light', 'black', 'oled');

  if (theme === 'light') {
    root.classList.add('light');
  } else if (theme === 'pure_black') {
    root.classList.add('dark', 'black', 'oled');
  } else if (theme === 'dark') {
    root.classList.add('dark');
  } else {
    // system preference
    const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    if (isDark) {
      root.classList.add('dark');
    } else {
      root.classList.add('light');
    }
  }
}

export function App() {
  const [surface, setSurface] = useState<SurfaceType>(initialSurface);
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [loading, setLoading] = useState(true);
  const [permissionSetup, setPermissionSetup] = useState(false);
  const [permissionDeferred, setPermissionDeferred] = useState(false);
  const [requestedSettingsTab, setRequestedSettingsTab] = useState<string | null>(null);
  const [navigationError, setNavigationError] = useState<string | null>(null);
  const snapshotGeneration = useRef(0);
  const hadPermissionsRef = useRef(false);
  const native = '__TAURI_INTERNALS__' in window;

  useLayoutEffect(() => {
    document.documentElement.dataset.surface = surface;
  }, [surface]);

  const loadSnapshot = useCallback(async () => {
    const generation = ++snapshotGeneration.current;
    const res = await sendCommand<Snapshot>('get_snapshot');
    if (generation !== snapshotGeneration.current) return;
    setLoading(false);
    if (res.ok && res.data) {
      setSnapshot(res.data);
    }
  }, []);

  useEffect(() => {
    void loadSnapshot();
    const refresh = () => { if (!document.hidden) void loadSnapshot(); };
    const timer = setInterval(refresh, 3000);
    window.addEventListener('focus', refresh);
    document.addEventListener('visibilitychange', refresh);
    // A reopened native panel need not receive a DOM focus event. Refresh its
    // theme, locale and runtime state immediately instead of waiting for polling.
    let disposed = false;
    let unlisten: (() => void) | undefined;
    if ('__TAURI_INTERNALS__' in window) {
      void getCurrentWindow().onFocusChanged(({ payload }) => {
        if (payload && !disposed) void loadSnapshot();
      }).then(stop => { if (disposed) stop(); else unlisten = stop; }).catch(() => {});
    }
    return () => {
      disposed = true; unlisten?.(); clearInterval(timer);
      window.removeEventListener('focus', refresh);
      document.removeEventListener('visibilitychange', refresh);
      snapshotGeneration.current += 1;
    };
  }, [loadSnapshot]);

  useEffect(() => {
    if (!native || surface !== 'settings' || !snapshot || snapshot.platform !== 'macos') return;
    const { missing: permissionsMissing, ready: permissionsReady } = nativePermissions(snapshot);

    if (permissionsReady) {
      hadPermissionsRef.current = true;
    } else if (permissionsMissing && hadPermissionsRef.current) {
      hadPermissionsRef.current = false;
      setPermissionDeferred(false);
      setPermissionSetup(true);
      return;
    }

    if (permissionDeferred) return;

    let pendingResume = false;
    let setupCompleted = false;
    try {
      pendingResume = localStorage.getItem('vxkey_setup_pending_resume') === '1';
      setupCompleted = localStorage.getItem('vxkey_setup_completed_v1') === '1';
    } catch {}

    if (
      permissionsMissing ||
      (permissionsReady &&
        snapshot.runtime.effective_mode === 'suspended' &&
        (pendingResume || !setupCompleted))
    ) {
      setPermissionSetup(true);
    }
  }, [native, surface, permissionDeferred, snapshot]);

  // Sync theme class and listen to system changes
  const currentTheme = snapshot?.config.ui.theme;
  useEffect(() => {
    if (!currentTheme) return;
    applyTheme(currentTheme);

    if (currentTheme === 'system') {
      const mql = window.matchMedia('(prefers-color-scheme: dark)');
      const handler = () => applyTheme('system');
      mql.addEventListener('change', handler);
      return () => mql.removeEventListener('change', handler);
    }
  }, [currentTheme]);

  const handleNavigate = async (target: SurfaceType, tab?: string) => {
    if (native) {
      try {
        await invoke('vxkey_open_surface', {surface: target, tab: tab ?? null});
        setNavigationError(null);
      } catch {
        setNavigationError(snapshot?.config.ui.locale === 'en' ? 'Could not open the window. Please retry.' : 'Không mở được cửa sổ. Vui lòng thử lại.');
      }
      return; // Never render another authority surface inside the current native window.
    }
    setSurface(target);
  };

  const handleClose = async (_current: SurfaceType) => {
    if (native) {
      try { await invoke('vxkey_close_surface'); }
      catch { setNavigationError(snapshot?.config.ui.locale === 'en' ? 'Could not close the window.' : 'Không đóng được cửa sổ.'); }
      return;
    }
    setSurface('settings');
  };

  useEffect(() => {
    if (surface === 'settings') return;
    const escape = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && !event.isComposing && !event.defaultPrevented) {event.preventDefault(); void handleClose(surface);}
    };
    window.addEventListener('keydown', escape);
    return () => window.removeEventListener('keydown', escape);
  });

  if (loading && !snapshot) {
    return (
      <div className="w-full h-full flex items-center justify-center text-xs text-white/50">
        <div className="animate-spin w-5 h-5 border-2 border-cyan-500 border-t-transparent rounded-full mr-2" />
        Đang khởi động giao diện VXKey…
      </div>
    );
  }

  if (!snapshot) {
    return (
      <div className="w-full h-full flex items-center justify-center text-xs text-rose-400 p-4 text-center">
        Không thể kết nối đến backend VXKey. Vui lòng thử lại.
      </div>
    );
  }

  return (
    <div className={`vx_app_surface vx_surface_${surface} w-full h-full relative overflow-hidden`}>
      {navigationError && <div role="alert" className="absolute top-1 inset-x-1 z-50 bg-rose-900 text-white text-xs p-2">{navigationError}</div>}
      {/* Dev Surface Switcher (only in browser query mode) */}
      {typeof window !== 'undefined' && !('__TAURI_INTERNALS__' in window) && (
        <div className="fixed bottom-2 left-2 z-50 flex items-center gap-1 bg-black/80 backdrop-blur border border-white/20 p-1 rounded-lg text-[10px]">
          <span className="px-1 text-amber-300">Preview · RAM</span>
          {(['settings', 'tray', 'history', 'conversion', 'onboarding'] as const).map(s => (
            <button
              key={s}
              type="button"
              onClick={() => setSurface(s)}
              className={`px-2 py-0.5 rounded capitalize ${
                surface === s ? 'bg-cyan-600 text-white font-semibold' : 'text-white/60 hover:text-white'
              }`}
            >
              {s}
            </button>
          ))}
        </div>
      )}

      {/* Permission setup occupies the native Settings window; no nested dialog. */}
      {surface === 'settings' && permissionSetup && (
        <PermissionSetup
          snapshot={snapshot}
          onRefresh={loadSnapshot}
          onLater={() => {
            setPermissionDeferred(true);
            setPermissionSetup(false);
            setRequestedSettingsTab('typing');
          }}
          onOpenFullSettings={() => {
            setPermissionDeferred(true);
            setPermissionSetup(false);
            setRequestedSettingsTab('typing');
          }}
          onFinishToMenubar={() => {
            setPermissionDeferred(true);
            setPermissionSetup(false);
            void handleClose('settings');
          }}
        />
      )}
      {/* Surface Router */}
      {surface === 'settings' && (
        <div hidden={permissionSetup} className="h-full">
          <SettingsView
            snapshot={snapshot}
            onRefresh={loadSnapshot}
            onClose={() => void handleClose('settings')}
            onPermissionSetup={() => {
              setPermissionDeferred(false);
              setPermissionSetup(true);
            }}
            onRevealSettings={() => { setPermissionDeferred(true); setPermissionSetup(false); }}
            requestedTab={requestedSettingsTab}
            onTabConsumed={() => setRequestedSettingsTab(null)}
          />
        </div>
      )}

      {surface === 'tray' && (
        <TrayPanel
          snapshot={snapshot}
          onNavigate={handleNavigate}
          onRefresh={loadSnapshot}
        />
      )}

      {surface === 'history' && (
        <HistoryPanel
          snapshot={snapshot}
          onClose={() => handleClose('history')}
        />
      )}

      {surface === 'conversion' && (
        <ConversionView
          snapshot={snapshot}
          onClose={() => handleClose('conversion')}
        />
      )}

      {surface === 'onboarding' && (
        <OnboardingView
          snapshot={snapshot}
          onFinish={() => handleClose('onboarding')}
          onClose={() => handleClose('onboarding')}
        />
      )}
    </div>
  );
}
