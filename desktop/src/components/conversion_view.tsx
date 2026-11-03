import { invoke } from '@tauri-apps/api/core';
import { useState } from 'react';
import type { Snapshot } from '../types';
import { uiCopy } from '../services/copy';
import { sendCommand } from '../services/api';
import {
  RefreshCw,
  ArrowRightLeft,
  Copy,
  Check,
  Trash2,
  X,
  FileText,
} from 'lucide-react';

interface Props {
  snapshot: Snapshot;
  onClose?: () => void;
}

export function ConversionView({ snapshot, onClose }: Props) {
  const language = (snapshot.config.ui.locale === 'en' ? 'en' : 'vi') as 'vi' | 'en';
  const t = uiCopy[language].conversion;

  const [fromCharset, setFromCharset] = useState('unicode');
  const [toCharset, setToCharset] = useState('unicode');
  const [transforms, setTransforms] = useState<string[]>([]);
  const [keepD, setKeepD] = useState(false);
  const [flattenCaps, setFlattenCaps] = useState(false);

  const [inputText, setInputText] = useState('');
  const [outputText, setOutputText] = useState('');
  const [stats, setStats] = useState<{ normalized: number; unmapped: number } | null>(null);
  const [copied, setCopied] = useState(false);
  const [busy, setBusy] = useState(false);
  const [errorNotice, setErrorNotice] = useState<string | null>(null);

  const charsets = [
    { id: 'unicode', label: t.unicode },
    { id: 'tcvn3', label: t.tcvn3 },
    { id: 'vni_windows', label: t.vniWindows },
    { id: 'unicode_combining', label: t.unicodeCombining },
  ];

  const transformOptions = [
    { id: 'upper', label: t.upper },
    { id: 'lower', label: t.lower },
    { id: 'no_diacritics', label: t.noDiacritics },
    { id: 'sentence', label: t.sentence },
    { id: 'title', label: t.titleCase },
  ];

  const toggleTransform = (id: string) => {
    if (transforms.includes(id)) {
      setTransforms(transforms.filter(item => item !== id));
    } else {
      if (transforms.length >= 5) return;
      setTransforms([...transforms, id]);
    }
  };

  const handleSwap = () => {
    const temp = fromCharset;
    setFromCharset(toCharset);
    setToCharset(temp);
  };

  const handleConvert = async () => {
    if (!inputText) return;
    setBusy(true);
    setErrorNotice(null);
    const res = await sendCommand<{ text: string; unmapped: number; normalized: number }>(
      'convert_text',
      {
        text: inputText,
        from_charset: fromCharset,
        to_charset: toCharset,
        transforms,
        keep_d: keepD,
        flatten_caps: flattenCaps,
      }
    );
    setBusy(false);
    if (res.ok && res.data) {
      setOutputText(res.data.text);
      setStats({
        normalized: res.data.normalized,
        unmapped: res.data.unmapped,
      });
    } else {
      setOutputText('');
      setStats(null);
      setErrorNotice(language === 'vi' ? 'Chuyển đổi thất bại (vượt hạn mức hoặc bảng mã không hỗ trợ).' : 'Conversion failed (quota exceeded or unsupported charset).');
    }
  };

  const handleCopy = async () => {
    if (!outputText) return;
    try {
      if ('__TAURI_INTERNALS__' in window) await invoke('vxkey_copy_text', {text: outputText});
      else await navigator.clipboard.writeText(outputText);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      setErrorNotice(language === 'vi' ? 'Không thể sao chép vào bộ nhớ tạm.' : 'Failed to copy to clipboard.');
    }
  };

  return (
    <div className="w-[920px] h-[620px] max-w-full max-h-full rounded-xl vx-surface flex flex-col shadow-2xl border border-black/10 dark:border-white/10 select-none overflow-hidden">
      {/* Header */}
      <div className="p-4 border-b border-black/10 dark:border-white/10 flex items-center justify-between bg-black/5 dark:bg-white/5">
        <div className="flex items-center gap-2.5">
          <FileText size={18} className="text-cyan-600 dark:text-cyan-400" />
          <h3 className="font-semibold text-sm tracking-tight">{t.title}</h3>
        </div>
        {onClose && (
          <button type="button" onClick={onClose} className="opacity-60 hover:opacity-100 p-1">
            <X size={16} />
          </button>
        )}
      </div>

      {errorNotice && (
        <div className="px-4 py-1.5 bg-rose-500/10 text-rose-500 dark:text-rose-400 text-xs border-b border-rose-500/20">
          {errorNotice}
        </div>
      )}

      {/* Settings Bar: Charsets & Transforms */}
      <div className="p-4 border-b border-black/10 dark:border-white/10 space-y-4 bg-black/5 dark:bg-black/10">
        {/* Charset selector */}
        <div className="flex flex-wrap items-center gap-3">
          <div className="flex-1 min-w-[200px] space-y-1">
            <label className="text-[11px] font-medium opacity-60 uppercase tracking-wider block">
              {t.sourceCharset}
            </label>
            <select
              value={fromCharset}
              onChange={e => setFromCharset(e.target.value)}
              className="w-full bg-white dark:bg-black/40 border border-black/15 dark:border-white/15 rounded-lg px-2.5 py-1.5 text-xs text-slate-900 dark:text-white focus:outline-none focus:border-cyan-500"
            >
              {charsets.map(c => (
                <option key={c.id} value={c.id} className="bg-white dark:bg-[#141821] text-slate-900 dark:text-white">
                  {c.label}
                </option>
              ))}
            </select>
          </div>

          <button
            type="button"
            onClick={handleSwap}
            className="p-2 mt-4 rounded-lg border border-black/10 dark:border-white/10 hover:border-cyan-500/30 bg-black/5 dark:bg-white/5 transition-colors"
            title={t.swap}
          >
            <ArrowRightLeft size={14} className="text-cyan-600 dark:text-cyan-400" />
          </button>

          <div className="flex-1 min-w-[200px] space-y-1">
            <label className="text-[11px] font-medium opacity-60 uppercase tracking-wider block">
              {t.targetCharset}
            </label>
            <select
              value={toCharset}
              onChange={e => setToCharset(e.target.value)}
              className="w-full bg-white dark:bg-black/40 border border-black/15 dark:border-white/15 rounded-lg px-2.5 py-1.5 text-xs text-slate-900 dark:text-white focus:outline-none focus:border-cyan-500"
            >
              {charsets.map(c => (
                <option key={c.id} value={c.id} className="bg-white dark:bg-[#141821] text-slate-900 dark:text-white">
                  {c.label}
                </option>
              ))}
            </select>
          </div>
        </div>

        {/* Transform Chips & Options */}
        <div className="space-y-2">
          <label className="text-[11px] font-medium opacity-60 uppercase tracking-wider block">
            {t.transforms}
          </label>
          <div className="flex flex-wrap items-center gap-2">
            {transformOptions.map(opt => {
              const active = transforms.includes(opt.id);
              return (
                <button
                  key={opt.id}
                  type="button"
                  onClick={() => toggleTransform(opt.id)}
                  className={`px-3 py-1 rounded-full text-xs font-medium border transition-all ${
                    active
                      ? 'border-cyan-500 bg-cyan-500/20 text-cyan-600 dark:text-cyan-300'
                      : 'border-black/10 dark:border-white/10 hover:border-cyan-500/30 bg-black/5 dark:bg-white/5 text-slate-700 dark:text-white/70'
                  }`}
                >
                  {opt.label}
                </button>
              );
            })}
          </div>

          <div className="flex flex-wrap items-center gap-4 pt-1">
            <label className="flex items-center gap-2 text-xs cursor-pointer opacity-80 hover:opacity-100">
              <input
                type="checkbox"
                checked={keepD}
                onChange={e => setKeepD(e.target.checked)}
                className="accent-cyan-500 rounded"
              />
              <span>{t.keepD}</span>
            </label>

            <label className="flex items-center gap-2 text-xs cursor-pointer opacity-80 hover:opacity-100">
              <input
                type="checkbox"
                checked={flattenCaps}
                onChange={e => setFlattenCaps(e.target.checked)}
                className="accent-cyan-500 rounded"
              />
              <span>{t.flattenCaps}</span>
            </label>
          </div>
        </div>
      </div>

      {/* Editor Main: Input vs Output */}
      <div className="flex-1 grid grid-cols-1 md:grid-cols-2 divide-y md:divide-y-0 md:divide-x divide-black/10 dark:divide-white/10 min-h-0 bg-black/5 dark:bg-black/20">
        {/* Input */}
        <div className="flex flex-col p-3 min-h-0">
          <div className="flex items-center justify-between pb-2 text-[11px] opacity-60">
            <span>{language === 'vi' ? 'Văn bản gốc' : 'Original Text'}</span>
            <button
              type="button"
              onClick={() => {
                setInputText('');
                setOutputText('');
                setStats(null);
                setErrorNotice(null);
              }}
              className="text-rose-500 hover:text-rose-400 flex items-center gap-1"
            >
              <Trash2 size={12} />
              <span>{t.clear}</span>
            </button>
          </div>
          <textarea
            placeholder={t.inputPlaceholder}
            value={inputText}
            onChange={e => setInputText(e.target.value)}
            className="flex-1 w-full p-3 rounded-lg bg-white dark:bg-black/40 border border-black/5 dark:border-white/5 text-xs text-slate-900 dark:text-white placeholder-slate-400 dark:placeholder-white/30 focus:outline-none focus:border-cyan-500 resize-none font-mono leading-relaxed"
          />
        </div>

        {/* Output */}
        <div className="flex flex-col p-3 min-h-0">
          <div className="flex items-center justify-between pb-2 text-[11px] opacity-60">
            <span>{language === 'vi' ? 'Văn bản đã chuyển đổi' : 'Converted Text'}</span>
            {stats && (
              <span className="text-cyan-600 dark:text-cyan-400 text-[10px] font-mono">
                {t.stats
                  .replace('{normalized}', String(stats.normalized))
                  .replace('{unmapped}', String(stats.unmapped))}
              </span>
            )}
          </div>
          <textarea
            readOnly
            placeholder={t.outputPlaceholder}
            value={outputText}
            className="flex-1 w-full p-3 rounded-lg bg-black/5 dark:bg-black/30 border border-black/5 dark:border-white/5 text-xs text-cyan-700 dark:text-cyan-200 placeholder-slate-400 dark:placeholder-white/30 focus:outline-none resize-none font-mono leading-relaxed"
          />
        </div>
      </div>

      {/* Footer Actions */}
      <div className="p-3 border-t border-black/10 dark:border-white/10 flex items-center justify-between bg-black/5 dark:bg-white/5 text-xs">
        <span className="opacity-50 text-[11px] font-mono">
          {inputText.length} ký tự
        </span>

        <div className="flex items-center gap-2">
          <button
            type="button"
            disabled={!outputText}
            onClick={handleCopy}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-black/10 dark:border-white/10 hover:border-cyan-500/30 bg-black/5 dark:bg-white/5 text-slate-800 dark:text-white disabled:opacity-40 text-xs font-medium transition-colors"
          >
            {copied ? <Check size={14} className="text-emerald-500" /> : <Copy size={14} />}
            <span>{copied ? (language === 'vi' ? 'Đã sao chép!' : 'Copied!') : t.copy}</span>
          </button>

          <button
            type="button"
            disabled={busy || !inputText.trim()}
            onClick={handleConvert}
            className="flex items-center gap-1.5 bg-cyan-600 hover:bg-cyan-500 text-white disabled:opacity-40 px-4 py-1.5 rounded-lg text-xs font-medium shadow-md transition-all"
          >
            <RefreshCw size={14} className={busy ? 'animate-spin' : ''} />
            <span>{t.convert}</span>
          </button>
        </div>
      </div>
    </div>
  );
}
