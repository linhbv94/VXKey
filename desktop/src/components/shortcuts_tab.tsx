import type { Config, Snapshot, ShortcutBinding } from '../types';
import { uiCopy } from '../services/copy';
import { Info } from 'lucide-react';

interface Props {
  config: Config;
  snapshot: Snapshot;
  language: 'vi' | 'en';
  disabled?: boolean;
}

export function ShortcutsTab({ config, snapshot, language }: Props) {
  const t = uiCopy[language].shortcuts;
  const isMac = snapshot.platform === 'macos';
  const binding = (shortcut: ShortcutBinding | null) => {
    if (!shortcut) return language === 'vi' ? 'Chưa gán phím' : 'Unassigned';
    const names: Record<string,string> = isMac ? {control:'⌃',alt:'⌥',shift:'⇧',meta:'⌘'} : {control:'Ctrl',alt:'Alt',shift:'Shift',meta:'Win'};
    const keys = shortcut.modifiers.map(value => names[value] || value);
    if (shortcut.kind === 'key_chord') keys.push(shortcut.key_code.replace(/^Key|^Digit/,''));
    return keys.join(isMac ? '' : ' + ');
  };

  return (
    <div className="vx_business_tab">
      <section className="space-y-3">
        <h4 className="font-semibold text-xs tracking-wider opacity-60 uppercase">{t.title}</h4>

        {/* Chuyển đổi Việt / Anh */}
        <div className="p-3.5 rounded-lg border border-white/10 bg-white/5 space-y-1.5">
          <div className="flex items-center justify-between">
            <span className="text-xs font-medium">{t.toggleMode}</span>
            <kbd className="px-2.5 py-1 rounded bg-black/40 border border-white/20 font-mono text-xs text-cyan-400 font-semibold shadow-sm">
              {binding(config.shortcuts.toggle)}
            </kbd>
          </div>
          <p className="text-[11px] opacity-70">
            {isMac ? t.toggleMacHint : t.toggleWinHint}
          </p>
        </div>

        {/* Lật từ vừa gõ */}
        <div className="p-3.5 rounded-lg border border-white/10 bg-white/5 space-y-1.5">
          <div className="flex items-center justify-between">
            <span className="text-xs font-medium">{t.flipWord}</span>
            <span className="text-[11px] opacity-50 italic">
              {binding(config.shortcuts.flip)}
            </span>
          </div>
          <p className="text-[11px] opacity-70">{t.flipHint}</p>
        </div>

        {/* Mở lịch sử sao chép */}
        <div className="p-3.5 rounded-lg border border-white/10 bg-white/5 space-y-1.5">
          <div className="flex items-center justify-between">
            <span className="text-xs font-medium">{t.history}</span>
            <kbd className="px-2.5 py-1 rounded bg-black/40 border border-white/20 font-mono text-xs text-cyan-400 font-semibold shadow-sm">
              {binding(config.shortcuts.history)}
            </kbd>
          </div>
          <p className="text-[11px] opacity-70">
            {isMac ? t.historyMacHint : t.historyWinHint}
          </p>
        </div>
      </section>

      {/* Thông báo trạng thái capability */}
      <div className="flex items-start gap-2.5 p-3 rounded-lg border border-cyan-500/20 bg-cyan-500/5 text-cyan-400 text-xs">
        <Info size={16} className="shrink-0 mt-0.5" />
        <p className="leading-relaxed opacity-90">{t.unsupportedNotice}</p>
      </div>
    </div>
  );
}
