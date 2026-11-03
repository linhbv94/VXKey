import { useState, useEffect, useRef } from 'react';
import type { Snapshot, HistoryItem, HistoryDetail } from '../types';
import { uiCopy } from '../services/copy';
import { sendCommand } from '../services/api';
import { Search, X, Copy, Check, Trash2, ShieldAlert } from 'lucide-react';

interface Props {
  snapshot: Snapshot;
  onClose?: () => void;
}

export function HistoryPanel({ snapshot, onClose }: Props) {
  const language = (snapshot.config.ui.locale === 'en' ? 'en' : 'vi') as 'vi' | 'en';
  const t = uiCopy[language].history;

  const [query, setQuery] = useState('');
  const [debouncedQuery, setDebouncedQuery] = useState('');
  const [items, setItems] = useState<HistoryItem[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [selectedDetail, setSelectedDetail] = useState<HistoryDetail | null>(null);
  const [copied, setCopied] = useState(false);
  const [storeRevision, setStoreRevision] = useState('0');
  const [detailRefresh, setDetailRefresh] = useState(0);
  const [loadingList, setLoadingList] = useState(false);
  const [loadingDetail, setLoadingDetail] = useState(false);
  const [errorNotice, setErrorNotice] = useState<string | null>(null);

  const detailGenRef = useRef(0);
  const listGenRef = useRef(0);

  // Debounce query 150ms
  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedQuery(query);
    }, 150);
    return () => clearTimeout(timer);
  }, [query]);

  // Fetch list
  const fetchList = async (searchQuery = '') => {
    const gen=++listGenRef.current;
    detailGenRef.current += 1;
    setSelectedDetail(null);
    setLoadingList(true);
    const res = await sendCommand<{ items: HistoryItem[]; total: number; store_revision: string }>(
      'list_history',
      { query: searchQuery, offset: 0, limit: 50 }
    );
    if (gen !== listGenRef.current) return;
    setLoadingList(false);
    if (res.ok && res.data) {
      setItems(res.data.items);
      setStoreRevision(res.data.store_revision);
      setErrorNotice(null);
      detailGenRef.current += 1;
      setSelectedDetail(null);
      const nextItems = res.data.items;
      setSelectedId(previous => previous && nextItems.some(i => i.id === previous) ? previous : nextItems[0]?.id ?? null);
      setDetailRefresh(value => value + 1);
    } else {
      setItems([]); setSelectedId(null); setSelectedDetail(null);
      setErrorNotice(language === 'vi' ? 'Không thể tải danh sách lịch sử.' : 'Failed to load history list.');
    }
  };

  useEffect(() => {
    void fetchList(debouncedQuery);
    return () => {listGenRef.current += 1; detailGenRef.current += 1;};
  }, [debouncedQuery]);

  // Fetch entry detail with generation guard
  useEffect(() => {
    if (!selectedId) {
      detailGenRef.current += 1;
      setLoadingDetail(false);
      setSelectedDetail(null);
      return;
    }
    const currentGen = ++detailGenRef.current;
    setSelectedDetail(null);
    setLoadingDetail(true);

    (async () => {
      const res = await sendCommand<HistoryDetail>('get_history_entry', {
        entry_id: selectedId,
        expected_store_revision: storeRevision,
      });
      if (currentGen !== detailGenRef.current) return;
      setLoadingDetail(false);

      if (res.ok && res.data) {
        setSelectedDetail(res.data);
        setStoreRevision(res.data.store_revision);
      } else {
        setSelectedDetail(null);
      }
    })();
    return () => {detailGenRef.current += 1;};
  }, [selectedId, storeRevision, detailRefresh]);

  // Copy entry
  const handleCopy = async () => {
    if (!selectedDetail) return;
    try {
      if ('__TAURI_INTERNALS__' in window) {
        const res=await sendCommand('copy_history_entry',{entry_id:selectedDetail.id,expected_store_revision:selectedDetail.store_revision});
        if (!res.ok) throw new Error(res.error?.code);
      } else await navigator.clipboard.writeText(selectedDetail.text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      setErrorNotice(language === 'vi' ? 'Không sao chép được mục này. Hãy tải lại lịch sử.' : 'Could not copy this entry. Refresh history.');
    }
  };

  // Delete entry
  const handleDelete = async (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    const res = await sendCommand<{ removed: boolean; store_revision: string }>('delete_history_entry', {
      entry_id: id,
      expected_store_revision: storeRevision,
    });
    if (res.ok && res.data && res.data.removed) {
      setStoreRevision(res.data.store_revision);
      const remaining = items.filter(item => item.id !== id);
      setItems(remaining);
      if (selectedId === id) {
        setSelectedId(remaining[0]?.id || null);
      }
    } else {
      setErrorNotice(language === 'vi' ? 'Không thể xoá mục này (xung đột phiên bản store).' : 'Failed to delete entry (store revision conflict).');
    }
  };

  // Keyboard navigation
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      e.preventDefault();
      onClose?.();
      return;
    }
    if (e.key === 'Enter' && selectedDetail) {
      handleCopy();
      return;
    }
    if (items.length === 0) return;

    const currentIndex = items.findIndex(i => i.id === selectedId);
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      const nextIndex = Math.min(items.length - 1, currentIndex + 1);
      setSelectedId(items[nextIndex].id);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      const prevIndex = Math.max(0, currentIndex - 1);
      setSelectedId(items[prevIndex].id);
    }
  };

  return (
    <div
      tabIndex={0}
      onKeyDown={handleKeyDown}
      className="w-[600px] h-[460px] max-w-full max-h-full rounded-xl vx-surface flex flex-col shadow-2xl border border-black/10 dark:border-white/10 select-none overflow-hidden outline-none"
    >
      {/* Header Search */}
      <div className="p-3 border-b border-black/10 dark:border-white/10 flex items-center justify-between gap-3 bg-black/5 dark:bg-white/5">
        <div className="flex-1 relative flex items-center">
          <Search size={14} className="absolute left-2.5 opacity-40 pointer-events-none" />
          <input
            type="text"
            placeholder={t.searchPlaceholder}
            value={query}
            onChange={e => setQuery(e.target.value)}
            className="w-full bg-white dark:bg-black/30 border border-black/10 dark:border-white/10 rounded-lg pl-8 pr-8 py-1.5 text-xs text-slate-900 dark:text-white placeholder-slate-400 dark:placeholder-white/40 focus:outline-none focus:border-cyan-500"
          />
          {query && (
            <button
              type="button"
              onClick={() => setQuery('')}
              className="absolute right-2.5 opacity-40 hover:opacity-100"
            >
              <X size={14} />
            </button>
          )}
        </div>
        {onClose && (
          <button
            type="button"
            onClick={onClose}
            className="opacity-60 hover:opacity-100 p-1"
          >
            <X size={16} />
          </button>
        )}
      </div>

      {errorNotice && (
        <div className="px-3 py-1 bg-rose-500/10 text-rose-400 text-[11px] border-b border-rose-500/20">
          {errorNotice}
        </div>
      )}

      {/* Main Split Layout */}
      <div className="flex-1 flex min-h-0 divide-x divide-black/10 dark:divide-white/10">
        {/* Left Column: Entry List (240px) */}
        <div className="w-[240px] overflow-y-auto divide-y divide-black/5 dark:divide-white/5 bg-black/5 dark:bg-black/20">
          {loadingList && items.length === 0 ? (
            <div className="p-4 text-center text-xs opacity-50 italic">
              {language === 'vi' ? 'Đang tải…' : 'Loading…'}
            </div>
          ) : items.length === 0 ? (
            <div className="p-4 text-center text-xs opacity-50 italic">
              {t.empty}
            </div>
          ) : (
            items.map(item => {
              const active = item.id === selectedId;
              return (
                <div
                  key={item.id}
                  onClick={() => setSelectedId(item.id)}
                  className={`p-2.5 text-xs cursor-pointer transition-all flex items-start justify-between gap-2 group ${
                    active
                      ? 'bg-cyan-500/20 text-cyan-600 dark:text-cyan-300 font-medium'
                      : 'hover:bg-black/5 dark:hover:bg-white/5 text-slate-700 dark:text-white/80'
                  }`}
                >
                  <div className="flex-1 min-w-0">
                    <p className="line-clamp-2 text-xs leading-snug break-words">
                      {item.preview}
                    </p>
                    <span className="text-[10px] opacity-50 mt-1 block font-mono">
                      {item.byte_length} bytes
                    </span>
                  </div>
                  <button
                    type="button"
                    onClick={e => handleDelete(item.id, e)}
                    className="opacity-0 group-hover:opacity-100 hover:text-rose-500 p-1 transition-opacity"
                    title={t.delete}
                  >
                    <Trash2 size={12} />
                  </button>
                </div>
              );
            })
          )}
        </div>

        {/* Right Column: Preview */}
        <div className="flex-1 flex flex-col p-4 bg-black/5 dark:bg-black/10 overflow-hidden">
          {loadingDetail ? (
            <div className="flex-1 flex items-center justify-center text-xs opacity-50 italic">
              {language === 'vi' ? 'Đang đọc nội dung…' : 'Reading entry…'}
            </div>
          ) : selectedDetail ? (
            <div className="flex-1 flex flex-col min-h-0">
              <div className="flex-1 overflow-y-auto p-3 rounded-lg bg-white dark:bg-black/40 border border-black/5 dark:border-white/5 font-mono text-xs text-slate-800 dark:text-slate-100 leading-relaxed whitespace-pre-wrap break-words select-text">
                {selectedDetail.text}
              </div>
              <div className="mt-2 text-[11px] opacity-50 flex items-center justify-between font-mono">
                <span>{selectedDetail.byte_length} bytes</span>
                <span>{selectedDetail.text.length} chars</span>
              </div>
            </div>
          ) : (
            <div className="flex-1 flex items-center justify-center text-xs opacity-40">
              {language === 'vi' ? 'Chọn một mục để xem nội dung' : 'Select an entry to preview'}
            </div>
          )}
        </div>
      </div>

      {/* Footer */}
      <div className="p-3 border-t border-black/10 dark:border-white/10 flex items-center justify-between bg-black/5 dark:bg-white/5 text-xs">
        <span className="opacity-60 text-[11px] flex items-center gap-1.5 font-medium">
          <ShieldAlert size={13} className="text-cyan-600 dark:text-cyan-400" />
          {t.sessionInfo} · {items.length}/{snapshot.config.clipboard.max_entries}
        </span>

        <div className="flex items-center gap-2">
          <button
            type="button"
            disabled={!selectedDetail}
            onClick={handleCopy}
            className="flex items-center gap-1 bg-cyan-600 hover:bg-cyan-500 disabled:opacity-40 text-white px-3 py-1 rounded text-xs font-medium transition-colors shadow-sm"
          >
            {copied ? <Check size={12} /> : <Copy size={12} />}
            <span>{copied ? t.copied : `${t.copy} ↵`}</span>
          </button>
        </div>
      </div>
    </div>
  );
}
