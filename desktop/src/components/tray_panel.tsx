import { useState, useLayoutEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { Snapshot } from '../types';
import { uiCopy } from '../services/copy';
import { sendCommand } from '../services/api';
import {
  Sliders,
  Clock,
  RefreshCw,
  Info,
  Power,
  Shield,
  Layers,
} from 'lucide-react';

interface Props {
  snapshot: Snapshot;
  onNavigate: (surface: 'settings' | 'history' | 'conversion' | 'onboarding', tab?: string) => void;
  onRefresh: () => void;
}

export function TrayPanel({ snapshot, onNavigate, onRefresh }: Props) {
  const panelRef=useRef<HTMLDivElement>(null);
  useLayoutEffect(() => {
    const panel=panelRef.current;
    if (!panel || !('__TAURI_INTERNALS__' in window)) return;
    let previous=0;
    const resize=() => {
      const height=Math.ceil(panel.getBoundingClientRect().height);
      if (height===previous) return;
      previous=height;
      void invoke('vxkey_fit_tray',{height}).catch(() => {});
    };
    const observer=new ResizeObserver(resize);observer.observe(panel);resize();
    return () => observer.disconnect();
  }, []);
  const language = (snapshot.config.ui.locale === 'en' ? 'en' : 'vi') as 'vi' | 'en';
  const t = uiCopy[language].tray;
  const runtime = snapshot.runtime;
  const [error, setError] = useState<string | null>(null);
  const isPlayful = snapshot.config.ui.tray_icon_style === 'playful';

  const isVi = runtime.effective_mode === 'vi';
  const isEn = runtime.effective_mode === 'en';

  const [busy, setBusy] = useState(false);
  const target = runtime.current_app;
  const canSwitch = snapshot.capabilities.native_engine && !!target?.engine_session_id && !!target?.focus_generation;
  const handleToggleLanguage = async (mode: 'vi' | 'en') => {
    if (!canSwitch || !target || busy) return;
    setBusy(true);
    const result=await sendCommand('set_language',{mode,target:{app_id:target.app_id,pid:target.pid,engine_session_id:target.engine_session_id,focus_generation:target.focus_generation}});
    setBusy(false);
    if (result.ok) onRefresh(); else setError(language === 'vi' ? 'Chưa đổi được chế độ gõ. Hãy mở lại bảng để lấy ứng dụng hiện tại.' : 'Mode was not changed. Reopen the panel to refresh the current app.');
  };
  const handleBypass = async () => {
    if (!target?.target_token || busy) return;
    setBusy(true);
    const result=await sendCommand('set_current_app_bypass',{target_token:target.target_token,expected_revision:snapshot.revision,input_bypass:!target.input_bypass});
    setBusy(false);
    if (result.ok) onRefresh(); else setError(language === 'vi' ? 'Chưa lưu được quy tắc. Hãy mở lại bảng để kiểm tra ứng dụng hiện tại.' : 'The policy was not saved. Reopen the panel to verify the current app.');
  };

  const handleSuspension = async () => {
    setBusy(true);
    const result=await sendCommand('set_input_suspended',{
      target_session_id:runtime.engine_session_id ?? null,
      suspended:runtime.effective_mode !== 'suspended',reason:'user',
    });
    setBusy(false);
    if (!result.ok) setError(language === 'vi' ? 'Chưa đổi được trạng thái xử lý phím. Hãy kiểm tra kết nối bộ gõ.' : 'Input state was not changed. Check the input method connection.');
    onRefresh();
  };

  const getStatusText = () => {
    if (runtime.effective_mode === 'suspended') return t.statusSuspended;
    if (runtime.effective_mode === 'unavailable') return t.statusUnavailable;
    if (runtime.effective_mode === 'bypass') return language === 'vi' ? 'Đã loại trừ ứng dụng này' : 'This application is excluded';
    return isVi || isEn ? t.statusActive : t.statusUnavailable;
  };

  const handleOpenSurface = (target: 'settings' | 'history' | 'conversion' | 'onboarding', tab?: string) => onNavigate(target, tab);

  const handleQuit = async () => {
    if (!('__TAURI_INTERNALS__' in window)) { setError(language === 'vi' ? 'Chỉ khả dụng trong ứng dụng.' : 'Available in the desktop app.'); return; }
    try { await invoke('vxkey_quit'); }
    catch { setError(language === 'vi' ? 'Không thoát được ứng dụng.' : 'Could not quit the app.'); }
  };

  return (
    <div ref={panelRef} className="vx_tray_panel vx-surface p-3 space-y-3 text-xs select-none border border-black/10 dark:border-white/10">
      {error && <p role="alert" className="text-rose-500">{error}</p>}
      {/* Header */}
      <div className="flex items-center justify-between pb-2 border-b border-black/10 dark:border-white/10">
        <div className="flex items-center gap-2">
          <img
            src={isVi || isEn ? `/menubar_${isPlayful ? 'playful' : 'serious'}_${isEn ? 'en' : 'vi'}.svg` : '/app_icon.png'}
            alt="Icon"
            className="w-4 h-4 invert dark:invert-0"
          />
          <span className="font-semibold text-sm tracking-tight">{t.title}</span>
        </div>
        <span
          className={`px-2 py-0.5 rounded text-[10px] font-medium border ${
            runtime.effective_mode === 'suspended'
              ? 'bg-amber-500/10 text-amber-500 dark:text-amber-400 border-amber-500/20'
              : runtime.effective_mode === 'unavailable'
              ? 'bg-rose-500/10 text-rose-500 dark:text-rose-400 border-rose-500/20'
              : 'bg-emerald-500/10 text-emerald-500 dark:text-emerald-400 border-emerald-500/20'
          }`}
        >
          {getStatusText()}
        </span>
      </div>

      {/* Mode selection buttons */}
      <div className="grid grid-cols-2 gap-2">
        <button
          type="button"
          disabled={busy || !canSwitch}
          onClick={() => handleToggleLanguage('vi')}
          className={`flex items-center justify-center gap-2 py-2 px-3 rounded-lg border font-medium transition-all ${
            isVi
              ? 'border-cyan-500 bg-cyan-500/20 text-cyan-600 dark:text-cyan-400 shadow-sm'
              : 'border-black/10 dark:border-white/10 hover:border-black/20 dark:hover:border-white/20 bg-black/5 dark:bg-white/5 opacity-80 hover:opacity-100'
          }`}
        >
          <span>{t.modeVi}</span>
          <span className="text-[10px] opacity-60 font-mono">[VI]</span>
        </button>

        <button
          type="button"
          disabled={busy || !canSwitch}
          onClick={() => handleToggleLanguage('en')}
          className={`flex items-center justify-center gap-2 py-2 px-3 rounded-lg border font-medium transition-all ${
            isEn
              ? 'border-cyan-500 bg-cyan-500/20 text-cyan-600 dark:text-cyan-400 shadow-sm'
              : 'border-black/10 dark:border-white/10 hover:border-black/20 dark:hover:border-white/20 bg-black/5 dark:bg-white/5 opacity-80 hover:opacity-100'
          }`}
        >
          <span>{t.modeEn}</span>
          <span className="text-[10px] opacity-60 font-mono">[EN]</span>
        </button>
      </div>

      {/* Quick Action: App Exclusion */}
      <div className="p-2 rounded-lg bg-black/5 dark:bg-white/5 border border-black/5 dark:border-white/5 text-[11px]">
        {runtime.current_app ? (
          <button
            type="button"
            disabled={busy || !target?.target_token}
            onClick={() => void handleBypass()}
            className="w-full text-left flex items-center justify-between text-cyan-600 dark:text-cyan-400 hover:opacity-80 py-1"
          >
            <span>{target?.input_bypass ? language === 'vi' ? 'Bật VXKey cho' : 'Enable VXKey for' : t.bypassApp} <strong>{runtime.current_app.display_name}</strong></span>
            <Shield size={12} />
          </button>
        ) : (
          <div className="flex items-center justify-between opacity-50 py-0.5">
            <span>{t.appUnavailable}</span>
            <Shield size={12} className="opacity-40" />
          </div>
        )}
      </div>

      <hr className="border-black/10 dark:border-white/10" />

      <button type="button" disabled={busy || (runtime.effective_mode === 'suspended' && !runtime.engine_session_id)}
        onClick={() => void handleSuspension()}
        className="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-lg hover:bg-black/5 dark:hover:bg-white/10 text-left transition-colors disabled:opacity-50">
        <Shield size={14} className="text-cyan-600 dark:text-cyan-400" />
        <span>{runtime.effective_mode === 'suspended'
          ? language === 'vi' ? 'Khôi phục xử lý phím' : 'Resume input processing'
          : language === 'vi' ? 'Tạm ngừng xử lý phím' : 'Suspend input processing'}</span>
      </button>

      {/* Utility links */}
      <div className="space-y-1">
        <button
          type="button"
          onClick={() => handleOpenSurface('settings', 'apps')}
          className="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-lg hover:bg-black/5 dark:hover:bg-white/10 text-left transition-colors"
        >
          <Layers size={14} className="text-cyan-600 dark:text-cyan-400" />
          <span>{t.manageApps}</span>
        </button>

        <button
          type="button"
          onClick={() => handleOpenSurface('history')}
          className="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-lg hover:bg-black/5 dark:hover:bg-white/10 text-left transition-colors"
        >
          <Clock size={14} className="text-cyan-600 dark:text-cyan-400" />
          <span>{t.history}</span>
        </button>

        <button
          type="button"
          onClick={() => handleOpenSurface('conversion')}
          className="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-lg hover:bg-black/5 dark:hover:bg-white/10 text-left transition-colors"
        >
          <RefreshCw size={14} className="text-cyan-600 dark:text-cyan-400" />
          <span>{t.conversion}</span>
        </button>

        <button
          type="button"
          onClick={() => handleOpenSurface('settings')}
          className="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg hover:bg-black/5 dark:hover:bg-white/10 text-left transition-colors"
        >
          <div className="flex items-center gap-2.5">
            <Sliders size={14} className="text-cyan-600 dark:text-cyan-400" />
            <span>{t.settings}</span>
          </div>
          <span className="text-[10px] opacity-40 font-mono">⌘,</span>
        </button>

        <button
          type="button"
          onClick={async () => {
            if (!('__TAURI_INTERNALS__' in window)) {handleOpenSurface('settings','about');return;}
            try {await invoke('vxkey_show_about');}
            catch {setError(language === 'vi' ? 'Không mở được Giới thiệu VXKey.' : 'Could not open About VXKey.');}
          }}
          className="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-lg hover:bg-black/5 dark:hover:bg-white/10 text-left transition-colors"
        >
          <Info size={14} className="text-cyan-600 dark:text-cyan-400" />
          <span>{t.about}</span>
        </button>
      </div>

      <hr className="border-black/10 dark:border-white/10" />

      {/* Quit */}
      <button
        type="button"
        onClick={handleQuit}
        className="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-lg text-rose-500 hover:bg-rose-500/10 text-left transition-colors"
      >
        <Power size={14} />
        <span>{t.quit}</span>
      </button>
    </div>
  );
}
