import { invoke } from '@tauri-apps/api/core';
import type { Envelope, Snapshot, HistoryItem, MacroEntry } from '../types';

const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

// Fallback in-memory state for browser visual preview
let mockSnapshot: Snapshot = {
  config: {
    revision: '1',
    schema_version: 1,
    ui: {
      locale: 'vi',
      theme: 'dark',
      tray_icon_style: 'serious',
    },
    typing: {
      input_method: 'telex',
      tone_style: 'modern',
      language_mode: 'vi',
      composition_mode: 'direct_verified',
      smart_restore: true,
      eager_restore: true,
      spell_check: false,
      retone: false,
      auto_capitalize: false,
      extra_onsets: [],
      double_space_period: false,
    },
    apps: {
      remember_per_app: true,
      language_memory: {
        'com.google.Chrome': 'vi',
        'com.apple.Terminal': 'en',
      },
      policies: {
        'com.openai.chat': {
          input_bypass: true,
          clipboard_blocked: false,
        },
      },
    },
    macros: {
      enabled: true,
      in_en: true,
      smart_case: true,
      entries: [],
    },
    shortcuts: {
      toggle: { kind: 'modifier_chord', modifiers: ['control', 'alt'] },
      flip: null,
      history: { kind: 'key_chord', modifiers: ['control', 'meta'], key_code: 'KeyV' },
    },
    clipboard: {
      enabled: false,
      history_mode: 'session',
      max_entries: 200,
      ttl_days: 7,
      windows_history_provider: 'native',
    },
    system: {
      launch_companion_at_login: false,
      show_tray_icon: true,
    },
  },
  revision: '1',
  version: '0.1.0',
  platform: 'macos',
  capabilities: {
    settings: true,
    session_history: true,
    native_capture: false,
    encrypted_history: false,
    native_engine: false,
    native_shortcuts: false,
    autostart: false,
    windows_history_provider: false,
    conversion_text: true,
    conversion_files: false,
    updater: false,
  },
  runtime: {
    engine_status: 'unavailable',
    driver_status: 'unknown',
    effective_mode: 'suspended',
    connection_status: 'disconnected',
    health_state: 'suspended',
    applied_revision: '0',
    period_substitution: {
      native_state: 'unknown',
      preference: false,
      effective_enabled: false,
      control_enabled: false,
      reason: 'checking',
    },
    current_app: null,
    capture_state: 'off',
  },
};

let mockMacros: MacroEntry[] = [
  { id: '1', trigger: 'vx', replacement: 'VXUtilities' },
  { id: '2', trigger: 'dc', replacement: 'được' },
];

let mockHistory: HistoryItem[] = [
  { id: '1', preview: 'Xin chào thế giới! Đây là ví dụ lịch sử clipboard.', created_at_ms: Date.now() - 60000, byte_length: 64 },
  { id: '2', preview: 'https://github.com/linhbv94/VXKey', created_at_ms: Date.now() - 120000, byte_length: 33 },
  { id: '3', preview: 'Cargo test --manifest-path crates/vxkey-runtime/Cargo.toml', created_at_ms: Date.now() - 300000, byte_length: 58 },
];

let mockStoreRevision = 1;

export async function sendCommand<T>(command: string, payload: Record<string, any> = {}): Promise<Envelope<T>> {
  if (isTauri) {
    try {
      return await invoke<Envelope<T>>('vxkey_command', {
        request: {
          command,
          ...payload,
        },
      });
    } catch (err: any) {
      return {
        ok: false,
        request_id: null,
        data: null,
        error: {
          code: 'ipc_error',
          message_key: typeof err === 'string' ? err : 'errors.ipc_error',
          retryable: true,
        },
      };
    }
  }

  // Browser mock fallback
  await new Promise(r => setTimeout(r, 20));
  if (command === 'get_snapshot') {
    return { ok: true, request_id: 'mock-1', data: JSON.parse(JSON.stringify(mockSnapshot)) as T, error: null };
  }
  if (command === 'patch_config') {
    const expected = payload.expected_revision;
    if (expected !== mockSnapshot.revision) {
      return { ok: false, request_id: 'mock-err', data: null, error: { code: 'revision_conflict', message_key: 'errors.revision_conflict', retryable: true } };
    }
    const nextRev = String(BigInt(mockSnapshot.revision) + 1n);
    const changes = payload.changes || {};
    for (const group of Object.keys(changes)) {
      (mockSnapshot.config as any)[group] = { ...(mockSnapshot.config as any)[group], ...changes[group] };
    }
    mockSnapshot.revision = nextRev;
    mockSnapshot.config.revision = nextRev;
    return {
      ok: true,
      request_id: 'mock-2',
      data: { persisted_revision: nextRev, applied_revision: '0', engine_ack_state: 'unavailable' } as T,
      error: null,
    };
  }
  if (command === 'list_macros') {
    return {
      ok: true,
      request_id: 'mock-3',
      data: { items: mockMacros, revision: mockSnapshot.revision } as T,
      error: null,
    };
  }
  if (command === 'replace_macros') {
    if (payload.expected_revision !== mockSnapshot.revision) {
      return { ok: false, request_id: 'mock-err', data: null, error: { code: 'revision_conflict', message_key: 'errors.revision_conflict', retryable: true } };
    }
    const nextRev = String(BigInt(mockSnapshot.revision) + 1n);
    mockMacros = payload.entries || [];
    mockSnapshot.config.macros.enabled = payload.enabled ?? mockSnapshot.config.macros.enabled;
    mockSnapshot.config.macros.in_en = payload.in_en ?? mockSnapshot.config.macros.in_en;
    mockSnapshot.config.macros.smart_case = payload.smart_case ?? mockSnapshot.config.macros.smart_case;
    mockSnapshot.revision = nextRev;
    mockSnapshot.config.revision = nextRev;
    return {
      ok: true,
      request_id: 'mock-4',
      data: { persisted_revision: nextRev, applied_revision: '0', engine_ack_state: 'unavailable' } as T,
      error: null,
    };
  }
  if (command === 'list_history') {
    const q = (payload.query || '').toLowerCase();
    const filtered = q ? mockHistory.filter(h => h.preview.toLowerCase().includes(q)) : mockHistory;
    return {
      ok: true,
      request_id: 'mock-5',
      data: { items: filtered, total: filtered.length, store_revision: String(mockStoreRevision) } as T,
      error: null,
    };
  }
  if (command === 'get_history_entry') {
    const found = mockHistory.find(h => h.id === payload.entry_id);
    if (!found) {
      return { ok: false, request_id: 'mock-6', data: null, error: { code: 'not_found', message_key: 'errors.not_found', retryable: false } };
    }
    return {
      ok: true,
      request_id: 'mock-6',
      data: { id: found.id, text: found.preview, byte_length: found.byte_length, store_revision: String(mockStoreRevision) } as T,
      error: null,
    };
  }
  if (command === 'delete_history_entry') {
    mockHistory = mockHistory.filter(h => h.id !== payload.entry_id);
    mockStoreRevision += 1;
    return { ok: true, request_id: 'mock-7', data: { removed: true, store_revision: String(mockStoreRevision) } as T, error: null };
  }
  // Browser preview never simulates charset conversion or native OS actions.

  return {
    ok: false,
    request_id: 'mock-unsupported',
    data: null,
    error: { code: 'unsupported', message_key: `errors.unsupported`, retryable: false },
  };
}
