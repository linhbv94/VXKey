import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useState, useEffect, useRef } from 'react';
import type { Snapshot } from '../types';
import {
  SettingsShell,
  GeneralSettings,
  UpdatesSettings,
  AboutSettings,
  SupportSettings,
  UpdateStatus,
  type SettingsTab,
  type SettingsTheme,
} from './shared/settings_ui';
import { uiCopy } from '../services/copy';
import { useAppUpdater, checkForUpdates, installUpdate, restartAfterUpdate, dismissUpdateNotice } from '../services/app_updater';
import { sendCommand } from '../services/api';
import { TypingTab } from './typing_tab';
import { AppsTab } from './apps_tab';
import { MacrosTab, type MacroDraftHandle } from './macros_tab';
import { ShortcutsTab } from './shortcuts_tab';
import { ClipboardTab } from './clipboard_tab';
import { SystemTab } from './system_tab';
import {
  Sliders,
  Sparkles,
  Info,
  Coffee,
  Type,
  LayoutGrid,
  Code,
  Keyboard,
  ClipboardList,
  Cpu,
} from 'lucide-react';

interface Props {
  snapshot: Snapshot;
  onRefresh: () => void;
  onClose?: () => void;
}

export function SettingsView({ snapshot, onRefresh, onClose = () => {} }: Props) {
  const [activeTab, setActiveTab] = useState('general');
  const macroDraft = useRef<MacroDraftHandle | null>(null);
  const [resetConfirmation,setResetConfirmation] = useState<{id:string;revision:string}|null>(null);
  const resetRef=useRef(resetConfirmation); resetRef.current=resetConfirmation;
  const [pendingAction, setPendingAction] = useState<string | null>(null);
  const [resolvingDraft, setResolvingDraft] = useState(false);
  const pendingRef = useRef<string | null>(null);
  const performAction = (action: string) => {
    if (action === 'close') onClose();
    else if (action === 'quit') void invoke('vxkey_quit', {confirmed:true}).catch(() => setNotice('VXKey: quit failed'));
    else if (action === 'reset') void beginReset();
    else setActiveTab(action);
  };
  const guardAction = (action: string) => {
    if (resetRef.current) {setResetConfirmation(null);return;}
    if (macroDraft.current?.dirty) {pendingRef.current = action; setPendingAction(action);}
    else performAction(action);
  };
  const quitRef = useRef(() => guardAction('quit'));
  quitRef.current = () => guardAction('quit');
  const closeRef = useRef(() => guardAction('close'));
  closeRef.current = () => guardAction('close');
  useEffect(() => {
    if (!('__TAURI_INTERNALS__' in window)) return;
    let stop: (() => void) | undefined;
    let disposed = false;
    void (async () => {
      const unlisten = await listen<string>('settings_tab', event => guardAction(event.payload));
      const unclose = await listen('request_close', () => closeRef.current());
      const unquit = await listen('request_quit', () => quitRef.current());
      const cleanup = () => {unlisten(); unclose(); unquit();};
      if (disposed) cleanup(); else stop = cleanup;
    })();
    return () => {disposed = true; stop?.();};
  }, []);
  const update = useAppUpdater();
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    const guard = (event: Event) => { if (busy || resolvingDraft || macroDraft.current?.dirty || resetRef.current) event.preventDefault(); };
    window.addEventListener('vx-before-update', guard);
    return () => window.removeEventListener('vx-before-update', guard);
  }, [busy, resolvingDraft]);
  const [notice, setNotice] = useState<string | null>(null);

  const config = snapshot.config;
  const language = (config.ui.locale === 'en' ? 'en' : 'vi') as 'vi' | 'en';
  const t = uiCopy[language];

  // Map theme from config to SettingsTheme
  const currentTheme = (config.ui.theme === 'pure_black' ? 'black' : config.ui.theme) as SettingsTheme;
  const availableThemes: SettingsTheme[] = ['system', 'dark', 'light', 'black'];

  const showToast = (msg: string) => {
    setNotice(msg);
    setTimeout(() => setNotice(null), 3500);
  };

  const handlePatch = async (changes: Record<string, any>) => {
    setBusy(true);
    const res = await sendCommand('patch_config', {
      expected_revision: snapshot.revision,
      changes,
    });
    setBusy(false);
    if (res.ok) {
      showToast(t.savedNotice);
      onRefresh();
    } else {
      if (res.error?.code === 'revision_conflict') {
        showToast(t.conflictNotice);
        onRefresh();
      } else if (res.error?.code === 'capability_locked') {
        showToast(language === 'vi' ? 'Tính năng này đang bị khóa bởi hệ điều hành.' : 'This capability is currently locked by the OS.');
      } else if (res.error?.code === 'durability_uncertain') {
        showToast(language === 'vi' ? 'Lưu chưa đảm bảo độ bền đĩa. Đang tải lại cấu hình…' : 'Durability uncertain. Reloading config…');
        onRefresh();
      } else {
        showToast(t.saveFailed);
      }
    }
  };

  const beginReset = async () => {
    if (!('__TAURI_INTERNALS__' in window)) {showToast(language === 'vi' ? 'Chỉ khả dụng trong ứng dụng.' : 'Available in the desktop app.');return;}
    try {
      const id=await invoke<string>('vxkey_prepare_reset',{expectedRevision:snapshot.revision});
      setResetConfirmation({id,revision:snapshot.revision});
    } catch {showToast(t.conflictNotice);onRefresh();}
  };
  const handleReset = async () => {
    if (!resetConfirmation || busy) return;
    setBusy(true);
    const result=await sendCommand('reset_preferences',{expected_revision:resetConfirmation.revision,confirmation_id:resetConfirmation.id});
    setBusy(false);setResetConfirmation(null);
    showToast(result.ok ? t.savedNotice : t.saveFailed);
    onRefresh();
  };

  // Define tabs
  const tabs: SettingsTab[] = [
    { id: 'general', label: t.tabs.general, icon: <Sliders size={16} /> },
    { id: 'typing', label: t.tabs.typing, icon: <Type size={16} /> },
    { id: 'apps', label: t.tabs.apps, icon: <LayoutGrid size={16} /> },
    { id: 'macros', label: t.tabs.macros, icon: <Code size={16} /> },
    { id: 'shortcuts', label: t.tabs.shortcuts, icon: <Keyboard size={16} /> },
    { id: 'clipboard', label: t.tabs.clipboard, icon: <ClipboardList size={16} /> },
    { id: 'system', label: t.tabs.system, icon: <Cpu size={16} /> },
    { id: 'updates', label: t.tabs.updates, icon: <Sparkles size={16} />, information: true },
    { id: 'about', label: t.tabs.about, icon: <Info size={16} />, information: true },
    { id: 'support', label: t.tabs.support, icon: <Coffee size={16} />, information: true },
  ];

  return (
    <div className="vx_settings_window w-full h-full relative">
      {notice && (
        <div className="absolute top-4 left-1/2 -translate-x-1/2 z-50 bg-cyan-600/90 text-white text-xs px-4 py-2 rounded-full shadow-lg backdrop-blur border border-white/20 animate-fade-in pointer-events-none">
          {notice}
        </div>
      )}

      {resetConfirmation && <div className="absolute inset-0 z-[100] bg-black/50 flex items-center justify-center p-6">
        <div onKeyDown={event => {
          if (event.key !== 'Tab') return;
          const buttons=Array.from(event.currentTarget.querySelectorAll<HTMLButtonElement>('button:not(:disabled)'));
          const first=buttons[0],last=buttons[buttons.length-1];
          if (event.shiftKey && document.activeElement===first) {event.preventDefault();last?.focus();}
          else if (!event.shiftKey && document.activeElement===last) {event.preventDefault();first?.focus();}
          event.stopPropagation();
        }} role="alertdialog" aria-modal="true" aria-labelledby="reset_scope_title" className="vx-surface p-5 rounded-xl max-w-sm text-sm shadow-2xl">
          <h3 id="reset_scope_title" className="font-semibold">{language === 'vi' ? 'Khôi phục Giao diện và Bộ gõ?' : 'Reset Theme and Typing?'}</h3>
          <p className="my-3 text-xs opacity-70">{language === 'vi' ? 'Giữ nguyên ngôn ngữ giao diện, gõ tắt, quy tắc ứng dụng, phím tắt, lịch sử và khởi động cùng máy.' : 'Keeps UI language, macros, app policies, shortcuts, history, and startup preferences.'}</p>
          <div className="flex gap-3 justify-end text-xs">
            <button autoFocus disabled={busy} type="button" onClick={()=>setResetConfirmation(null)}>{language === 'vi' ? 'Hủy' : 'Cancel'}</button>
            <button disabled={busy} type="button" onClick={()=>void handleReset()} className="rounded bg-cyan-600 text-white px-3 py-1">{language === 'vi' ? 'Khôi phục' : 'Reset'}</button>
          </div>
        </div>
      </div>}
      {pendingAction && <div className="absolute inset-0 z-[100] bg-black/50 flex items-center justify-center p-6" role="presentation">
        <div onKeyDown={event => {
          if (event.key === 'Tab') {
            const buttons = Array.from(event.currentTarget.querySelectorAll<HTMLButtonElement>('button:not(:disabled)'));
            const first=buttons[0], last=buttons[buttons.length-1];
            if (event.shiftKey && document.activeElement===first) {event.preventDefault();last?.focus();}
            else if (!event.shiftKey && document.activeElement===last) {event.preventDefault();first?.focus();}
            event.stopPropagation();
          }
        }} role="alertdialog" aria-modal="true" aria-labelledby="macro_unsaved_title" className="vx-surface rounded-xl p-5 max-w-sm text-sm shadow-2xl">
          <h3 id="macro_unsaved_title" className="font-semibold">{language === 'vi' ? 'Có thay đổi gõ tắt chưa lưu' : 'Unsaved macro changes'}</h3>
          <p className="text-xs opacity-70 my-3">{language === 'vi' ? 'Lưu hoặc bỏ nháp trước khi tiếp tục. Nếu lưu lỗi, nháp sẽ được giữ lại.' : 'Save or discard the draft before continuing. Failed saves keep the draft.'}</p>
          <div className="flex gap-2 text-xs">
            <button autoFocus disabled={resolvingDraft} type="button" onClick={() => {pendingRef.current=null;setPendingAction(null);}}>{language === 'vi' ? 'Ở lại' : 'Stay'}</button>
            <button disabled={resolvingDraft} type="button" onClick={() => {macroDraft.current?.discard(); const action=pendingRef.current;pendingRef.current=null;setPendingAction(null);if(action)performAction(action);}}>{language === 'vi' ? 'Bỏ nháp' : 'Discard'}</button>
            <button disabled={resolvingDraft} type="button" className="bg-cyan-600 text-white px-3 py-1 rounded" onClick={async () => {
              setResolvingDraft(true);
              const saved = await macroDraft.current?.save();
              setResolvingDraft(false);
              if (saved) {const action=pendingRef.current;pendingRef.current=null;setPendingAction(null);if(action)performAction(action);}
            }}>{language === 'vi' ? 'Lưu' : 'Save'}</button>
          </div>
        </div>
      </div>}
      <SettingsShell
        appName="VXKey"
        language={language}
        theme={currentTheme}
        tabs={tabs}
        activeTab={activeTab}
        onTabChange={guardAction}
        onClose={() => guardAction('close')}
        canEscape={() => !pendingRef.current}
        canTrapTab={() => !pendingRef.current && !resetRef.current}
        onEscape={() => {if (busy || resolvingDraft) return; if (resetRef.current) {setResetConfirmation(null);return;} if (pendingRef.current) {pendingRef.current=null;setPendingAction(null);} else guardAction('close');}}
        onReset={() => guardAction('reset')}
        disabled={busy}
      >
        {activeTab === 'general' && (
          <GeneralSettings
            language={language}
            theme={currentTheme}
            themes={availableThemes}
            disabled={busy}
            onThemeChange={theme =>
              handlePatch({ ui: { theme: theme === 'black' ? 'pure_black' : theme } })
            }
            onLanguageChange={locale => handlePatch({ ui: { locale } })}
          />
        )}

        {activeTab === 'typing' && (
          <div>
          {!snapshot.capabilities.native_engine && <p role="status" className="vx_typing_notice">{language === 'vi' ? 'Bộ gõ chưa kết nối. Bạn vẫn có thể lưu thiết lập.' : 'The input method is disconnected. You can still save your preferences.'}</p>}
          <TypingTab
            config={config}
            runtime={snapshot.runtime}
            language={language}
            disabled={busy}
            onChange={patch => handlePatch({ typing: patch })}
          /></div>
        )}

        {activeTab === 'apps' && (
          <AppsTab
            config={config}
            language={language}
            disabled={busy}
            onRememberChange={remember_per_app => handlePatch({ apps: { remember_per_app } })}
            onForgetLanguage={async appId => {
              setBusy(true);
              const res = await sendCommand('forget_app_language', {
                expected_revision: snapshot.revision,
                app_id: appId,
              });
              setBusy(false);
              if (res.ok) {
                showToast(language === 'vi' ? `Đã xoá ghi nhớ cho ${appId}` : `Forgot memory for ${appId}`);
                onRefresh();
              } else {
                showToast(t.saveFailed);
              }
            }}
            onSetPolicy={async (appId, inputBypass, clipboardBlocked) => {
              setBusy(true);
              const res = await sendCommand('set_app_policy', {
                expected_revision: snapshot.revision,
                app_id: appId,
                input_bypass: inputBypass,
                clipboard_blocked: clipboardBlocked,
              });
              setBusy(false);
              if (res.ok) {
                showToast(t.savedNotice);
                onRefresh();
              } else {
                showToast(t.saveFailed);
              }
            }}
            onRemovePolicy={async appId => {
              setBusy(true);
              const res = await sendCommand('set_app_policy', {
                expected_revision: snapshot.revision,
                app_id: appId,
                input_bypass: false,
                clipboard_blocked: false,
              });
              setBusy(false);
              if (res.ok) {
                showToast(language === 'vi' ? `Đã gỡ quy tắc cho ${appId}` : `Removed policy for ${appId}`);
                onRefresh();
              } else {
                showToast(t.saveFailed);
              }
            }}
          />
        )}

        {activeTab === 'macros' && (
          <MacrosTab
            config={config}
            language={language}
            disabled={busy}
            onRefresh={onRefresh}
            onNotice={showToast}
            draftRef={macroDraft}
          />
        )}

        {activeTab === 'shortcuts' && (
          <ShortcutsTab
            config={config}
            snapshot={snapshot}
            language={language}
            disabled={busy}
          />
        )}

        {activeTab === 'clipboard' && (
          <ClipboardTab
            config={config}
            language={language}
            disabled={busy}
            onChange={patch => handlePatch({ clipboard: patch })}
          />
        )}

        {activeTab === 'system' && (
          <SystemTab
            snapshot={snapshot}
            config={config}
            language={language}
            disabled={busy}
            onSystemChange={patch => handlePatch({ system: patch })}
            onUiChange={patch => handlePatch({ ui: patch })}
          />
        )}

        {activeTab === 'general' && (
          <section className="rounded-lg border border-cyan-500/30 p-3 text-sm space-y-2" aria-live="polite">
            <h3>{language === 'vi' ? 'Quyền xử lý bàn phím' : 'Keyboard permissions'}</h3>
            <p>{language === 'vi' ? 'VXKey Engine cần Accessibility và Input Monitoring để xử lý phím trên máy. Không cần thêm Input Source. Sau khi cho phép trong System Settings, quay lại đây kiểm tra rồi bấm Resume.' : 'VXKey Engine needs Accessibility and Input Monitoring to process keys locally. No Input Source is required. Return from System Settings, check permissions, then choose Resume.'}</p>
            <p>Accessibility: {String(snapshot.runtime.driver?.accessibility ?? false)} · Input Monitoring: {String(snapshot.runtime.driver?.listen_access ?? false)} · {snapshot.runtime.connection_status}</p>
            <button disabled={busy || !snapshot.runtime.engine_session_id} onClick={() => void invoke('vxkey_request_permissions').then(onRefresh).catch(() => setNotice(language === 'vi' ? 'Chưa yêu cầu được quyền; kiểm tra kết nối VXKey Engine rồi thử lại.' : 'Permission request failed; check VXKey Engine connection and retry.'))}>{language === 'vi' ? 'Yêu cầu quyền' : 'Request permissions'}</button>
            <button onClick={onRefresh}>{language === 'vi' ? 'Kiểm tra lại' : 'Check again'}</button>
            <button disabled={busy || !snapshot.runtime.driver?.accessibility || !snapshot.runtime.driver?.listen_access || !snapshot.runtime.driver?.post_access} onClick={() => void sendCommand('set_input_suspended',{target_session_id:snapshot.runtime.engine_session_id,suspended:false,reason:'user'}).then(result => {if(!result.ok)setNotice('Resume failed');onRefresh();})}>Resume</button>
          </section>
        )}

        {activeTab === 'updates' && (
          <UpdatesSettings language={language} version={snapshot.version}>
            <UpdateStatus language={language} update={update}
              onCheck={checkForUpdates} onInstall={installUpdate} onRestart={restartAfterUpdate} />
          </UpdatesSettings>
        )}

        {activeTab === 'about' && (
          <AboutSettings
            language={language}
            appName="VXKey"
            version={snapshot.version}
            tagline={t.tagline}
            description={t.description}
            repository="https://github.com/linhbv94/VXKey"
            icon={<Keyboard aria-hidden="true" />}
            technologies={['Rust Core Engine', 'CoreGraphics (macOS)', 'Tauri 2', 'React 19', 'TypeScript']}
          />
        )}

        {activeTab === 'support' && (
          <SupportSettings
            language={language}
            qrSource="/support_qr.png"
          />
        )}
      </SettingsShell>
      {update.noticeVisible && update.status !== 'disabled' && (
        <aside className="fixed bottom-4 right-4 z-50 w-80 rounded-xl border border-cyan-500/40 bg-zinc-900 p-4 shadow-xl" aria-live="polite">
          <button className="float-right" onClick={dismissUpdateNotice} aria-label={language === 'vi' ? 'Đóng' : 'Close'}>×</button>
          <UpdateStatus language={language} update={update} onCheck={checkForUpdates} onInstall={installUpdate} onRestart={restartAfterUpdate} />
        </aside>
      )}
    </div>
  );
}
