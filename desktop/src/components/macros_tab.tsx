import { useState, useEffect, useRef, useImperativeHandle, type Ref } from 'react';
import type { Config, MacroEntry } from '../types';
import { uiCopy } from '../services/copy';
import { sendCommand } from '../services/api';
import { loadAllMacros } from '../services/macro_loader';
import { Trash2, Plus, Save, RotateCcw } from 'lucide-react';

export interface MacroDraftHandle { dirty: boolean; save: () => Promise<boolean>; discard: () => void; }

interface Props {
  config: Config;
  language: 'vi' | 'en';
  disabled?: boolean;
  onRefresh: () => void;
  onNotice: (msg: string) => void;
  draftRef?: Ref<MacroDraftHandle>;
}

export function MacrosTab({ config, language, disabled: externalDisabled = false, onRefresh, onNotice, draftRef }: Props) {
  const t = uiCopy[language].macros;
  const m = config.macros;

  // Local draft state
  const [enabled, setEnabled] = useState(m.enabled);
  const [inEn, setInEn] = useState(m.in_en);
  const [smartCase, setSmartCase] = useState(m.smart_case);
  const [entries, setEntries] = useState<MacroEntry[]>([]);
  const [initialEntries, setInitialEntries] = useState<MacroEntry[]>([]);
  const [dirty, setDirty] = useState(false);
  const [loading, setLoading] = useState(true);

  const [newTrigger, setNewTrigger] = useState('');
  const [newReplacement, setNewReplacement] = useState('');
  const [error, setError] = useState<string | null>(null);

  const [saving, setSaving] = useState(false);
  const [loadedRevision, setLoadedRevision] = useState<string | null>(null);
  const generation = useRef(0);
  const dirtyRef = useRef(dirty);
  dirtyRef.current = dirty;
  const disabled = externalDisabled || loading || saving || loadedRevision === null;

  const loadMacros = async () => {
    const gen = ++generation.current;
    setLoading(true);
    setLoadedRevision(null);
    try {
      const all = await loadAllMacros(config.revision, (offset, limit) =>
        sendCommand('list_macros', {expected_revision: config.revision, offset, limit}));
      if (gen !== generation.current) return;
      setEntries(all);
      setInitialEntries(all);
      setEnabled(m.enabled); setInEn(m.in_en); setSmartCase(m.smart_case);
      setLoadedRevision(config.revision);
      setDirty(false); setError(null);
    } catch {
      if (gen === generation.current) setError(language === 'vi'
        ? 'Chưa tải đủ từ điển. Không thể lưu; hãy thử tải lại.'
        : 'The complete dictionary could not be loaded. Reload before saving.');
    } finally {
      if (gen === generation.current) setLoading(false);
    }
  };

  useEffect(() => {
    if (dirtyRef.current) {
      setError(language === 'vi' ? 'Cấu hình đã đổi. Nháp được giữ lại; hủy nháp để tải bản mới.' : 'Configuration changed. Draft preserved; cancel to reload.');
      return;
    }
    void loadMacros();
    return () => { generation.current += 1; };
  }, [config.revision]);

  const handleAdd = (e: React.FormEvent) => {
    e.preventDefault();
    const trigger = newTrigger.trim();
    const replacement = newReplacement;
    if (disabled || !trigger || !replacement) return;

    if (/\s/.test(trigger)) {
      setError(language === 'vi' ? 'Từ viết tắt không được chứa khoảng trắng.' : 'Trigger must not contain whitespace.');
      return;
    }
    if ([...trigger].length > 32 || entries.length >= 1000 || new TextEncoder().encode(replacement).length > 8192) {
      setError(language === 'vi' ? 'Vượt giới hạn: 1.000 mục, trigger 32 ký tự, nội dung 8 KiB.' : 'Limit: 1,000 entries, 32-character trigger, 8 KiB replacement.');
      return;
    }
    if (entries.some(item => smartCase ? item.trigger.toLowerCase() === trigger.toLowerCase() : item.trigger === trigger)) {
      setError(language === 'vi' ? 'Từ viết tắt đã tồn tại trong bảng.' : 'Trigger already exists.');
      return;
    }

    const updated = [
      ...entries,
      {
        id: crypto.randomUUID(),
        trigger,
        replacement,
      },
    ];
    setEntries(updated);
    setNewTrigger('');
    setNewReplacement('');
    setError(null);
    setDirty(true);
  };

  const handleDelete = (id: string) => {
    setEntries(entries.filter(item => item.id !== id));
    setDirty(true);
  };

  const handleSave = async () => {
    if (disabled || !loadedRevision) return false;
    setSaving(true);
    const res = await sendCommand('replace_macros', {
      expected_revision: loadedRevision,
      enabled,
      in_en: inEn,
      smart_case: smartCase,
      entries,
    });
    setSaving(false);
    if (res.ok) {
      dirtyRef.current = false;
      setDirty(false);
      onNotice(t.saveNotice);
      onRefresh();
      return true;
    } else {
      if (res.error?.code === 'revision_conflict') {
        onNotice(language === 'vi' ? 'Xung đột dữ liệu. Nháp được giữ lại; hủy nháp để tải bản mới.' : 'Revision conflict. Draft preserved; cancel to reload.');
        onRefresh();
      } else {
        onNotice(language === 'vi' ? 'Lưu gõ tắt thất bại.' : 'Failed to save macros.');
      }
    }
    return false;
  };

  const handleCancel = () => {
    setEnabled(m.enabled);
    setInEn(m.in_en);
    setSmartCase(m.smart_case);
    setEntries(initialEntries);
    setDirty(false);
    setError(null);
    dirtyRef.current = false;
    void loadMacros();
  };

  useImperativeHandle(draftRef, () => ({dirty, save: handleSave, discard: handleCancel}));

  return (
    <div className="vx_business_tab">
      {error && <div role="alert" className="text-xs text-rose-500">{error}
        {!dirty && <button type="button" onClick={() => void loadMacros()} disabled={loading} className="ml-2 underline">{language === 'vi' ? 'Tải lại' : 'Reload'}</button>}
      </div>}
      <section className="space-y-3">
        <div className="vx_macro_toolbar">
          <h4 className="font-semibold text-xs tracking-wider opacity-60 uppercase">{t.title}</h4>
          {dirty && (
            <div className="vx_macro_actions">
              <button
                type="button"
                onClick={handleCancel}
                className="flex items-center gap-1 text-xs px-2.5 py-1 rounded border border-black/10 dark:border-white/10 hover:bg-black/5 dark:hover:bg-white/5 transition-colors opacity-80"
              >
                <RotateCcw size={12} />
                <span>{language === 'vi' ? 'Hủy nháp' : 'Cancel'}</span>
              </button>
              <button
                type="button"
                onClick={handleSave}
                disabled={disabled}
                className="flex items-center gap-1 text-xs font-medium px-3 py-1 rounded bg-cyan-600 hover:bg-cyan-500 text-white shadow-sm transition-colors"
              >
                <Save size={12} />
                <span>{language === 'vi' ? 'Lưu thay đổi' : 'Save'}</span>
              </button>
            </div>
          )}
        </div>

        <div className="space-y-2.5">
          <label className="flex items-center gap-2.5 cursor-pointer">
            <input
              type="checkbox"
              checked={enabled}
              disabled={disabled}
              onChange={e => {
                setEnabled(e.target.checked);
                setDirty(true);
              }}
              className="accent-cyan-500 rounded"
            />
            <span className="text-xs font-medium">{t.enabled}</span>
          </label>

          <label className={`flex items-center gap-2.5 cursor-pointer pl-6 ${!enabled ? 'opacity-40 pointer-events-none' : ''}`}>
            <input
              type="checkbox"
              checked={inEn}
              disabled={disabled || !enabled}
              onChange={e => {
                setInEn(e.target.checked);
                setDirty(true);
              }}
              className="accent-cyan-500 rounded"
            />
            <span className="text-xs">{t.inEn}</span>
          </label>

          <label className={`flex items-center gap-2.5 cursor-pointer pl-6 ${!enabled ? 'opacity-40 pointer-events-none' : ''}`}>
            <input
              type="checkbox"
              checked={smartCase}
              disabled={disabled || !enabled}
              onChange={e => {
                setSmartCase(e.target.checked);
                setDirty(true);
              }}
              className="accent-cyan-500 rounded"
            />
            <span className="text-xs">{t.smartCase}</span>
          </label>
        </div>
      </section>

      <hr className="border-black/10 dark:border-white/10" />

      {/* Danh sách & Thêm gõ tắt */}
      <section className="space-y-3">
        <h4 className="font-semibold text-xs tracking-wider opacity-60 uppercase">
          {language === 'vi' ? 'DANH SÁCH TỪ GÕ TẮT' : 'MACRO DICTIONARY'} ({entries.length})
        </h4>

        {/* Form thêm gõ tắt */}
        <form onSubmit={handleAdd} className="space-y-2 p-3 rounded-lg border border-black/10 dark:border-white/10 bg-black/5 dark:bg-white/5">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
            <div>
              <input
                type="text"
                autoCapitalize="off" autoCorrect="off" spellCheck={false}
                placeholder={t.trigger}
                value={newTrigger}
                disabled={disabled || !enabled}
                onChange={e => setNewTrigger(e.target.value)}
                className="w-full bg-white dark:bg-black/30 border border-black/15 dark:border-white/15 rounded px-2.5 py-1.5 text-xs text-slate-900 dark:text-white placeholder-slate-400 dark:placeholder-white/40 focus:outline-none focus:border-cyan-500"
              />
            </div>
            <div>
              <input
                type="text"
                autoCapitalize="off" autoCorrect="off" spellCheck={false}
                placeholder={t.replacement}
                value={newReplacement}
                disabled={disabled || !enabled}
                onChange={e => setNewReplacement(e.target.value)}
                className="w-full bg-white dark:bg-black/30 border border-black/15 dark:border-white/15 rounded px-2.5 py-1.5 text-xs text-slate-900 dark:text-white placeholder-slate-400 dark:placeholder-white/40 focus:outline-none focus:border-cyan-500"
              />
            </div>
          </div>
          {error && <p className="text-[11px] text-rose-500 dark:text-rose-400">{error}</p>}
          <div className="flex justify-end">
            <button
              type="submit"
              disabled={disabled || !enabled || !newTrigger.trim() || !newReplacement.trim()}
              className="flex items-center gap-1 bg-cyan-600 hover:bg-cyan-500 disabled:opacity-40 text-white px-3 py-1.5 rounded text-xs font-medium transition-colors"
            >
              <Plus size={14} />
              <span>{t.addMacro}</span>
            </button>
          </div>
        </form>

        {/* Bảng danh sách gõ tắt */}
        {loading ? (
          <p className="text-xs opacity-50 italic py-2">{language === 'vi' ? 'Đang tải danh sách gõ tắt…' : 'Loading macros…'}</p>
        ) : entries.length === 0 ? (
          <p className="text-xs opacity-50 italic py-2">{t.empty}</p>
        ) : (
          <div className="border border-black/10 dark:border-white/10 rounded-lg overflow-hidden divide-y divide-black/5 dark:divide-white/5 bg-black/5 dark:bg-white/5 max-h-[220px] overflow-y-auto">
            {entries.map(item => (
              <div key={item.id} className="flex items-center justify-between px-3 py-2 text-xs">
                <div className="flex items-center gap-3">
                  <span className="font-mono font-semibold text-cyan-600 dark:text-cyan-400 bg-cyan-500/10 px-2 py-0.5 rounded border border-cyan-500/20">
                    {item.trigger}
                  </span>
                  <span className="opacity-40">→</span>
                  <span className="opacity-90 max-w-[280px] truncate" title={item.replacement}>
                    {item.replacement}
                  </span>
                </div>
                <button
                  type="button"
                  disabled={disabled}
                  onClick={() => handleDelete(item.id)}
                  className="text-rose-500 hover:text-rose-400 p-1"
                  title={t.delete}
                >
                  <Trash2 size={14} />
                </button>
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
