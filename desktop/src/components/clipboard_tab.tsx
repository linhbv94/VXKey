import type { Config } from '../types';
import { uiCopy } from '../services/copy';
import { ShieldCheck } from 'lucide-react';

interface Props {
  config: Config;
  language: 'vi' | 'en';
  disabled?: boolean;
  onChange: (patch: Partial<Config['clipboard']>) => void;
}

export function ClipboardTab({ config, language, disabled = false, onChange }: Props) {
  const t = uiCopy[language].clipboard;
  const clip = config.clipboard;

  return (
    <div className="vx_business_tab">
      <section className="space-y-3">
        <h4 className="font-semibold text-xs tracking-wider opacity-60 uppercase">{t.title}</h4>

        {/* Thông tin chế độ lưu trữ */}
        <div className="flex items-start gap-2.5 p-3 rounded-lg border border-cyan-500/20 bg-cyan-500/10 text-cyan-300">
          <ShieldCheck size={18} className="shrink-0 mt-0.5 text-cyan-400" />
          <p className="text-xs leading-relaxed">{t.storageMode}</p>
        </div>

        {/* Số lượng mục tối đa */}
        <div className="p-3.5 rounded-lg border border-white/10 bg-white/5 space-y-2">
          <div className="flex items-center justify-between">
            <label className="text-xs font-medium">{t.maxEntries}</label>
            <span className="font-mono text-xs font-semibold text-cyan-400">{clip.max_entries}</span>
          </div>
          <input
            type="range"
            min={1}
            max={200}
            step={1}
            value={clip.max_entries}
            disabled={disabled}
            onChange={e => onChange({ max_entries: Number(e.target.value) })}
            className="w-full accent-cyan-500 cursor-pointer"
          />
        </div>

        {/* TTL */}
        <div className="p-3.5 rounded-lg border border-white/10 bg-white/5 flex items-center justify-between">
          <label className="text-xs font-medium">{t.ttlDays}</label>
          <input
            type="number"
            min={1}
            max={7}
            value={clip.ttl_days}
            disabled={disabled}
            onChange={e => onChange({ ttl_days: Math.max(1, Math.min(7, Number(e.target.value))) })}
            className="w-16 bg-black/40 border border-white/15 rounded px-2 py-1 text-xs text-right font-mono text-cyan-400 focus:outline-none focus:border-cyan-500"
          />
        </div>

        <p className="text-[11px] opacity-60 pt-1 leading-relaxed">{t.note}</p>
      </section>
    </div>
  );
}
