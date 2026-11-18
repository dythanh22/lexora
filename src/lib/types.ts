// Phản chiếu kiểu serialize của Rust (crates/core, src-tauri/src/commands). Giữ đồng bộ bằng tay;
// thay bằng sinh tự động ở M2 nếu số kiểu tăng (không thêm công cụ sớm).
export interface AppError {
  code: string;
  retryable: boolean;
  messageKey: string;
  diagnostic?: string;
}

export type SourceLang = 'auto' | { explicit: string };

export interface Detected {
  language: string;
  confidence: number | null;
}

export type ProviderEvent = { requestId: number; attemptId: number; sequence: number } & (
  | { kind: 'started' }
  | { kind: 'delta'; text: string }
  | { kind: 'completed'; text: string; detected: Detected | null }
  | { kind: 'failed'; error: AppError }
);

export type SnapshotKind = 'ready' | 'needs_input' | 'editing';
export type TextOrigin = 'selection' | 'clipboard' | 'manual';

export interface SessionSnapshot {
  requestId: number;
  kind: SnapshotKind;
  text: string | null;
  origin: TextOrigin | null;
  method: string | null;
}

export type CapabilityState = 'supported' | 'conditional' | 'unavailable' | 'denied' | 'untested';

export interface Capability {
  id: string;
  state: CapabilityState;
  backend: string | null;
  condition: string | null;
  reason: string | null;
  fallback: string | null;
  checkedAt: number | null;
}

export interface ProviderConfig {
  id: string;
  endpoint: string;
  region: string | null;
  secretRef: string;
}

export interface Config {
  schemaVersion: number;
  uiLanguage: 'vi' | 'en';
  targetLanguage: string;
  provider: ProviderConfig;
  pauseNetwork: boolean;
  runInBackground: boolean;
  autostart: boolean;
  popupSize: [number, number] | null;
}
