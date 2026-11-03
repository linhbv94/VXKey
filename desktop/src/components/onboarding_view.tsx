import type { Snapshot } from '../types';
import { uiCopy } from '../services/copy';
import { Sparkles, Keyboard, ShieldCheck, ArrowRight, X } from 'lucide-react';

interface Props {
  snapshot: Snapshot;
  onFinish?: () => void;
  onClose?: () => void;
}

export function OnboardingView({ snapshot, onFinish, onClose }: Props) {
  const language = (snapshot.config.ui.locale === 'en' ? 'en' : 'vi') as 'vi' | 'en';
  const t = uiCopy[language].onboarding;

  return (
    <div className="w-[560px] h-[520px] max-w-full max-h-full rounded-2xl vx-surface flex flex-col shadow-2xl border border-white/10 select-none overflow-hidden text-xs">
      {/* Top Banner */}
      <div className="p-6 text-center border-b border-white/10 bg-gradient-to-b from-cyan-500/10 to-transparent relative">
        {onClose && (
          <button
            type="button"
            onClick={onClose}
            className="absolute top-4 right-4 text-white/50 hover:text-white p-1"
          >
            <X size={16} />
          </button>
        )}
        <div className="w-14 h-14 mx-auto mb-3 rounded-2xl bg-black/40 border border-white/15 p-2 shadow-xl flex items-center justify-center">
          <img src="/app_icon.png" alt="VXKey" className="w-full h-full rounded-xl object-contain" />
        </div>
        <h2 className="text-lg font-bold tracking-tight text-white">{t.title}</h2>
        <p className="text-xs text-white/70 max-w-sm mx-auto mt-1 leading-relaxed">
          {t.subtitle}
        </p>
      </div>

      {/* Steps List */}
      <div className="flex-1 overflow-y-auto p-6 space-y-4">
        {/* Step 1 */}
        <div className="flex items-start gap-3.5 p-3 rounded-xl border border-white/10 bg-white/5">
          <div className="p-2 rounded-lg bg-cyan-500/10 text-cyan-400 shrink-0">
            <Sparkles size={18} />
          </div>
          <div>
            <h4 className="font-semibold text-white">{t.step1Title}</h4>
            <p className="text-[11px] text-white/70 mt-1 leading-relaxed">{t.step1Desc}</p>
          </div>
        </div>

        {/* Step 2 */}
        <div className="flex items-start gap-3.5 p-3 rounded-xl border border-white/10 bg-white/5">
          <div className="p-2 rounded-lg bg-cyan-500/10 text-cyan-400 shrink-0">
            <Keyboard size={18} />
          </div>
          <div>
            <h4 className="font-semibold text-white">{t.step2Title}</h4>
            <p className="text-[11px] text-white/70 mt-1 leading-relaxed">{t.step2Desc}</p>
          </div>
        </div>

        {/* Step 3 */}
        <div className="flex items-start gap-3.5 p-3 rounded-xl border border-white/10 bg-white/5">
          <div className="p-2 rounded-lg bg-cyan-500/10 text-cyan-400 shrink-0">
            <ShieldCheck size={18} />
          </div>
          <div>
            <h4 className="font-semibold text-white">{t.step3Title}</h4>
            <p className="text-[11px] text-white/70 mt-1 leading-relaxed">{t.step3Desc}</p>
          </div>
        </div>
      </div>

      {/* Footer Actions */}
      <div className="p-4 border-t border-white/10 bg-white/5 flex items-center justify-between">
        <button
          type="button"
          onClick={onClose}
          className="px-4 py-2 rounded-lg text-white/60 hover:text-white transition-colors text-xs font-medium"
        >
          {t.close}
        </button>

        <button
          type="button"
          onClick={onFinish}
          className="flex items-center gap-1.5 px-5 py-2 rounded-lg bg-cyan-600 hover:bg-cyan-500 text-white font-medium shadow-md transition-all text-xs"
        >
          <span>{t.getStarted}</span>
          <ArrowRight size={14} />
        </button>
      </div>
    </div>
  );
}
