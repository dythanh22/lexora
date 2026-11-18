import { describe, expect, it } from 'vitest';
import { initialState, reduce, type SessionAction, type SessionState } from './session-reducer';
import type { ProviderEvent } from './types';

const run = (actions: SessionAction[], from: SessionState = initialState) =>
  actions.reduce(reduce, from);

const snap = (requestId = 1): SessionAction => ({
  type: 'snapshot',
  snapshot: { requestId, kind: 'ready', text: 'hello', origin: 'selection', method: 'at_spi' }
});

const ev = (e: Record<string, unknown>): SessionAction => ({
  type: 'event',
  event: { requestId: 1, attemptId: 1, sequence: 0, ...e } as ProviderEvent
});

describe('session reducer', () => {
  it('snapshot is idempotent for the same request', () => {
    const once = run([snap()]);
    expect(reduce(once, snap())).toBe(once);
  });

  it('completed replaces streamed text instead of appending', () => {
    const s = run([
      snap(), { type: 'submitting', text: 'hello' },
      ev({ kind: 'delta', sequence: 1, text: 'xin ' }),
      ev({ kind: 'completed', sequence: 2, text: 'xin chào', detected: null })
    ]);
    expect(s.result).toBe('xin chào');
    expect(s.phase).toBe('result');
  });

  it('accepts events that arrive before the command response', () => {
    const s = run([
      snap(), { type: 'submitting', text: 'hello' },
      ev({ kind: 'completed', sequence: 1, text: 'ok', detected: null }),
      { type: 'submitted', attemptId: 1 }
    ]);
    expect(s.phase).toBe('result');
    expect(s.attemptId).toBe(1);
  });

  it('drops events from another request, an older attempt, or out of order', () => {
    const base = run([snap(), { type: 'submitting', text: 'hello' }, { type: 'submitted', attemptId: 2 }]);
    const other = reduce(base, { type: 'event', event: { requestId: 9, attemptId: 2, sequence: 0, kind: 'started' } });
    expect(other).toBe(base);
    const old = reduce(base, ev({ kind: 'completed', attemptId: 1, sequence: 5, text: 'stale', detected: null }));
    expect(old.phase).toBe('translating');
    const s1 = reduce(base, ev({ kind: 'delta', attemptId: 2, sequence: 3, text: 'a' }));
    const s2 = reduce(s1, ev({ kind: 'delta', attemptId: 2, sequence: 2, text: 'b' }));
    expect(s2.result).toBe('a');
  });

  it('ignores events after the session reached a terminal state', () => {
    const s = run([
      snap(), { type: 'submitting', text: 'hello' },
      ev({ kind: 'completed', sequence: 1, text: 'ok', detected: null }),
      ev({ kind: 'failed', sequence: 2, error: { code: 'network', retryable: true, messageKey: 'error.network' } })
    ]);
    expect(s.phase).toBe('result');
    expect(s.error).toBeNull();
  });

  it('retry: a newer attempt supersedes the previous stream', () => {
    const failed = run([
      snap(), { type: 'submitting', text: 'hello' },
      ev({ kind: 'failed', sequence: 0, error: { code: 'network', retryable: true, messageKey: 'error.network' } })
    ]);
    expect(failed.phase).toBe('error');
    const retried = run([
      { type: 'submitting', text: 'hello' },
      ev({ kind: 'completed', attemptId: 2, sequence: 0, text: 'ok', detected: null })
    ], failed);
    expect(retried.phase).toBe('result');
    expect(retried.attemptId).toBe(2);
  });

  it('edit only applies after a result or an error', () => {
    const editing = run([snap()]);
    expect(reduce(editing, { type: 'edit' })).toBe(editing);
  });
});
