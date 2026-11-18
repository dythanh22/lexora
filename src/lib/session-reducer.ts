// Reducer thuần của phiên dịch phía UI. Phản chiếu chính sách chống race của Rust
// (crates/core/src/coordinator.rs): UI VÀ Rust đều lọc sự kiện cũ (thiết kế §8.3).
import type { AppError, Detected, ProviderEvent, SessionSnapshot, TextOrigin } from './types';

export type Phase =
  | 'idle' | 'acquiring' | 'needs_input' | 'editing' | 'translating' | 'result' | 'error';

export interface SessionState {
  requestId: number | null;
  attemptId: number;
  lastSequence: number;
  phase: Phase;
  source: string;
  origin: TextOrigin | null;
  result: string;
  detected: Detected | null;
  error: AppError | null;
}

export const initialState: SessionState = {
  requestId: null,
  attemptId: 0,
  lastSequence: -1,
  phase: 'idle',
  source: '',
  origin: null,
  result: '',
  detected: null,
  error: null
};

export type SessionAction =
  | { type: 'snapshot'; snapshot: SessionSnapshot }
  | { type: 'submitting'; text: string }
  | { type: 'submitted'; attemptId: number }
  | { type: 'submit_failed'; error: AppError }
  | { type: 'event'; event: ProviderEvent }
  | { type: 'edit' }
  | { type: 'reset' };

export function reduce(state: SessionState, action: SessionAction): SessionState {
  switch (action.type) {
    case 'snapshot': {
      const s = action.snapshot;
      // Snapshot đến hai lần (lấy khi mở + sự kiện) phải idempotent.
      if (state.requestId === s.requestId) return state;
      return {
        ...initialState,
        requestId: s.requestId,
        origin: s.origin,
        source: s.text ?? '',
        phase: s.kind === 'ready' ? 'acquiring' : s.kind === 'needs_input' ? 'needs_input' : 'editing'
      };
    }
    case 'submitting':
      return { ...state, source: action.text, phase: 'translating', result: '', detected: null, error: null };
    case 'submitted':
      // Sự kiện có thể tới trước phản hồi của lệnh: không bao giờ hạ attemptId.
      return { ...state, attemptId: Math.max(state.attemptId, action.attemptId) };
    case 'submit_failed':
      return { ...state, phase: 'error', error: action.error };
    case 'edit':
      return state.phase === 'result' || state.phase === 'error' ? { ...state, phase: 'editing' } : state;
    case 'reset':
      return initialState;
    case 'event':
      return applyEvent(state, action.event);
  }
}

function applyEvent(state: SessionState, ev: ProviderEvent): SessionState {
  if (state.requestId === null || ev.requestId !== state.requestId) return state;
  if (state.phase !== 'translating') return state;
  if (ev.attemptId < state.attemptId) return state;

  let next = state;
  if (ev.attemptId > state.attemptId) {
    // Attempt mới (retry hoặc sự kiện tới trước phản hồi): chuyển sang attempt đó.
    next = { ...state, attemptId: ev.attemptId, lastSequence: -1, result: '' };
  }
  if (ev.sequence <= next.lastSequence) return next === state ? state : next;
  next = { ...next, lastSequence: ev.sequence };

  switch (ev.kind) {
    case 'started':
      return next;
    case 'delta':
      return { ...next, result: next.result + ev.text };
    case 'completed':
      // Completed mang toàn văn cuối cùng: thay thế, không nối thêm.
      return { ...next, result: ev.text, detected: ev.detected, phase: 'result', error: null };
    case 'failed':
      return { ...next, phase: 'error', error: ev.error };
  }
}
