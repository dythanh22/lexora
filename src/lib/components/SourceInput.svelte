<script lang="ts">
  import { t } from '../i18n/index.svelte';

  let {
    value = $bindable(''),
    disabled = false,
    onsubmit
  }: { value: string; disabled?: boolean; onsubmit: () => void } = $props();

  // Enter gửi, Shift+Enter xuống dòng. Không gửi khi bộ gõ (IME) đang soạn chữ:
  // bắt buộc với tiếng Việt (thiết kế §4.4, NFR07).
  function onkeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey && !e.isComposing && e.keyCode !== 229) {
      e.preventDefault();
      onsubmit();
    }
  }
</script>

<label class="field">
  <span class="muted">{t('label.source')}</span>
  <textarea
    bind:value
    {disabled}
    {onkeydown}
    rows="4"
    placeholder={t('guidance.input_placeholder')}
  ></textarea>
</label>

<style>
  .field { display: grid; gap: 4px; }
  textarea { resize: vertical; min-height: 84px; }
</style>
