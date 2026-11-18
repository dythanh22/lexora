import { describe, expect, it } from 'vitest';
import en from './en';
import vi from './vi';

describe('i18n', () => {
  it('vi and en define exactly the same keys', () => {
    expect(Object.keys(en).sort()).toEqual(Object.keys(vi).sort());
  });

  it('has a message for every AppErrorCode (docs: core/error.rs)', () => {
    const codes = [
      'input_invalid', 'config_missing', 'config_incompatible', 'auth', 'quota', 'rate_limited',
      'timeout', 'network', 'network_paused', 'invalid_response', 'language_unsupported',
      'cancelled', 'capture_unsupported', 'capture_failed', 'secret_store_unavailable', 'internal'
    ];
    for (const c of codes) expect(vi[`error.${c}`], c).toBeTruthy();
  });
});
