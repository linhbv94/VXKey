import type { Config, Runtime } from '../types';
import { uiCopy } from '../services/copy';

interface Props {
  config: Config;
  runtime: Runtime;
  language: 'vi' | 'en';
  disabled?: boolean;
  onChange: (patch: Partial<Config['typing']>) => void;
}

export function TypingTab({ config, runtime, language, disabled = false, onChange }: Props) {
  const t = uiCopy[language].typing;
  const typing = config.typing;
  const period = runtime.period_substitution;
  const onsets = Array.isArray(typing.extra_onsets) ? typing.extra_onsets : [];

  const handleExtraMaster = (checked: boolean) => {
    onChange({
      extra_onsets: checked ? ['z', 'f', 'w', 'j'] : [],
    });
  };

  const handleExtraLetter = (letter: string, checked: boolean) => {
    let next: string[];
    if (checked) {
      next = Array.from(new Set([...onsets, letter]));
    } else {
      next = onsets.filter(l => l !== letter);
    }
    onChange({
      extra_onsets: next,
    });
  };

  return (
    <div className="vx_business_tab">
      {/* Kiểu gõ & Bỏ dấu */}
      <section className="space-y-3">
        <h4 className="font-semibold text-xs tracking-wider opacity-60 uppercase">{t.title}</h4>

        <div className="space-y-2">
          <label className="block text-xs font-medium opacity-80">{t.inputMethod}</label>
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-2">
            {[
              { id: 'telex', label: t.telex },
              { id: 'telex_advanced', label: t.telexAdvanced },
              { id: 'vni', label: t.vni },
            ].map(item => (
              <label
                key={item.id}
                className={`flex items-center gap-2 p-3 rounded-lg border cursor-pointer transition-all ${
                  typing.input_method === item.id
                    ? 'border-cyan-500 bg-cyan-500/10 font-medium'
                    : 'border-black/10 dark:border-white/10 hover:border-cyan-500/30 bg-black/5 dark:bg-white/5'
                } ${disabled ? 'opacity-50 pointer-events-none' : ''}`}
              >
                <input
                  type="radio"
                  name="input_method"
                  value={item.id}
                  checked={typing.input_method === item.id}
                  disabled={disabled}
                  onChange={() => onChange({ input_method: item.id })}
                  className="accent-cyan-500"
                />
                <span className="text-xs">{item.label}</span>
              </label>
            ))}
          </div>
        </div>

        {/* Tone placement */}
        <div className="space-y-2 pt-2">
          <label className="block text-xs font-medium opacity-80">{t.toneStyle}</label>
          <div className="grid grid-cols-2 gap-2">
            {[
              { id: 'modern', label: t.toneModern },
              { id: 'traditional', label: t.toneTraditional },
            ].map(item => (
              <label
                key={item.id}
                className={`flex items-center gap-2 p-2.5 rounded-lg border cursor-pointer transition-all ${
                  typing.tone_style === item.id
                    ? 'border-cyan-500 bg-cyan-500/10 font-medium'
                    : 'border-black/10 dark:border-white/10 hover:border-cyan-500/30 bg-black/5 dark:bg-white/5'
                } ${disabled ? 'opacity-50 pointer-events-none' : ''}`}
              >
                <input
                  type="radio"
                  name="tone_style"
                  value={item.id}
                  checked={typing.tone_style === item.id}
                  disabled={disabled}
                  onChange={() => onChange({ tone_style: item.id })}
                  className="accent-cyan-500"
                />
                <span className="text-xs">{item.label}</span>
              </label>
            ))}
          </div>
        </div>

        <p className="text-xs opacity-70 leading-relaxed">
          {language === 'vi' ? 'Bộ gõ chạy nền trên nguồn nhập hiện tại. Những ứng dụng chưa hỗ trợ sửa chữ an toàn sẽ nhận phím nguyên bản.' : 'The background driver uses your current input source. Apps without a safe text-editing context receive the original keys.'}
        </p>
      </section>

      <hr className="border-black/10 dark:border-white/10" />

      {/* Hành vi gõ & Chính tả */}
      <section className="space-y-3">
        <h4 className="font-semibold text-xs tracking-wider opacity-60 uppercase">{t.behaviorTitle}</h4>

        <div className="space-y-2.5">
          <label className="flex items-center gap-2.5 cursor-pointer">
            <input
              type="checkbox"
              checked={typing.smart_restore}
              disabled={disabled}
              onChange={e => onChange({ smart_restore: e.target.checked })}
              className="accent-cyan-500 rounded"
            />
            <span className="text-xs">{t.smartRestore}</span>
          </label>

          <label className={`flex items-center gap-2.5 cursor-pointer pl-6 ${!typing.smart_restore ? 'opacity-40 pointer-events-none' : ''}`}>
            <input
              type="checkbox"
              checked={typing.eager_restore}
              disabled={disabled || !typing.smart_restore}
              onChange={e => onChange({ eager_restore: e.target.checked })}
              className="accent-cyan-500 rounded"
            />
            <span className="text-xs">{t.eagerRestore}</span>
          </label>

          <label className="flex items-center gap-2.5 cursor-pointer">
            <input
              type="checkbox"
              checked={typing.spell_check}
              disabled={disabled}
              onChange={e => onChange({ spell_check: e.target.checked })}
              className="accent-cyan-500 rounded"
            />
            <span className="text-xs">{t.spellCheck}</span>
          </label>

          <label className="flex items-center gap-2.5 cursor-pointer">
            <input
              type="checkbox"
              checked={typing.retone}
              disabled={disabled}
              onChange={e => onChange({ retone: e.target.checked })}
              className="accent-cyan-500 rounded"
            />
            <span className="text-xs">{t.retone}</span>
          </label>

          <label className="flex items-center gap-2.5 cursor-pointer">
            <input
              type="checkbox"
              checked={typing.auto_capitalize}
              disabled={disabled}
              onChange={e => onChange({ auto_capitalize: e.target.checked })}
              className="accent-cyan-500 rounded"
            />
            <span className="text-xs">{t.autoCapitalize}</span>
          </label>

          {/* Dấu chấm hai Space */}
          <div className="p-2.5 rounded-lg border border-black/10 dark:border-white/10 bg-black/5 dark:bg-white/5 space-y-1">
            <label className={`flex items-center gap-2.5 ${!period.control_enabled ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'}`}>
              <input
                type="checkbox"
                checked={typing.double_space_period}
                disabled={disabled || !period.control_enabled}
                onChange={e => onChange({ double_space_period: e.target.checked })}
                className="accent-cyan-500 rounded"
              />
              <span className="text-xs font-medium">{t.doubleSpacePeriod}</span>
            </label>
            {!period.control_enabled && (
              <p className="text-[11px] text-amber-500 dark:text-amber-400 pl-6">
                {period.native_state === 'enabled' ? t.doubleSpacePeriodDisabledMac : language === 'vi' ? 'Đang kiểm tra cài đặt hệ thống; tùy chọn tạm khóa.' : 'Checking the system setting; this option is temporarily locked.'}
              </p>
            )}
          </div>
        </div>
      </section>

      <hr className="border-black/10 dark:border-white/10" />

      {/* Phụ âm mở rộng */}
      <section className="space-y-3">
        <h4 className="font-semibold text-xs tracking-wider opacity-60 uppercase">{t.extraOnsetsTitle}</h4>

        <label className="flex items-center gap-2.5 cursor-pointer">
          <input
            type="checkbox"
            checked={onsets.length > 0}
            disabled={disabled}
            onChange={e => handleExtraMaster(e.target.checked)}
            className="accent-cyan-500 rounded"
          />
          <span className="text-xs font-medium">{t.extraOnsetsMaster}</span>
        </label>

        <div className="flex gap-4 pl-6">
          {(['z', 'f', 'w', 'j'] as const).map(letter => (
            <label key={letter} className="flex items-center gap-1.5 cursor-pointer">
              <input
                type="checkbox"
                checked={onsets.includes(letter)}
                disabled={disabled}
                onChange={e => handleExtraLetter(letter, e.target.checked)}
                className="accent-cyan-500 rounded"
              />
              <span className="text-xs font-mono font-semibold uppercase">{letter}</span>
            </label>
          ))}
        </div>
      </section>
    </div>
  );
}
