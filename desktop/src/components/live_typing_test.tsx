import { nativePermissions } from '../services/native_permissions';
import { useState, useRef, useEffect } from 'react';
import type { Snapshot } from '../types';
import { sendCommand } from '../services/api';
import { Keyboard, RotateCcw, CheckCircle2, Lock } from 'lucide-react';

interface TestKeystrokeResult {
  replace_chars: number;
  insert: string;
  buffer: string;
  mode: string;
  input_method: string;
}

interface Props {
  snapshot: Snapshot;
  compact?: boolean;
  autoFocusWhenReady?: boolean;
}

export function LiveTypingTest({ snapshot, compact = false, autoFocusWhenReady = false }: Props) {
  const vi = snapshot.config.ui.locale !== 'en';
  const runtime = snapshot.runtime;
  const driver = runtime.driver;
  const permissionsReady = nativePermissions(snapshot).ready;
  const canTest =
    runtime.connection_status === 'connected' &&
    permissionsReady &&
    (snapshot.platform === 'windows' || Boolean(driver?.attached)) &&
    runtime.driver_status === 'active' &&
    runtime.effective_mode !== 'suspended';

  const [text, setText] = useState('');
  const [composingBuffer, setComposingBuffer] = useState('');
  const [verifiedKeystrokes, setVerifiedKeystrokes] = useState(0);
  const [engineError, setEngineError] = useState<string | null>(null);

  const textareaRef = useRef<HTMLTextAreaElement | null>(null);
  const stateRef = useRef<{ text: string; start: number; end: number }>({
    text: '',
    start: 0,
    end: 0,
  });
  const queueRef = useRef<Promise<void>>(Promise.resolve());

  useEffect(() => {
    if (canTest && autoFocusWhenReady && textareaRef.current) {
      textareaRef.current.focus();
    }
  }, [canTest, autoFocusWhenReady]);

  // Reset backend composition session when config or active status changes
  useEffect(() => {
    if (!canTest) {
      setComposingBuffer('');
      return;
    }
    void sendCommand<TestKeystrokeResult>('test_keystroke', {
      action: 'reset',
      key: '',
    });
  }, [canTest, snapshot.revision, runtime.effective_mode]);

  const syncDomState = (nextText: string, nextCursor: number) => {
    stateRef.current = { text: nextText, start: nextCursor, end: nextCursor };
    setText(nextText);
    requestAnimationFrame(() => {
      const el = textareaRef.current;
      if (el && document.activeElement === el) {
        el.setSelectionRange(nextCursor, nextCursor);
      }
    });
  };

  const applyEngineEdit = (replaceChars: number, insert: string) => {
    const { text: cur, start, end } = stateRef.current;
    const before = cur.slice(0, start);
    const after = cur.slice(end);
    const beforeCodepoints = Array.from(before);
    const kept =
      replaceChars > 0
        ? beforeCodepoints
            .slice(0, Math.max(0, beforeCodepoints.length - replaceChars))
            .join('')
        : before;
    const nextText = kept + insert + after;
    const nextCursor = (kept + insert).length;
    syncDomState(nextText, nextCursor);
  };

  const enqueueAction = (task: () => Promise<void>) => {
    queueRef.current = queueRef.current.then(task).catch(() => {});
  };

  const handleClear = () => {
    syncDomState('', 0);
    setComposingBuffer('');
    setVerifiedKeystrokes(0);
    setEngineError(null);
    if (canTest) {
      enqueueAction(async () => {
        await sendCommand<TestKeystrokeResult>('test_keystroke', {
          action: 'reset',
          key: '',
        });
      });
    }
    textareaRef.current?.focus();
  };

  const method = snapshot.config.typing.input_method;
  const methodLabel =
    method === 'vni'
      ? 'VNI'
      : method === 'telex_advanced'
      ? vi
        ? 'Telex mở rộng'
        : 'Telex Advanced'
      : 'Telex';

  const samples =
    method === 'vni'
      ? [
          { raw: 'xin chao2', out: 'xin chào' },
          { raw: 'd9u7o75c', out: 'được' },
          { raw: 'Tie61ng Vie65t', out: 'Tiếng Việt' },
        ]
      : [
          { raw: 'xin chaof', out: 'xin chào' },
          { raw: 'ddwojwc', out: 'được' },
          { raw: 'Tieengs Vieetj', out: 'Tiếng Việt' },
        ];

  return (
    <>
    <section className="vx-card rounded-xl border border-black/10 dark:border-white/10 p-4 space-y-3" aria-label={vi ? 'Thử gõ trong ứng dụng khác' : 'Test typing in another app'}>
      <h4 className="text-sm font-semibold">{vi ? 'Thử gõ trong ứng dụng bạn dùng' : 'Try typing in your app'}</h4>
      <p className="text-xs leading-relaxed">{vi ? 'Mở TextEdit hoặc một ô nhập văn bản trong ứng dụng khác, chọn chế độ tiếng Việt trên menu VXKey, rồi gõ tay chuỗi bên dưới. Không dán văn bản.' : 'Open TextEdit or a text field in another app, choose Vietnamese in the VXKey menu, and type the keys below by hand. Do not paste.'}</p>
      <p className="text-sm"><code>{samples[1].raw}</code> → <strong>{samples[1].out}</strong></p>
      <p className="text-xs leading-relaxed">{vi ? 'Thử thêm sau chọn tất cả → xóa và sau xuống dòng. Nếu ra đúng, bạn đã kiểm tra được đường gõ tại ứng dụng đó. Trạng thái “đang hoạt động” chỉ xác nhận bộ gõ đã bật; VXKey không tự đọc kết quả trong ứng dụng khác.' : 'Also test after Select All → Delete and on a new line. Correct output checks the native typing path in that app. “Active” only confirms the driver is enabled; VXKey does not read the result in other apps.'}</p>
    </section>
    <details className="mt-3">
    <summary className="cursor-pointer text-xs py-2">{vi ? 'Thử riêng quy tắc Telex/VNI (core)' : 'Try Telex/VNI rules separately (core)'}</summary>
    <section
      className={`vx-card rounded-xl border border-black/10 dark:border-white/10 ${
        compact ? 'p-3.5 space-y-2.5' : 'p-4 space-y-3'
      }`}
      aria-label={vi ? 'Thử quy tắc core' : 'Core rules sandbox'}
    >
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="flex items-center gap-2">
          <div className="w-7 h-7 rounded-lg bg-cyan-500/15 border border-cyan-500/30 flex items-center justify-center text-cyan-600 dark:text-cyan-400">
            <Keyboard size={15} />
          </div>
          <div>
            <h4 className="text-xs font-semibold">
              {vi ? 'Thử quy tắc gõ trong core Rust' : 'Rust core rules sandbox'}
            </h4>
            <p className="text-[11px] opacity-65">
              {canTest
                ? vi
                  ? 'Ô này dùng phiên core Rust riêng trong app, không kiểm đường gõ native, quyền OS hay thao tác sửa chữ trong ứng dụng khác.'
                  : 'This uses a separate Rust core session inside the app. It does not test native input, OS access, or text replacement in other apps.'
                : vi
                ? snapshot.platform === 'windows' ? 'Nhấn "Bắt đầu gõ tiếng Việt" để mở khóa ô gõ thử.' : 'Hãy cấp đủ quyền và nhấn "Bắt đầu gõ tiếng Việt" để mở khóa ô gõ thử.'
                : snapshot.platform === 'windows' ? 'Click "Start typing Vietnamese" to unlock the test box.' : 'Grant permissions and click "Start typing Vietnamese" to unlock the test box.'}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <span className="px-2 py-0.5 rounded-full text-[11px] font-medium bg-cyan-500/10 text-cyan-600 dark:text-cyan-400 border border-cyan-500/20">
            {methodLabel}
          </span>
          {text.length > 0 && (
            <button
              type="button"
              onClick={handleClear}
              className="inline-flex items-center gap-1 px-2 py-1 rounded-md text-[11px] font-medium border border-black/10 dark:border-white/10 hover:bg-black/5 dark:hover:bg-white/10 transition-colors"
            >
              <RotateCcw size={11} />
              <span>{vi ? 'Xoá' : 'Clear'}</span>
            </button>
          )}
        </div>
      </div>

      {/* Sample hints */}
      <div className="flex flex-wrap items-center gap-1.5 text-[11px]">
        <span className="opacity-60">{vi ? 'Gợi ý gõ thử:' : 'Try typing:'}</span>
        {samples.map(item => (
          <span
            key={item.raw}
            className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md bg-black/5 dark:bg-white/5 border border-black/10 dark:border-white/10 font-mono"
          >
            <span className="text-cyan-600 dark:text-cyan-400">{item.raw}</span>
            <span className="opacity-40">→</span>
            <span className="font-sans font-medium">{item.out}</span>
          </span>
        ))}
      </div>

      <div className="relative">
        <textarea
          ref={textareaRef}
          rows={compact ? 2 : 3}
          value={text}
          disabled={!canTest}
          placeholder={
            canTest
              ? vi
                ? `Nhập phím trực tiếp tại đây (ví dụ: ${samples[0].raw}, ${samples[1].raw}, ${samples[2].raw})…`
                : `Type directly here (e.g. ${samples[0].raw}, ${samples[1].raw}, ${samples[2].raw})…`
              : vi
              ? 'Ô gõ thử sẽ sẵn sàng ngay khi bộ gõ chuyển sang trạng thái Đang hoạt động…'
              : 'The live typing test unlocks as soon as the input method is Active…'
          }
          onSelect={e => {
            const el = e.currentTarget;
            stateRef.current = {
              text: el.value,
              start: el.selectionStart ?? el.value.length,
              end: el.selectionEnd ?? el.value.length,
            };
          }}
          onPaste={() => {
            // Pasted text is raw clipboard content, never simulated as native typing.
            if (canTest) {
              enqueueAction(async () => {
                await sendCommand<TestKeystrokeResult>('test_keystroke', {
                  action: 'reset',
                  key: '',
                });
                setComposingBuffer('');
              });
            }
          }}
          onChange={e => {
            // Fallback for paste or non-intercepted edits
            const el = e.currentTarget;
            stateRef.current = {
              text: el.value,
              start: el.selectionStart ?? el.value.length,
              end: el.selectionEnd ?? el.value.length,
            };
            setText(el.value);
          }}
          onKeyDown={e => {
            if (!canTest || e.nativeEvent.isComposing || e.metaKey || e.ctrlKey || e.altKey) return;
            const el = e.currentTarget;
            stateRef.current = {
              text: el.value,
              start: el.selectionStart ?? el.value.length,
              end: el.selectionEnd ?? el.value.length,
            };

            if (
              ['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown', 'Home', 'End', 'Escape'].includes(
                e.key
              )
            ) {
              enqueueAction(async () => {
                await sendCommand<TestKeystrokeResult>('test_keystroke', {
                  action: 'reset',
                  key: '',
                });
                setComposingBuffer('');
              });
              return;
            }

            if (e.key === 'Backspace' || e.key === 'Enter' || e.key.length === 1) {
              e.preventDefault();
              const pressedKey = e.key;
              enqueueAction(async () => {
                const { text: cur, start, end } = stateRef.current;
                if (start !== end) {
                  await sendCommand<TestKeystrokeResult>('test_keystroke', {
                    action: 'reset',
                    key: '',
                  });
                  const collapsed = cur.slice(0, start) + cur.slice(end);
                  syncDomState(collapsed, start);
                  if (pressedKey === 'Backspace') {
                    setComposingBuffer('');
                    return;
                  }
                }

                const res = await sendCommand<TestKeystrokeResult>('test_keystroke', {
                  action: pressedKey === 'Backspace' ? 'backspace' : 'key',
                  key: pressedKey === 'Enter' ? '\n' : pressedKey === 'Backspace' ? '' : pressedKey,
                });

                if (res.ok && res.data) {
                  setEngineError(null);
                  applyEngineEdit(res.data.replace_chars, res.data.insert);
                  setComposingBuffer(res.data.buffer || '');
                  setVerifiedKeystrokes(c => c + 1);
                } else {
                  setEngineError(
                    vi
                      ? 'Bộ gõ chưa ở trạng thái hoạt động. Hãy kiểm tra quyền hoặc nhấn Bắt đầu gõ.'
                      : 'The input method is not active yet. Check permissions or click Start typing.'
                  );
                }
              });
            }
          }}
          className="w-full rounded-lg border border-black/15 dark:border-white/15 bg-black/[0.03] dark:bg-white/[0.04] px-3 py-2 text-sm leading-relaxed focus:outline-none focus:border-cyan-500 focus:ring-1 focus:ring-cyan-500/40 disabled:opacity-50 disabled:cursor-not-allowed resize-none"
        />
      </div>

      {/* Live Engine Feedback Footer */}
      <div className="flex flex-wrap items-center justify-between gap-2 text-[11px]">
        <div className="flex items-center gap-2">
          {canTest ? (
            <span className="inline-flex items-center gap-1 text-emerald-600 dark:text-emerald-400 font-medium">
              <CheckCircle2 size={12} />
              <span>
                {verifiedKeystrokes > 0
                  ? vi
                    ? `Core riêng đã xử lý ${verifiedKeystrokes} phím`
                    : `Separate core processed ${verifiedKeystrokes} keystrokes`
                  : vi
                  ? 'Sẵn sàng nhận phím gõ thử'
                  : 'Ready for test keystrokes'}
              </span>
            </span>
          ) : (
            <span className="inline-flex items-center gap-1 text-amber-600 dark:text-amber-400">
              <Lock size={12} />
              <span>
                {vi
                  ? 'Đang khóa cho đến khi bộ gõ được kích hoạt'
                  : 'Locked until the input method is activated'}
              </span>
            </span>
          )}
        </div>

        {canTest && (
          <div className="font-mono text-[11px] opacity-75">
            {composingBuffer ? (
              <span>
                {vi ? 'Âm đang ghép:' : 'Composing:'}{' '}
                <strong className="text-cyan-600 dark:text-cyan-400">{composingBuffer}</strong>
              </span>
            ) : (
              <span>
                {vi ? 'Chế độ:' : 'Mode:'}{' '}
                <strong>{snapshot.config.typing.language_mode === 'en' ? 'EN' : 'VI'}</strong>
              </span>
            )}
          </div>
        )}
      </div>

      {engineError && (
        <p role="alert" className="text-[11px] text-rose-500 dark:text-rose-400">
          {engineError}
        </p>
      )}
    </section>
    </details>
    </>
  );
}
