import { nativePermissions } from '../services/native_permissions';
import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react';
import { sendCommand } from '../services/api';
import type { Snapshot } from '../types';
import { LiveTypingTest } from './live_typing_test';
import {
  ShieldCheck,
  CheckCircle2,
  AlertTriangle,
  ExternalLink,
  RefreshCw,
  RotateCcw,
  Play,
  Sliders,
  Check,
  Sparkles,
} from 'lucide-react';

interface Props {
  snapshot: Snapshot;
  onRefresh: () => void;
  onLater: () => void;
  onFinishToMenubar?: () => void;
  onOpenFullSettings?: () => void;
}

export function PermissionSetup({
  snapshot,
  onRefresh,
  onLater,
  onFinishToMenubar,
  onOpenFullSettings,
}: Props) {
  const vi = snapshot.config.ui.locale !== 'en';
  const runtime = snapshot.runtime;
  const driver = runtime.driver;
  const connected = runtime.connection_status === 'connected';
  const { stream, ax, listen, post, ready: ready } = nativePermissions(snapshot);
  const active =
    ready &&
    Boolean(driver?.attached) &&
    runtime.driver_status === 'active' &&
    runtime.effective_mode !== 'suspended';

  const [busy, setBusy] = useState(false);
  const [requestedOnce, setRequestedOnce] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [statusHint, setStatusHint] = useState<string | null>(null);

  useEffect(() => {
    if (active) {
      setError(null);
      setStatusHint(null);
      try {
        localStorage.removeItem('vxkey_setup_pending_resume');
      } catch {}
    }
  }, [active]);

  const handleRequestPermissions = async () => {
    setBusy(true);
    setError(null);
    setStatusHint(null);
    try {
      await invoke('vxkey_request_permissions');
      setRequestedOnce(true);
      setStatusHint(
        stream ? (vi ? 'Đã mở mục Trợ năng. Bật VXKey rồi quay lại đây để bắt đầu gõ.' : 'Opened Accessibility. Enable VXKey, then return here to start typing.') : vi
          ? 'Đã mở Cài đặt Quyền của macOS. Sau khi bật công tắc cho VXKey, hãy quay lại đây (hoặc nhấn "Khởi động lại VXKey" nếu vừa cấp quyền Giám sát đầu vào).'
          : 'Opened macOS Permission Settings. After toggling VXKey on, return here (or click "Restart VXKey" if you just granted Input Monitoring).'
      );
    } catch {
      setError(
        stream ? (vi ? 'Mở System Settings → Privacy & Security → Accessibility rồi bật VXKey.' : 'Open System Settings → Privacy & Security → Accessibility and enable VXKey.') : vi
          ? 'Không mở được Cài đặt Hệ thống tự động. Hãy mở System Settings → Privacy & Security → Accessibility và Input Monitoring rồi bật VXKey.'
          : 'Could not open System Settings automatically. Open System Settings → Privacy & Security → Accessibility and Input Monitoring, then enable VXKey.'
      );
    } finally {
      setBusy(false);
      onRefresh();
    }
  };

  const handleRestartApp = async () => {
    setBusy(true);
    setError(null);
    if (!('__TAURI_INTERNALS__' in window)) {
      setBusy(false);
      onRefresh();
      return;
    }
    try {
      await invoke('vxkey_restart_app');
      setBusy(false); // Settings now owns save/discard/cancel; request is not restart completion.
    } catch {
      setBusy(false);
      setError(
        vi
          ? 'Không thể tự khởi động lại. Vui lòng chọn Thoát VXKey trên thanh Menu rồi mở lại ứng dụng.'
          : 'Could not restart automatically. Please choose Quit VXKey from the Menu Bar and reopen the app.'
      );
    }
  };

  const handleStartTyping = async () => {
    if (!ready || busy) return;
    setBusy(true);
    setError(null);
    setStatusHint(null);
    const res = await sendCommand('set_input_suspended', {
      target_session_id: runtime.engine_session_id ?? null,
      suspended: false,
      reason: 'user',
    });
    onRefresh();

    if (!res.ok) {
      setBusy(false);
      setError(
        vi
          ? 'Chưa kích hoạt được bộ gõ. Nếu bạn vừa bật quyền trong macOS, hãy nhấn "Khởi động lại VXKey" để áp dụng quyền mới.'
          : 'Could not activate the input method yet. If you just granted permissions in macOS, click "Restart VXKey" to apply them.'
      );
      return;
    }

    // Poll briefly so the UI transitions cleanly to Active without getting stuck
    for (let i = 0; i < 6; i++) {
      await new Promise(r => setTimeout(r, 180));
      onRefresh();
    }
    setBusy(false);
  };

  const handleDoneToMenubar = () => {
    try {
      localStorage.removeItem('vxkey_setup_pending_resume');
      localStorage.setItem('vxkey_setup_completed_v1', '1');
    } catch {}
    if (onFinishToMenubar) onFinishToMenubar();
    else onLater();
  };

  const handleOpenSettings = () => {
    try {
      localStorage.removeItem('vxkey_setup_pending_resume');
      if (active) localStorage.setItem('vxkey_setup_completed_v1', '1');
    } catch {}
    if (onOpenFullSettings) onOpenFullSettings();
    else onLater();
  };

  const permissions = [
    {
      id: 'accessibility',
      granted: ax,
      title: vi ? 'Trợ năng (Accessibility)' : 'Accessibility',
      desc: vi
        ? 'Cho phép VXKey xử lý phím và ghép chữ tiếng Việt trong ứng dụng đang dùng.'
        : 'Allows VXKey to process keystrokes and compose Vietnamese in your active app.',
    },
    {
      id: 'listen',
      granted: listen,
      title: vi ? 'Đọc phím (Input Monitoring)' : 'Input Monitoring',
      desc: vi
        ? 'Đọc tổ hợp phím Telex / VNI bạn vừa nhấn để ghép âm tiếng Việt.'
        : 'Reads Telex / VNI keystrokes as you type to compose Vietnamese syllables.',
    },
    {
      id: 'post',
      granted: post,
      title: vi ? 'Phát sự kiện phím (Post Events)' : 'Post Keyboard Events',
      desc: vi
        ? 'Đưa chữ tiếng Việt đã bỏ dấu vào ứng dụng bạn đang sử dụng.'
        : 'Delivers composed Vietnamese characters into your active application.',
    },
  ].filter(permission => !stream || permission.id === 'accessibility');

  const grantedCount = permissions.filter(permission => permission.granted).length;
  const needsRestartHint = requestedOnce && !ready;

  return (
    <main
      className="vx-surface w-full h-full flex flex-col justify-between overflow-hidden select-none text-xs"
      aria-labelledby="vx_perm_title"
    >
      {/* Header */}
      <header className="px-6 py-4 border-b border-black/10 dark:border-white/10 flex items-center justify-between gap-4 shrink-0">
        <div className="flex items-center gap-3">
          <div className="w-9 h-9 rounded-xl bg-gradient-to-br from-cyan-500 to-blue-600 flex items-center justify-center text-white shadow-sm shrink-0">
            <ShieldCheck size={20} />
          </div>
          <div>
            <h1 id="vx_perm_title" className="text-base font-bold tracking-tight">
              {vi ? 'Thiết lập bộ gõ VXKey' : 'Set up VXKey Input Method'}
            </h1>
            <p className="text-[11px] opacity-70">
              {vi
                ? 'Cấp quyền, bật bộ gõ và gõ thử trước khi thu về thanh Menu.'
                : 'Grant access, start the input method and test typing before returning to the Menu Bar.'}
            </p>
          </div>
        </div>

        {/* Overall Live Status Pill */}
        <div
          role="status"
          aria-live="polite"
          className={`px-3 py-1.5 rounded-full text-[11px] font-semibold border flex items-center gap-1.5 shrink-0 ${
            active
              ? 'bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 border-emerald-500/30'
              : ready
              ? 'bg-cyan-500/15 text-cyan-600 dark:text-cyan-400 border-cyan-500/30'
              : 'bg-amber-500/15 text-amber-600 dark:text-amber-400 border-amber-500/30'
          }`}
        >
          <span
            className={`w-2 h-2 rounded-full ${
              active
                ? 'bg-emerald-500 animate-pulse'
                : ready
                ? 'bg-cyan-500'
                : 'bg-amber-500'
            }`}
          />
          <span>
            {active
              ? vi
                ? 'Đang hoạt động · Sẵn sàng gõ'
                : 'Active · Ready to type'
              : ready
              ? vi
                ? 'Đã đủ quyền · Chờ kích hoạt'
                : 'Permissions ready · Waiting to start'
              : !connected
              ? vi
                ? 'Bộ gõ nền chưa kết nối'
                : 'Background engine disconnected'
              : vi
              ? `Đã cấp ${grantedCount}/${permissions.length} quyền hệ thống`
              : `${grantedCount}/${permissions.length} system permissions granted`}
          </span>
        </div>
      </header>

      {/* Scrollable Main Content */}
      <div className="flex-1 overflow-y-auto px-6 py-4 space-y-4">
        {/* Stepper Bar */}
        <div className="grid grid-cols-3 gap-2">
          {[
            {
              step: 1,
              done: ready,
              current: !ready,
              label: vi ? '1. Cấp quyền macOS' : '1. Grant macOS access',
            },
            {
              step: 2,
              done: active,
              current: ready && !active,
              label: vi ? '2. Bắt đầu gõ tiếng Việt' : '2. Start Vietnamese input',
            },
            {
              step: 3,
              done: false,
              current: active,
              label: vi ? '3. Gõ thử & Hoàn tất' : '3. Test typing & Finish',
            },
          ].map(item => (
            <div
              key={item.step}
              className={`px-3 py-2 rounded-lg border flex items-center gap-2 transition-all ${
                item.done
                  ? 'border-emerald-500/30 bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 font-semibold'
                  : item.current
                  ? 'border-cyan-500 bg-cyan-500/10 text-cyan-600 dark:text-cyan-400 font-semibold'
                  : 'border-black/10 dark:border-white/10 opacity-55'
              }`}
            >
              <span
                className={`w-4 h-4 rounded-full flex items-center justify-center text-[10px] shrink-0 ${
                  item.done
                    ? 'bg-emerald-500 text-white'
                    : item.current
                    ? 'bg-cyan-500 text-white'
                    : 'bg-black/10 dark:bg-white/10'
                }`}
              >
                {item.done ? <Check size={10} /> : item.step}
              </span>
              <span className="truncate">{item.label}</span>
            </div>
          ))}
        </div>

        {/* 3 Permission Status Cards */}
        <section
          className={`grid grid-cols-1 ${stream ? '' : 'sm:grid-cols-3'} gap-2.5`}
          aria-label={vi ? 'Danh sách quyền hệ thống' : 'System permissions list'}
        >
          {permissions.map(perm => (
            <div
              key={perm.id}
              className={`vx-card p-3 rounded-xl border transition-all flex flex-col justify-between gap-2 ${
                perm.granted
                  ? 'border-emerald-500/25 bg-emerald-500/[0.04]'
                  : 'border-amber-500/30 bg-amber-500/[0.04]'
              }`}
            >
              <div className="space-y-1">
                <div className="flex items-center justify-between gap-1.5">
                  <span className="font-semibold text-xs">{perm.title}</span>
                  <span
                    className={`px-2 py-0.5 rounded-full text-[10px] font-semibold shrink-0 ${
                      perm.granted
                        ? 'bg-emerald-500/15 text-emerald-600 dark:text-emerald-400'
                        : 'bg-amber-500/15 text-amber-600 dark:text-amber-400'
                    }`}
                  >
                    {perm.granted
                      ? vi
                        ? '✓ Đã cấp'
                        : '✓ Granted'
                      : vi
                      ? 'Cần cấp'
                      : 'Required'}
                  </span>
                </div>
                <p className="text-[11px] opacity-70 leading-relaxed">{perm.desc}</p>
              </div>
            </div>
          ))}
        </section>

        {/* Dynamic Action Card */}
        {active ? (
          <section className="p-4 rounded-xl border border-emerald-500/35 bg-emerald-500/10 flex flex-wrap items-center justify-between gap-3">
            <div className="flex items-start gap-3">
              <CheckCircle2
                size={20}
                className="text-emerald-600 dark:text-emerald-400 shrink-0 mt-0.5"
              />
              <div className="space-y-0.5">
                <h2 className="text-sm font-bold text-emerald-700 dark:text-emerald-300">
                  {vi
                    ? 'VXKey đang hoạt động và sẵn sàng gõ tiếng Việt!'
                    : 'VXKey is active and ready to type Vietnamese!'}
                </h2>
                <p className="text-[11px] opacity-80 leading-relaxed">
                  {vi
                    ? 'Hãy gõ thử trong TextEdit hoặc ứng dụng bạn dùng. Khi kết quả đúng, nhấn "Hoàn tất & Thu về thanh Menu" để sử dụng.'
                    : 'Test typing in TextEdit or your usual app. When the result is correct, click "Done & Minimize to Menu Bar".'}
                </p>
              </div>
            </div>
          </section>
        ) : ready ? (
          <section className="p-4 rounded-xl border border-cyan-500/40 bg-cyan-500/10 space-y-3">
            <div className="flex items-start justify-between gap-3">
              <div className="space-y-1">
                <h2 className="text-sm font-bold text-cyan-700 dark:text-cyan-300 flex items-center gap-1.5">
                  <Sparkles size={16} />
                  <span>
                    {vi
                      ? 'Đã cấp đủ quyền hệ thống! Bấm nút bên dưới để kích hoạt bộ gõ'
                      : 'Required permissions granted! Click below to activate the input method'}
                  </span>
                </h2>
                <p className="text-[11px] opacity-80 leading-relaxed">
                  {vi
                    ? 'Bộ gõ hiện đang ở trạng thái chờ. Nhấn "Bắt đầu gõ tiếng Việt" để bật xử lý phím và mở khóa ô gõ thử trực tiếp.'
                    : 'The input method is currently standing by. Click "Start typing Vietnamese" to enable keystroke processing and unlock the live test box.'}
                </p>
              </div>
            </div>

            <div className="flex flex-wrap items-center gap-2.5">
              <button
                type="button"
                disabled={busy}
                onClick={() => void handleStartTyping()}
                className="vx_permission_primary inline-flex items-center gap-2"
              >
                <Play size={15} />
                <span>
                  {busy
                    ? vi
                      ? 'Đang kích hoạt bộ gõ…'
                      : 'Activating input method…'
                    : vi
                    ? 'Bắt đầu gõ tiếng Việt'
                    : 'Start typing Vietnamese'}
                </span>
              </button>

              <button
                type="button"
                disabled={busy}
                onClick={() => void handleRestartApp()}
                className="vx_permission_secondary inline-flex items-center gap-1.5"
              >
                <RotateCcw size={14} />
                <span>{vi ? 'Khởi động lại VXKey' : 'Restart VXKey'}</span>
              </button>
            </div>
          </section>
        ) : (
          <section className="vx-card p-4 rounded-xl border border-black/10 dark:border-white/10 space-y-3">
            <div className="space-y-1.5">
              <h2 className="text-xs font-bold uppercase tracking-wide text-cyan-600 dark:text-cyan-400">
                {vi ? 'Hướng dẫn cấp quyền nhanh' : 'Quick Permission Guide'}
              </h2>
              <ol className="list-decimal pl-4 space-y-1 text-[11px] opacity-85 leading-relaxed">
                <li>
                  {vi
                    ? 'Nhấn "Mở Cài đặt Quyền macOS" bên dưới để mở đúng mục Quyền riêng tư & Bảo mật.'
                    : 'Click "Open macOS Permission Settings" below to open Privacy & Security.'}
                </li>
                <li>
                  {stream ? (vi ? 'Tìm VXKey trong Trợ năng (Accessibility), sau đó bật công tắc.' : 'Find VXKey under Accessibility and turn its switch on.') : vi
                    ? 'Tìm "VXKey" trong danh sách Trợ năng (Accessibility) và Giám sát đầu vào (Input Monitoring), sau đó gạt công tắc sang Bật (ON).'
                    : 'Find "VXKey" under Accessibility and Input Monitoring, then toggle the switch ON.'}
                </li>
                <li>
                  {vi
                    ? 'Nếu đã bật quyền trong macOS mà ô trạng thái vẫn báo "Cần cấp", hãy nhấn "Khởi động lại VXKey" để macOS áp dụng quyền mới ngay.'
                    : 'If you enabled permissions in macOS but the badges above still show "Required", click "Restart VXKey" so macOS applies the new grant immediately.'}
                </li>
              </ol>
            </div>

            {needsRestartHint && (
              <div className="p-2.5 rounded-lg border border-amber-500/30 bg-amber-500/10 flex items-center justify-between gap-3 text-[11px]">
                <div className="flex items-center gap-2 text-amber-700 dark:text-amber-300">
                  <AlertTriangle size={15} className="shrink-0" />
                  <span>
                    {stream ? (vi ? 'Nếu trạng thái chưa cập nhật, kiểm tra lại hoặc khởi động lại VXKey.' : 'If status has not updated, check again or restart VXKey.') : vi
                      ? 'Vừa bật quyền trong System Settings? macOS yêu cầu khởi động lại ứng dụng để quyền Đọc/Phát phím có hiệu lực.'
                      : 'Just toggled permissions in System Settings? macOS requires restarting the app for Input Monitoring / Post Events to take effect.'}
                  </span>
                </div>
              </div>
            )}

            <div className="flex flex-wrap items-center gap-2.5 pt-1">
              <button
                type="button"
                disabled={busy || !connected}
                onClick={() => void handleRequestPermissions()}
                className="vx_permission_primary inline-flex items-center gap-2"
              >
                <ExternalLink size={14} />
                <span>
                  {busy
                    ? vi
                      ? 'Đang mở…'
                      : 'Opening…'
                    : vi
                    ? 'Mở Cài đặt Quyền macOS'
                    : 'Open macOS Permission Settings'}
                </span>
              </button>

              <button
                type="button"
                disabled={busy}
                onClick={() => void handleRestartApp()}
                className="vx_permission_secondary inline-flex items-center gap-1.5"
              >
                <RotateCcw size={14} />
                <span>
                  {vi
                    ? 'Khởi động lại VXKey (nhận quyền mới)'
                    : 'Restart VXKey (apply permissions)'}
                </span>
              </button>

              <button
                type="button"
                disabled={busy}
                onClick={onRefresh}
                className="vx_permission_secondary inline-flex items-center gap-1.5"
              >
                <RefreshCw size={13} />
                <span>{vi ? 'Kiểm tra lại' : 'Check again'}</span>
              </button>
            </div>
          </section>
        )}

        {statusHint && !error && (
          <p role="status" className="text-[11px] text-cyan-600 dark:text-cyan-400 px-1">
            {statusHint}
          </p>
        )}

        {error && (
          <div
            role="alert"
            className="p-3 rounded-lg border border-rose-500/30 bg-rose-500/10 text-rose-600 dark:text-rose-300 text-[11px] flex items-center justify-between gap-3"
          >
            <span>{error}</span>
            <button
              type="button"
              onClick={() => void handleRestartApp()}
              className="px-2.5 py-1 rounded-md bg-rose-600 text-white font-semibold shrink-0"
            >
              {vi ? 'Khởi động lại ngay' : 'Restart now'}
            </button>
          </div>
        )}

        {/* Live Typing Sandbox right on the Setup Screen */}
        <LiveTypingTest snapshot={snapshot} compact autoFocusWhenReady />
      </div>

      {/* Footer */}
      <footer className="px-6 py-3.5 border-t border-black/10 dark:border-white/10 flex items-center justify-between gap-3 shrink-0 bg-black/[0.02] dark:bg-white/[0.02]">
        <button
          type="button"
          disabled={busy}
          onClick={handleOpenSettings}
          className="vx_permission_secondary inline-flex items-center gap-1.5"
        >
          <Sliders size={14} />
          <span>
            {active
              ? vi
                ? 'Vào Cài đặt chi tiết'
                : 'Open Full Settings'
              : vi
              ? 'Để sau, vào Cài đặt'
              : 'Later, open Settings'}
          </span>
        </button>

        <div className="flex items-center gap-2">
          {active ? (
            <button
              type="button"
              disabled={busy}
              onClick={handleDoneToMenubar}
              className="vx_permission_primary inline-flex items-center gap-2"
            >
              <CheckCircle2 size={15} />
              <span>
                {vi
                  ? 'Hoàn tất & Thu về thanh Menu'
                  : 'Done & Minimize to Menu Bar'}
              </span>
            </button>
          ) : ready ? (
            <button
              type="button"
              disabled={busy}
              onClick={() => void handleStartTyping()}
              className="vx_permission_primary inline-flex items-center gap-2"
            >
              <Play size={15} />
              <span>{vi ? 'Bắt đầu gõ tiếng Việt' : 'Start typing Vietnamese'}</span>
            </button>
          ) : (
            <button
              type="button"
              disabled={busy || !connected}
              onClick={() => void handleRequestPermissions()}
              className="vx_permission_primary inline-flex items-center gap-2"
            >
              <ExternalLink size={14} />
              <span>{vi ? 'Mở Cài đặt Quyền macOS' : 'Open macOS Permission Settings'}</span>
            </button>
          )}
        </div>
      </footer>
    </main>
  );
}
