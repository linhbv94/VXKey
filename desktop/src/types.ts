export type Envelope<T> = {
  ok: boolean;
  request_id: string | null;
  data: T | null;
  error: null | { code: string; message_key: string; retryable: boolean };
};

export type ShortcutBinding =
  | { kind: 'modifier_chord'; modifiers: string[] }
  | { kind: 'key_chord'; modifiers: string[]; key_code: string };

export interface ShortcutsConfig {
  toggle: ShortcutBinding | null;
  history: ShortcutBinding | null;
  flip: ShortcutBinding | null;
}

export interface Config {
  revision: string;
  schema_version: number;
  ui: {
    locale: string;
    theme: string;
    tray_icon_style: string;
  };
  typing: {
    input_method: string;
    tone_style: string;
    language_mode: string;
    composition_mode: string;
    smart_restore: boolean;
    retone: boolean;
    auto_capitalize: boolean;
    eager_restore: boolean;
    spell_check: boolean;
    extra_onsets: string[];
    double_space_period: boolean;
  };
  apps: {
    remember_per_app: boolean;
    language_memory: Record<string, string>;
    policies: Record<string, {
      input_bypass: boolean;
      clipboard_blocked: boolean;
    }>;
  };
  macros: {
    enabled: boolean;
    in_en: boolean;
    smart_case: boolean;
    entries: MacroEntry[];
  };
  shortcuts: ShortcutsConfig;
  clipboard: {
    enabled: boolean;
    history_mode: string;
    max_entries: number;
    ttl_days: number;
    windows_history_provider: string;
  };
  system: {
    launch_companion_at_login: boolean;
    show_tray_icon: boolean;
  };
}

export interface Capabilities {
  settings: boolean;
  session_history: boolean;
  native_capture: boolean;
  encrypted_history: boolean;
  native_engine: boolean;
  native_session_stream?: boolean;
  native_shortcuts: boolean;
  autostart: boolean;
  windows_history_provider: boolean;
  conversion_text: boolean;
  conversion_files: boolean;
  updater: boolean;
}

export interface Runtime {
  launch_at_login?: {state: 'disabled' | 'not_found' | 'enabled' | 'requires_approval' | 'unavailable'};
  app_names?: Record<string,string>;
  engine_status: 'unavailable' | 'ready' | 'running' | 'degraded' | 'suspended';
  driver_fault?: string | null;
  driver?: {accessibility: boolean; listen_access: boolean; post_access: boolean; attached: boolean};
  driver_status: 'unknown' | 'active' | 'inactive';
  effective_mode: 'vi' | 'en' | 'bypass' | 'inactive' | 'suspended' | 'unavailable';
  connection_status: 'disconnected' | 'connected' | 'incompatible';
  health_state: 'healthy' | 'degraded' | 'suspended' | 'unresponsive';
  applied_revision: string;
  engine_session_id?: string;
  runtime_sequence?: string;
  period_substitution: {
    native_state: 'enabled' | 'disabled' | 'unknown' | 'not_applicable';
    preference: boolean;
    effective_enabled: boolean;
    control_enabled: boolean;
    reason: string;
  };
  current_app: null | {
    app_id: string;
    display_name: string;
    pid: number;
    target_token: string;
    input_bypass: boolean;
    engine_session_id?: string;
    focus_generation?: string;
  };
  capture_state: 'off' | 'monitoring' | 'paused' | 'locked' | 'denied' | 'store_error';
}

export interface Snapshot {
  config: Config;
  revision: string;
  version: string;
  platform: 'macos' | 'windows' | 'linux';
  capabilities: Capabilities;
  runtime: Runtime;
}

export interface HistoryItem {
  id: string;
  preview: string;
  created_at_ms: number;
  byte_length: number;
}

export interface HistoryDetail {
  id: string;
  text: string;
  byte_length: number;
  store_revision: string;
}

export interface MacroEntry {
  id: string;
  trigger: string;
  replacement: string;
}

export type SurfaceType = 'settings' | 'tray' | 'history' | 'onboarding' | 'conversion';
