<script lang="ts">
  import { t } from '../i18n/index.svelte';
  import type { Phase } from '../session-reducer';
  import type { AppError, Detected } from '../types';

  let {
    phase,
    result,
    error,
    detected
  }: { phase: Phase; result: string; error: AppError | null; detected: Detected | null } = $props();
</script>

<section class="pane" aria-live="polite" aria-busy={phase === 'translating'}>
  <span class="muted">{t('label.result')}</span>

  {#if phase === 'needs_input'}
    <p>{t('guidance.copy_then_clipboard')}</p>
  {:else if phase === 'error' && error}
    <p class="error" role="alert">{t(error.messageKey)}</p>
  {:else if phase === 'translating' && result === ''}
    <p class="muted">{t('status.translating')}</p>
  {:else}
    <!-- Văn bản thuần: không bao giờ chèn HTML từ dịch vụ ngoài. -->
    <p class="text">{result}</p>
  {/if}

  {#if detected && phase === 'result'}
    <p class="muted">{t('label.detected')}: {detected.language}</p>
  {/if}
</section>

<style>
  .pane {
    display: grid;
    gap: 6px;
    padding: 10px 12px;
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: var(--radius);
    min-height: 96px;
  }
  p { margin: 0; }
  .text { white-space: pre-wrap; overflow-wrap: anywhere; }
</style>
