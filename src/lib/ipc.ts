// Lớp IPC duy nhất của UI. Không có hàm nào đọc lại plaintext key (thiết kế §8.5).
import { Channel, invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  AppError, Capability, Config, ProviderEvent, SessionSnapshot, SourceLang
} from './types';

export const SESSION_EVENT = 'lexora:session';

export function toAppError(e: unknown): AppError {
  if (typeof e === 'object' && e !== null && 'messageKey' in e) return e as AppError;
  return { code: 'internal', retryable: false, messageKey: 'error.internal' };
}

export const getSessionSnapshot = () => invoke<SessionSnapshot | null>('get_session_snapshot');

export const onSession = (cb: (s: SessionSnapshot) => void): Promise<UnlistenFn> =>
  listen<SessionSnapshot>(SESSION_EVENT, (e) => cb(e.payload));

export interface SubmitArgs {
  requestId: number;
  text: string;
  source: SourceLang;
  target: string;
}

/** Sự kiện dịch đi qua Channel gắn với đúng lời gọi này, không broadcast. */
export function submitTranslation(args: SubmitArgs, onEvent: (e: ProviderEvent) => void) {
  const channel = new Channel<ProviderEvent>();
  channel.onmessage = onEvent;
  return invoke<{ attemptId: number | null; duplicate: boolean }>('submit_translation', {
    requestId: args.requestId,
    text: args.text,
    source: args.source,
    target: args.target,
    onEvent: channel
  });
}

export const cancelTranslation = () => invoke<void>('cancel_translation');
export const triggerAction = (action: 'selection' | 'clipboard' | 'input') =>
  invoke<void>('trigger_action', { action });
export const getCapabilities = () => invoke<Capability[]>('get_capabilities');
export const getConfig = () => invoke<Config>('get_config');
export const updateConfig = (config: Config) => invoke<void>('update_config', { config });
export const saveSecret = (providerId: string, value: string, sessionOnly: boolean) =>
  invoke<void>('save_secret', { providerId, value, sessionOnly });
export const deleteSecret = (providerId: string) => invoke<void>('delete_secret', { providerId });
export const closeWindow = (label: 'popup' | 'settings') => invoke<void>('close_window', { label });
export const quitApp = () => invoke<void>('quit_app');
