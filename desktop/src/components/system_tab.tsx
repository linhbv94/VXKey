import { nativePermissions } from '../services/native_permissions';
import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { Config, Snapshot } from '../types';
import { uiCopy } from '../services/copy';
import { sendCommand } from '../services/api';
import {
  ShieldCheck,
  Play,
  Pause,
  RotateCcw,
  ExternalLink,
  CheckCircle2,
  AlertTriangle,
} from 'lucide-react';

interface Props {
  config: Config;
  snapshot: Snapshot;
  language: 'vi' | 'en';
  disabled?: boolean;
  onSystemChange: (patch: Partial<Config['system']>) => void;
  onUiChange: (patch: Partial<Config['ui']>) => void;
  onRefresh?: () => void;
  onPermissionSetup?: () => void;
}

export function SystemTab({
  config,
  snapshot,
  language,
  disabled = false,
  onSystemChange,
  onUiChange,
  onRefresh,
  onPermissionSetup,
}: Props) {
  const vi = language === 'vi';
  const t = uiCopy[language].system;
  const [actionBusy, setActionBusy] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);
  const sys = config.system;
  const ui = config.ui;
  const runtime = snapshot.runtime;
  const driver = runtime.driver;

  const connected = runtime.connection_status === 'connected';
  const { stream, ax, listen, post, ready: permissionsReady } = nativePermissions(snapshot);
  const isActive =
    permissionsReady &&
    (snapshot.platform === 'windows' || Boolean(driver?.attached)) &&
    runtime.driver_status === 'active' &&
    runtime.effective_mode !== 'suspended';

  const handleToggleSuspend = async () => {
    if (!connected || actionBusy) return;
    setActionBusy(true);
    setActionError(null);
    const nextSuspended = isActive;
    const res = await sendCommand('set_input_suspended', {
      target_session_id: runtime.engine_session_id ?? null,
      suspended: nextSuspended,
      reason: 'user',
    });
    setActionBusy(false);
    if (!res.ok) {
      setActionError(
        snapshot.platform === 'windows' ? (vi ? 'Chưa đổi được trạng thái bộ gõ Windows. Hãy kiểm tra kết nối hoặc thoát và mở lại VXKey.' : 'The Windows input state was not changed. Check the connection or quit and reopen VXKey.') : vi
          ? 'Chưa đổi được trạng thái bộ gõ. Nếu vừa cấp quyền trong macOS, hãy nhấn "Khởi động lại VXKey".'
          : 'Could not change input method state. If you just granted permissions in macOS, click "Restart VXKey".'
      );
    }
    onRefresh?.();
  };

  const handleRestart = async () => {
    setActionBusy(true);
    setActionError(null);
    if (!('__TAURI_INTERNALS__' in window)) {
      setActionBusy(false);
      onRefresh?.();
      return;
    }
    try {
      await invoke('vxkey_restart_app');
      setActionBusy(false); // Guarded request may be cancelled without stopping the helper.
    } catch {
      setActionBusy(false);
      setActionError(
        vi
          ? 'Không thể tự khởi động lại. Vui lòng thoát và mở lại VXKey từ thanh Menu.'
          : 'Could not restart automatically. Please quit and reopen VXKey from the Menu Bar.'
      );
    }
  };

  const handleRequestPermissions = async () => {
    if (onPermissionSetup) {
      onPermissionSetup();
      return;
    }
    setActionBusy(true);
    setActionError(null);
    try {
      await invoke('vxkey_request_permissions');
    } catch {
      setActionError(
        vi
          ? 'Chưa gửi được yêu cầu. Kiểm tra kết nối bộ gõ chạy nền.'
          : 'Request unavailable. Check the background driver connection.'
      );
    } finally {
      setActionBusy(false);
      onRefresh?.();
    }
  };

  const renderBadge = (ok: boolean, okText: string, warnText: string) => (
    <span
      className={`px-2 py-0.5 rounded-full text-[11px] font-semibold ${
        ok
          ? 'bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 border border-emerald-500/25'
          : 'bg-amber-500/15 text-amber-600 dark:text-amber-400 border border-amber-500/25'
      }`}
    >
      {ok ? okText : warnText}
    </span>
  );

  return (
    <div className="vx_business_tab space-y-5">
      {/* Trạng thái bộ gõ & Quyền hệ thống */}
      <section className="space-y-3">
        <div className="flex items-center justify-between gap-2">
          <h4 className="font-semibold text-xs tracking-wider opacity-60 uppercase">
            {vi ? 'TRẠNG THÁI BỘ GÕ & QUYỀN HỆ THỐNG' : 'INPUT METHOD & PERMISSIONS STATUS'}
          </h4>
          <span
            className={`inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-[11px] font-semibold border ${
              isActive
                ? 'bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 border-emerald-500/30'
                : permissionsReady
                ? 'bg-cyan-500/15 text-cyan-600 dark:text-cyan-400 border-cyan-500/30'
                : 'bg-amber-500/15 text-amber-600 dark:text-amber-400 border-amber-500/30'
            }`}
          >
            {isActive ? <CheckCircle2 size={12} /> : <AlertTriangle size={12} />}
            <span>
              {isActive
                ? vi
                  ? 'Đang hoạt động'
                  : 'Active'
                : permissionsReady
                ? vi
                  ? 'Đang tạm ngừng'
                  : 'Suspended'
                : vi
                ? 'Cần cấp quyền'
                : 'Permissions needed'}
            </span>
          </span>
        </div>

        <dl className="vx-card p-3.5 rounded-xl border border-black/10 dark:border-white/10 space-y-2.5 text-xs">
          <div className="flex items-center justify-between gap-3">
            <dt className="opacity-75">{vi ? 'Kết nối bộ gõ nền' : 'Background connection'}</dt>
            <dd>
              {renderBadge(
                connected,
                vi ? 'Đã kết nối' : 'Connected',
                vi ? 'Chưa kết nối' : 'Disconnected'
              )}
            </dd>
          </div>
          <div className="flex items-center justify-between gap-3">
            <dt className="opacity-75">{vi ? 'Bộ gõ chạy nền' : 'Background driver'}</dt>
            <dd>
              {renderBadge(
                runtime.driver_status === 'active',
                vi ? 'Đang dùng' : 'Active',
                vi ? 'Chưa hoạt động' : 'Inactive'
              )}
            </dd>
          </div>
          <div className="flex items-center justify-between gap-3">
            <dt className="opacity-75">{vi ? 'Xử lý phím' : 'Input processing'}</dt>
            <dd>
              {!connected
                ? renderBadge(false, '', vi ? 'Chưa xác nhận' : 'Unconfirmed')
                : renderBadge(
                    runtime.health_state === 'healthy' && runtime.effective_mode !== 'suspended',
                    vi ? 'Sẵn sàng' : 'Ready',
                    vi ? 'Đã tạm ngừng' : 'Suspended'
                  )}
            </dd>
          </div>
          {driver && (
            <>
              <div className="flex items-center justify-between gap-3">
                <dt className="opacity-75">{vi ? 'Quyền Trợ năng (Accessibility)' : 'Accessibility'}</dt>
                <dd>{renderBadge(ax, vi ? 'Đã cấp' : 'Granted', vi ? 'Chưa cấp' : 'Required')}</dd>
              </div>
              {!stream && <>
              <div className="flex items-center justify-between gap-3">
                <dt className="opacity-75">{vi ? 'Quyền Đọc phím (Input Monitoring)' : 'Input Monitoring'}</dt>
                <dd>{renderBadge(listen, vi ? 'Đã cấp' : 'Granted', vi ? 'Chưa cấp' : 'Required')}</dd>
              </div>
              <div className="flex items-center justify-between gap-3">
                <dt className="opacity-75">{vi ? 'Quyền Phát phím (Post Events)' : 'Post keyboard events'}</dt>
                <dd>{renderBadge(post, vi ? 'Đã cấp' : 'Granted', vi ? 'Chưa cấp' : 'Required')}</dd>
              </div>
              </>}
            </>
          )}
          <div className="flex items-center justify-between gap-3 pt-1 border-t border-black/10 dark:border-white/10">
            <dt className="opacity-75">
              {vi ? 'Cấu hình đã áp dụng / đã lưu' : 'Applied / saved configuration'}
            </dt>
            <dd className="font-mono text-[11px] opacity-80">
              {runtime.applied_revision} / {snapshot.revision}
            </dd>
          </div>
        </dl>

        {snapshot.platform === 'windows' && (
          <div className="space-y-2 pt-2">
            <p className="text-xs opacity-70">{vi ? 'Không cần quyền quản trị. VXKey xử lý ứng dụng chạy quyền thường; ứng dụng chạy quyền Administrator được bỏ qua.' : 'Administrator access is not required. VXKey processes standard-user applications; elevated applications are bypassed.'}</p>
            <button type="button" disabled={!connected || actionBusy} onClick={() => void handleToggleSuspend()} className="px-3 py-2 rounded-lg bg-cyan-600 text-white text-xs">
              {isActive ? (vi ? 'Tạm ngừng xử lý phím' : 'Pause input processing') : (vi ? 'Bắt đầu gõ tiếng Việt' : 'Start typing Vietnamese')}
            </button>
          </div>
        )}
        {snapshot.platform === 'macos' && (
          <div className="flex flex-wrap items-center gap-2 pt-1">
            {permissionsReady ? (
              <button
                type="button"
                disabled={disabled || actionBusy}
                onClick={() => void handleToggleSuspend()}
                className="vx_permission_primary inline-flex items-center gap-1.5 text-xs"
              >
                {isActive ? <Pause size={13} /> : <Play size={13} />}
                <span>
                  {isActive
                    ? vi
                      ? 'Tạm ngừng xử lý phím'
                      : 'Suspend input processing'
                    : vi
                    ? 'Bắt đầu gõ tiếng Việt'
                    : 'Resume input processing'}
                </span>
              </button>
            ) : (
              <button
                type="button"
                disabled={disabled || actionBusy}
                onClick={() => void handleRequestPermissions()}
                className="vx_permission_primary inline-flex items-center gap-1.5 text-xs"
              >
                <ExternalLink size={13} />
                <span>
                  {vi ? 'Thiết lập quyền bàn phím…' : 'Set up keyboard permissions…'}
                </span>
              </button>
            )}

            {onPermissionSetup && (
              <button
                type="button"
                disabled={disabled || actionBusy}
                onClick={onPermissionSetup}
                className="vx_permission_secondary inline-flex items-center gap-1.5 text-xs"
              >
                <ShieldCheck size={13} />
                <span>
                  {vi ? 'Mở màn hình Thiết lập & Gõ thử' : 'Open Setup & Live Test'}
                </span>
              </button>
            )}

            <button
              type="button"
              disabled={disabled || actionBusy}
              onClick={() => void handleRestart()}
              className="vx_permission_secondary inline-flex items-center gap-1.5 text-xs"
            >
              <RotateCcw size={13} />
              <span>{vi ? 'Khởi động lại VXKey' : 'Restart VXKey'}</span>
            </button>
          </div>
        )}

        {actionError && (
          <p role="alert" className="text-xs text-rose-500 dark:text-rose-400">
            {actionError}
          </p>
        )}
      </section>

      <hr className="border-black/10 dark:border-white/10" />

      {/* Tùy chọn thanh menu & Khởi động */}
      <section className="space-y-3">
        <h4 className="font-semibold text-xs tracking-wider opacity-60 uppercase">{t.title}</h4>

        {/* Hiện biểu tượng thanh menu */}
        <label className="flex items-center gap-2.5 cursor-pointer p-3 rounded-lg border border-black/10 dark:border-white/10 bg-black/5 dark:bg-white/5">
          <input
            type="checkbox"
            checked={sys.show_tray_icon}
            disabled={disabled}
            onChange={e => onSystemChange({ show_tray_icon: e.target.checked })}
            className="accent-cyan-500 rounded"
          />
          <span className="text-xs font-medium">{t.showTray}</span>
        </label>

        {/* Kiểu biểu tượng menu bar */}
        <div className="space-y-2 pt-2">
          <label className="block text-xs font-medium opacity-80">{t.trayStyleTitle}</label>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            {/* Nghiêm túc V / E */}
            <label
              className={`flex flex-col gap-2 p-3.5 rounded-lg border cursor-pointer transition-all ${
                ui.tray_icon_style === 'serious'
                  ? 'border-cyan-500 bg-cyan-500/10 font-medium'
                  : 'border-black/10 dark:border-white/10 hover:border-cyan-500/30 bg-black/5 dark:bg-white/5'
              } ${disabled ? 'opacity-50 pointer-events-none' : ''}`}
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <input
                    type="radio"
                    name="tray_style"
                    value="serious"
                    checked={ui.tray_icon_style === 'serious'}
                    disabled={disabled}
                    onChange={() => onUiChange({ tray_icon_style: 'serious' })}
                    className="accent-cyan-500"
                  />
                  <span className="text-xs">{t.serious}</span>
                </div>
                <div className="flex items-center gap-1.5 p-1 bg-black/40 rounded border border-white/10">
                  <img
                    src="/menubar_serious_vi.svg"
                    alt="V"
                    className="w-4 h-4 invert dark:invert-0"
                  />
                  <span className="opacity-30 text-[10px]">/</span>
                  <img
                    src="/menubar_serious_en.svg"
                    alt="E"
                    className="w-4 h-4 invert dark:invert-0"
                  />
                </div>
              </div>
              <p className="text-[11px] opacity-70 leading-relaxed">{t.seriousDesc}</p>
            </label>

            {/* Nhây / Playful Ư / W */}
            <label
              className={`flex flex-col gap-2 p-3.5 rounded-lg border cursor-pointer transition-all ${
                ui.tray_icon_style === 'playful'
                  ? 'border-cyan-500 bg-cyan-500/10 font-medium'
                  : 'border-black/10 dark:border-white/10 hover:border-cyan-500/30 bg-black/5 dark:bg-white/5'
              } ${disabled ? 'opacity-50 pointer-events-none' : ''}`}
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <input
                    type="radio"
                    name="tray_style"
                    value="playful"
                    checked={ui.tray_icon_style === 'playful'}
                    disabled={disabled}
                    onChange={() => onUiChange({ tray_icon_style: 'playful' })}
                    className="accent-cyan-500"
                  />
                  <span className="text-xs">{t.playful}</span>
                </div>
                <div className="flex items-center gap-1.5 p-1 bg-black/40 rounded border border-white/10">
                  <img
                    src="/menubar_playful_vi.svg"
                    alt="Ư"
                    className="w-4 h-4 invert dark:invert-0"
                  />
                  <span className="opacity-30 text-[10px]">/</span>
                  <img
                    src="/menubar_playful_en.svg"
                    alt="W"
                    className="w-4 h-4 invert dark:invert-0"
                  />
                </div>
              </div>
              <p className="text-[11px] opacity-70 leading-relaxed">{t.playfulDesc}</p>
            </label>
          </div>
        </div>

        <hr className="border-black/10 dark:border-white/10 my-4" />

        {/* Khởi động cùng hệ thống */}
        <div className={`p-3.5 rounded-lg border border-black/10 dark:border-white/10 bg-black/5 dark:bg-white/5 space-y-1 ${snapshot.capabilities.autostart ? '' : 'vx_disabled_setting'}`}>
          <label className="flex items-center gap-2.5">
            <input
              type="checkbox"
              checked={sys.launch_companion_at_login}
              disabled={disabled || !snapshot.capabilities.autostart}
              onChange={e => onSystemChange({launch_companion_at_login:e.target.checked})}
              className="accent-cyan-500 rounded"
            />
            <span className="text-xs font-medium">{t.launchAtLogin}</span>
          </label>
          <p className="vx_disabled_hint text-[11px] pl-6 leading-relaxed">{snapshot.platform === 'windows' ? (vi ? 'Mở VXKey cùng Windows. Bộ gõ luôn khởi động ở trạng thái tạm dừng; chọn Bắt đầu khi sẵn sàng.' : 'Open VXKey with Windows. Input always starts paused; choose Start when ready.') : t.launchAtLoginDesc}</p>
        </div>
      </section>
    </div>
  );
}
