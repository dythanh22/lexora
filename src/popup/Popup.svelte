<script lang="ts">
  import { onMount } from 'svelte';
  import * as ipc from '../lib/ipc';
  import { i18n, t } from '../lib/i18n/index.svelte';
  import { session } from '../lib/session.svelte';
  import SourceInput from '../lib/components/SourceInput.svelte';
  import ResultPane from '../lib/components/ResultPane.svelte';
  import type { SessionSnapshot } from '../lib/types';

  const TARGETS = ['vi', 'en', 'zh-Hans', 'ja', 'ko', 'fr', 'de', 'es'];

  let source = $state('');
  let target = $state('vi');
  let copied = $state(false);

  const s = $derived(session.state);
  const busy = $derived(s.phase === 'translating');

  async function translate() {
    const requestId = session.state.requestId;
    if (requestId === null || source.trim() === '') return;
    session.dispatch({ type: 'submitting', text: source });
    try {
      const res = await ipc.submitTranslation(
        { requestId, text: source, source: 'auto', target },
        (event) => session.dispatch({ type: 'event', event })
      );
      if (res.attemptId !== null) session.dispatch({ type: 'submitted', attemptId: res.attemptId });
    } catch (e) {
      session.dispatch({ type: 'submit_failed', error: ipc.toAppError(e) });
    }
  }

  function applySnapshot(snap: SessionSnapshot) {
    if (session.state.requestId === snap.requestId) return; // đã xử lý
    session.dispatch({ type: 'snapshot', snapshot: snap });
    source = snap.text ?? '';
    // Selection/clipboard hợp lệ: bắt đầu dịch ngay. needs_input: chỉ hướng dẫn, không gửi gì.
    if (snap.kind === 'ready' && source.trim() !== '') void translate();
  }

  async function cancel() {
    await ipc.cancelTranslation().catch(() => {});
    session.dispatch({ type: 'reset' });
    await ipc.closeWindow('popup').catch(() => {});
  }

  async function copy() {
    try {
      await navigator.clipboard.writeText(s.result);
      copied = true;
      setTimeout(() => (copied = false), 1500);
    } catch {
      // TODO(POC-W): đường copy phía Rust nếu WebView bị chặn clipboard trên Wayland.
    }
  }

  function onwindowkeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') void cancel();
  }

  onMount(() => {
    let unlisten: (() => void) | undefined;
    (async () => {
      try {
        const cfg = await ipc.getConfig();
        target = cfg.targetLanguage;
        i18n.lang = cfg.uiLanguage;
      } catch { /* dùng mặc định */ }
      const snap = await ipc.getSessionSnapshot().catch(() => null);
      if (snap) applySnapshot(snap);
      unlisten = await ipc.onSession(applySnapshot);
    })();
    return () => unlisten?.();
  });
</script>

<svelte:window onkeydown={onwindowkeydown} />

<main>
  <SourceInput bind:value={source} disabled={busy} onsubmit={translate} />

  <div class="row">
    <label class="target">
      <span class="muted">{t('label.target')}</span>
      <select bind:value={target} disabled={busy}>
        {#each TARGETS as code}<option value={code}>{code}</option>{/each}
      </select>
    </label>
    <div class="actions">
      {#if busy}
        <button onclick={cancel}>{t('action.cancel')}</button>
      {:else}
        <button class="primary" onclick={translate} disabled={source.trim() === ''}>
          {t('action.translate')}
        </button>
      {/if}
    </div>
  </div>

  <ResultPane phase={s.phase} result={s.result} error={s.error} detected={s.detected} />

  <div class="actions">
    {#if s.phase === 'result'}
      <button onclick={copy}>{copied ? t('status.copied') : t('action.copy')}</button>
      <button onclick={() => session.dispatch({ type: 'edit' })}>{t('action.edit')}</button>
    {/if}
    {#if s.phase === 'error'}
      <button onclick={translate}>{t('action.retry')}</button>
    {/if}
    <button onclick={cancel}>{t('action.close')}</button>
  </div>
</main>

<style>
  main { display: grid; gap: var(--gap); padding: var(--gap); }
  .row { display: flex; align-items: end; justify-content: space-between; gap: var(--gap); }
  .target { display: grid; gap: 4px; min-width: 120px; }
  .actions { display: flex; gap: 8px; justify-content: flex-end; }
</style>
