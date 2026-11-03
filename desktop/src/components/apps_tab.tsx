import React, { useState } from 'react';
import type { Config } from '../types';
import { uiCopy } from '../services/copy';
import { Trash2, Plus, Shield } from 'lucide-react';

interface Props {
  config: Config;
  language: 'vi' | 'en';
  disabled?: boolean;
  onRememberChange: (remember: boolean) => void;
  onForgetLanguage: (appId: string) => void;
  onSetPolicy: (appId: string, inputBypass: boolean, clipboardBlocked: boolean) => void;
  onRemovePolicy: (appId: string) => void;
}

export function AppsTab({
  config,
  language,
  disabled = false,
  onRememberChange,
  onForgetLanguage,
  onSetPolicy,
  onRemovePolicy,
}: Props) {
  const t = uiCopy[language].apps;
  const { remember_per_app, language_memory, policies } = config.apps;

  const [newAppId, setNewAppId] = useState('');
  const [newBypass, setNewBypass] = useState(true);
  const [newClipBlock, setNewClipBlock] = useState(false);

  const handleAddPolicy = (e: React.FormEvent) => {
    e.preventDefault();
    const id = newAppId.trim();
    if (!id) return;
    onSetPolicy(id, newBypass, newClipBlock);
    setNewAppId('');
    setNewBypass(true);
    setNewClipBlock(false);
  };

  const memoryKeys = Object.keys(language_memory);
  const policyKeys = Object.keys(policies);

  return (
    <div className="vx_business_tab">
      {/* Ghi nhớ chế độ theo app */}
      <section className="space-y-3">
        <h4 className="font-semibold text-xs tracking-wider opacity-60 uppercase">{t.title}</h4>

        <label className="flex items-start gap-2.5 cursor-pointer p-3 rounded-lg border border-white/10 bg-white/5">
          <input
            type="checkbox"
            checked={remember_per_app}
            disabled={disabled}
            onChange={e => onRememberChange(e.target.checked)}
            className="accent-cyan-500 rounded mt-0.5"
          />
          <div>
            <div className="text-xs font-medium">{t.rememberTitle}</div>
            <p className="text-[11px] opacity-70 mt-0.5">{t.rememberDesc}</p>
          </div>
        </label>
      </section>

      {/* Danh sách ứng dụng đã nhớ ngôn ngữ */}
      <section className="space-y-3">
        <h4 className="font-semibold text-xs tracking-wider opacity-60 uppercase">{t.memoryListTitle}</h4>

        {memoryKeys.length === 0 ? (
          <p className="text-xs opacity-50 italic py-2">{t.emptyMemory}</p>
        ) : (
          <div className="border border-white/10 rounded-lg overflow-hidden divide-y divide-white/5 bg-white/5">
            {memoryKeys.map(appId => (
              <div key={appId} className="vx_policy_row">
                <span className="font-mono opacity-90 truncate max-w-[280px]" title={appId}>
                  {appId}
                </span>
                <div className="vx_policy_actions">
                  <span className="px-2 py-0.5 rounded text-[11px] font-medium bg-cyan-500/10 text-cyan-400 border border-cyan-500/20">
                    {language_memory[appId] === 'vi' ? 'Tiếng Việt' : 'English'}
                  </span>
                  <button
                    type="button"
                    disabled={disabled}
                    onClick={() => onForgetLanguage(appId)}
                    className="text-xs text-rose-400 hover:text-rose-300 transition-colors"
                  >
                    {t.forget}
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </section>

      <hr className="border-white/10" />

      {/* Quy tắc loại trừ ứng dụng */}
      <section className="space-y-3">
        <h4 className="font-semibold text-xs tracking-wider opacity-60 uppercase">{t.policiesTitle}</h4>

        {/* Form thêm quy tắc */}
        <form onSubmit={handleAddPolicy} className="vx_policy_form">
          <input
            type="text"
            placeholder="com.example.app"
            value={newAppId}
            disabled={disabled}
            onChange={e => setNewAppId(e.target.value)}
            className="vx_policy_app_input" autoCapitalize="off" autoCorrect="off" spellCheck={false}
          />
          <label className="flex items-center gap-1.5 text-xs cursor-pointer">
            <input
              type="checkbox"
              checked={newBypass}
              disabled={disabled}
              onChange={e => setNewBypass(e.target.checked)}
              className="accent-cyan-500 rounded"
            />
            <span>{t.bypass}</span>
          </label>
          <label className="flex items-center gap-1.5 text-xs cursor-pointer">
            <input
              type="checkbox"
              checked={newClipBlock}
              disabled={disabled}
              onChange={e => setNewClipBlock(e.target.checked)}
              className="accent-cyan-500 rounded"
            />
            <span>{t.blockClipboard}</span>
          </label>
          <button
            type="submit"
            disabled={disabled || !newAppId.trim()}
            className="flex items-center gap-1 bg-cyan-600 hover:bg-cyan-500 disabled:opacity-40 text-white px-3 py-1.5 rounded text-xs font-medium transition-colors"
          >
            <Plus size={14} />
            <span>{t.addPolicy}</span>
          </button>
        </form>

        {/* Danh sách policies */}
        {policyKeys.length === 0 ? (
          <p className="text-xs opacity-50 italic py-2">{t.emptyPolicies}</p>
        ) : (
          <div className="border border-white/10 rounded-lg overflow-hidden divide-y divide-white/5 bg-white/5">
            {policyKeys.map(appId => {
              const pol = policies[appId];
              return (
                <div key={appId} className="vx_policy_row">
                  <div className="vx_policy_identity" title={appId}>
                    <Shield size={14} className="text-cyan-400 shrink-0" />
                    <span className="font-mono opacity-90 truncate">{appId}</span>
                  </div>
                  <div className="vx_policy_actions">
                    <label className="flex items-center gap-1 text-[11px] cursor-pointer">
                      <input
                        type="checkbox"
                        checked={pol.input_bypass}
                        disabled={disabled}
                        onChange={e => onSetPolicy(appId, e.target.checked, pol.clipboard_blocked)}
                        className="accent-cyan-500 rounded"
                      />
                      <span>{t.bypass}</span>
                    </label>
                    <label className="flex items-center gap-1 text-[11px] cursor-pointer">
                      <input
                        type="checkbox"
                        checked={pol.clipboard_blocked}
                        disabled={disabled}
                        onChange={e => onSetPolicy(appId, pol.input_bypass, e.target.checked)}
                        className="accent-cyan-500 rounded"
                      />
                      <span>{t.blockClipboard}</span>
                    </label>
                    <button
                      type="button"
                      disabled={disabled}
                      onClick={() => onRemovePolicy(appId)}
                      className="text-rose-400 hover:text-rose-300 p-1"
                      title={t.actions}
                    >
                      <Trash2 size={14} />
                    </button>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </section>
    </div>
  );
}
