import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { Config, Snapshot } from '../types';
import { uiCopy } from '../services/copy';

interface Props {
  config: Config;
  snapshot: Snapshot;
  language: 'vi' | 'en';
  disabled?: boolean;
  onSystemChange: (patch: Partial<Config['system']>) => void;
  onUiChange: (patch: Partial<Config['ui']>) => void;
}

export function SystemTab({ config, snapshot, language, disabled = false, onSystemChange, onUiChange }: Props) {
  const t = uiCopy[language].system;
  const [permissionError, setPermissionError] = useState(false);
  const [permissionBusy, setPermissionBusy] = useState(false);
  const sys = config.system;
  const ui = config.ui;

  return (
    <div className="vx_business_tab">
      <section className="space-y-3">
        <h4 className="font-semibold text-xs tracking-wider opacity-60 uppercase">{t.title}</h4>

        {/* Hiện biểu tượng thanh menu */}
        <label className="flex items-center gap-2.5 cursor-pointer p-3 rounded-lg border border-white/10 bg-white/5">
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
                  : 'border-white/10 hover:border-white/20 bg-white/5'
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
                {/* Previews */}
                <div className="flex items-center gap-1.5 p-1 bg-black/40 rounded border border-white/10">
                  <img src="/menubar_serious_vi.svg" alt="V" className="w-4 h-4 invert dark:invert-0" />
                  <span className="opacity-30 text-[10px]">/</span>
                  <img src="/menubar_serious_en.svg" alt="E" className="w-4 h-4 invert dark:invert-0" />
                </div>
              </div>
              <p className="text-[11px] opacity-70 leading-relaxed">{t.seriousDesc}</p>
            </label>

            {/* Nhây / Playful Ư / W */}
            <label
              className={`flex flex-col gap-2 p-3.5 rounded-lg border cursor-pointer transition-all ${
                ui.tray_icon_style === 'playful'
                  ? 'border-cyan-500 bg-cyan-500/10 font-medium'
                  : 'border-white/10 hover:border-white/20 bg-white/5'
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
                {/* Previews */}
                <div className="flex items-center gap-1.5 p-1 bg-black/40 rounded border border-white/10">
                  <img src="/menubar_playful_vi.svg" alt="Ư" className="w-4 h-4 invert dark:invert-0" />
                  <span className="opacity-30 text-[10px]">/</span>
                  <img src="/menubar_playful_en.svg" alt="W" className="w-4 h-4 invert dark:invert-0" />
                </div>
              </div>
              <p className="text-[11px] opacity-70 leading-relaxed">{t.playfulDesc}</p>
            </label>
          </div>
        </div>

        <hr className="border-white/10 my-4" />

        {/* Khởi động cùng hệ thống */}
        <div className="p-3.5 rounded-lg border border-white/10 bg-white/5 space-y-1 vx_disabled_setting">
          <label className="flex items-center gap-2.5 cursor-not-allowed">
            <input
              type="checkbox"
              checked={false}
              disabled={true}
              className="accent-cyan-500 rounded"
            />
            <span className="text-xs font-medium">{t.launchAtLogin}</span>
          </label>
          <p className="vx_disabled_hint text-[11px] pl-6 leading-relaxed">
            {t.launchAtLoginDesc}
          </p>
        </div>
      </section>
      <section className="space-y-3">
        <h4>{language === 'vi' ? 'TRẠNG THÁI BỘ GÕ' : 'INPUT METHOD STATUS'}</h4>
        <dl className="p-3.5 rounded-lg border border-white/10 bg-white/5 space-y-2 text-xs">
          <div className="flex justify-between gap-3"><dt>{language === 'vi' ? 'Kết nối' : 'Connection'}</dt>
            <dd>{snapshot.runtime.connection_status === 'connected' ? language === 'vi' ? 'Đã kết nối' : 'Connected' : language === 'vi' ? 'Chưa kết nối' : 'Disconnected'}</dd></div>
          <div className="flex justify-between gap-3"><dt>{language === 'vi' ? 'Bộ gõ chạy nền' : 'Background driver'}</dt>
            <dd>{snapshot.runtime.driver_status === 'active' ? language === 'vi' ? 'Đang dùng' : 'Active' : language === 'vi' ? 'Chưa hoạt động' : 'Inactive'}</dd></div>
          <div className="flex justify-between gap-3"><dt>{language === 'vi' ? 'Xử lý phím' : 'Input processing'}</dt>
            <dd>{snapshot.runtime.connection_status !== 'connected' ? language === 'vi' ? 'Chưa xác nhận' : 'Unconfirmed'
              : snapshot.runtime.health_state === 'healthy' ? language === 'vi' ? 'Sẵn sàng' : 'Ready' : language === 'vi' ? 'Đã tạm ngừng' : 'Suspended'}</dd></div>
          <div className="flex justify-between gap-3"><dt>{language === 'vi' ? 'Cấu hình đã áp dụng / đã lưu' : 'Applied / saved configuration'}</dt>
            <dd className="font-mono">{snapshot.runtime.applied_revision} / {snapshot.revision}</dd></div>
          {snapshot.runtime.driver && <div className="flex justify-between gap-3"><dt>{language === 'vi' ? 'Quyền Trợ năng' : 'Accessibility'}</dt><dd>{snapshot.runtime.driver.accessibility ? language === 'vi' ? 'Đã cấp' : 'Granted' : language === 'vi' ? 'Chưa cấp' : 'Required'}</dd></div>}
          {snapshot.runtime.driver && <div className="flex justify-between gap-3"><dt>{language === 'vi' ? 'Quyền phát phím' : 'Post keyboard events'}</dt><dd>{snapshot.runtime.driver.post_access ? language === 'vi' ? 'Đã cấp' : 'Granted' : language === 'vi' ? 'Chưa cấp' : 'Required'}</dd></div>}
        </dl>
        {snapshot.runtime.driver_fault === 'permission_required' && <p role="status" className="text-xs">{language === 'vi' ? 'Chưa có đủ quyền xử lý phím.' : 'Keyboard permission is required.'}</p>}
        {snapshot.platform === 'macos' && <div className="space-y-2 text-xs">
          <p>{language === 'vi' ? 'Giữ nguyên ABC hoặc nguồn nhập đang dùng. VXKey cần quyền Trợ năng để xử lý phím; sau khi cấp quyền, chọn Tiếp tục trong menu VXKey.' : 'Keep ABC or your current input source. VXKey needs Accessibility permission to process keys; after granting permission, choose Resume in the VXKey menu.'}</p>
          <button type="button" disabled={permissionBusy} className="px-3 py-2 rounded-lg border border-cyan-500/40 bg-cyan-500/10 hover:bg-cyan-500/20 text-cyan-400 disabled:opacity-50" onClick={async () => {
            setPermissionError(false); setPermissionBusy(true);
            try { await invoke('vxkey_request_permissions'); } catch { setPermissionError(true); } finally { setPermissionBusy(false); }
          }}>{language === 'vi' ? 'Thiết lập quyền xử lý phím…' : 'Set up keyboard permissions…'}</button>
          {permissionError && <p role="alert">{language === 'vi' ? 'Chưa gửi được yêu cầu. Kiểm tra kết nối bộ gõ chạy nền.' : 'Request unavailable. Check the background driver connection.'}</p>}
        </div>}
      </section>
    </div>
  );
}
