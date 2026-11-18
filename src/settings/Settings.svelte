<script lang="ts">
  import { onMount } from 'svelte';
  import * as ipc from '../lib/ipc';
  import { i18n, t } from '../lib/i18n/index.svelte';
  import type { Capability, Config } from '../lib/types';

  type Tab = 'general' | 'engine' | 'shortcuts' | 'capabilities' | 'privacy' | 'about';
  const TABS: Tab[] = ['general', 'engine', 'shortcuts', 'capabilities', 'privacy', 'about'];

  let tab = $state<Tab>('general');
  let config = $state<Config | null>(null);
  let caps = $state<Capability[]>([]);
  let keyInput = $state('');
  let sessionOnly = $state(false);
  let message = $state('');
  let failure = $state('');

  async function load() {
    config = await ipc.getConfig();
    i18n.lang = config.uiLanguage;
    caps = await ipc.getCapabilities();
  }

  async function run(fn: () => Promise<void>) {
    message = '';
    failure = '';
    try {
      await fn();
      message = t('status.saved');
    } catch (e) {
      failure = t(ipc.toAppError(e).messageKey);
    }
  }

  const saveConfig = () =>
    run(async () => {
      if (!config) return;
      await ipc.updateConfig($state.snapshot(config) as Config);
      i18n.lang = config.uiLanguage;
    });

  const saveKey = () =>
    run(async () => {
      if (!config) return;
      await ipc.saveSecret(config.provider.id, keyInput, sessionOnly);
    }).finally(() => (keyInput = '')); // xóa khỏi bộ nhớ UI ngay sau khi gửi

  const removeKey = () =>
    run(async () => {
      if (config) await ipc.deleteSecret(config.provider.id);
    });

  onMount(() => void load().catch(() => {}));
</script>

<div class="shell">
  <nav role="tablist" aria-label={t('action.settings')}>
    {#each TABS as id}
      <button role="tab" aria-selected={tab === id} class:active={tab === id} onclick={() => (tab = id)}>
        {t(`settings.${id}`)}
      </button>
    {/each}
  </nav>

  <section role="tabpanel">
    {#if !config}
      <p class="muted">…</p>
    {:else if tab === 'general'}
      <label>
        <span class="muted">{t('settings.ui_language')}</span>
        <select bind:value={config.uiLanguage}>
          <option value="vi">Tiếng Việt</option>
          <option value="en">English</option>
        </select>
      </label>
      <label>
        <span class="muted">{t('label.target')}</span>
        <input type="text" bind:value={config.targetLanguage} />
      </label>
      <label class="check">
        <input type="checkbox" bind:checked={config.runInBackground} />
        {t('settings.run_in_background')}
      </label>
      <button class="primary" onclick={saveConfig}>{t('action.save')}</button>
    {:else if tab === 'engine'}
      <label>
        <span class="muted">{t('settings.endpoint')}</span>
        <input type="text" bind:value={config.provider.endpoint} />
      </label>
      <label>
        <span class="muted">{t('settings.region')}</span>
        <input type="text" bind:value={config.provider.region} />
      </label>
      <button onclick={saveConfig}>{t('action.save')}</button>
      <hr />
      <label>
        <span class="muted">{t('settings.key')}</span>
        <input type="password" autocomplete="off" bind:value={keyInput} />
      </label>
      <p class="muted">{t('settings.key_hint')}</p>
      <label class="check">
        <input type="checkbox" bind:checked={sessionOnly} />
        {t('settings.key_session_only')}
      </label>
      <div class="row">
        <button class="primary" onclick={saveKey} disabled={keyInput.trim() === ''}>{t('action.save')}</button>
        <button onclick={removeKey}>{t('action.delete_key')}</button>
      </div>
    {:else if tab === 'shortcuts'}
      <p>{t('settings.shortcuts_note')}</p>
      <pre>lexora --action selection
lexora --action clipboard
lexora --action input</pre>
    {:else if tab === 'capabilities'}
      <ul class="caps">
        {#each caps as c}
          <li>
            <strong>{t(`cap.id.${c.id}`)}</strong>
            <span class="muted">{t(`cap.state.${c.state}`)}{c.backend ? ` · ${c.backend}` : ''}</span>
            {#if c.reason}<span class="muted">{c.reason}</span>{/if}
          </li>
        {/each}
      </ul>
    {:else if tab === 'privacy'}
      <p>{t('settings.privacy_note')}</p>
      <label class="check">
        <input type="checkbox" bind:checked={config.pauseNetwork} />
        {t('settings.pause_network')}
      </label>
      <button class="primary" onclick={saveConfig}>{t('action.save')}</button>
    {:else}
      <p>Lexora 0.0.0</p>
      <p class="muted">{t('settings.license_note')}</p>
      <button onclick={() => ipc.quitApp()}>{t('action.quit')}</button>
    {/if}

    {#if message}<p class="muted" role="status">{message}</p>{/if}
    {#if failure}<p class="error" role="alert">{failure}</p>{/if}
  </section>
</div>

<style>
  .shell { display: grid; grid-template-columns: 190px 1fr; height: 100%; }
  nav { display: grid; align-content: start; gap: 2px; padding: 12px; border-right: 1px solid var(--line); }
  nav button { text-align: left; border-color: transparent; background: transparent; }
  nav button.active { background: var(--surface); border-color: var(--line); font-weight: 600; }
  section { display: grid; align-content: start; gap: var(--gap); padding: 16px; overflow: auto; }
  label { display: grid; gap: 4px; }
  label.check { display: flex; align-items: center; gap: 8px; }
  label.check input { width: auto; }
  .row { display: flex; gap: 8px; }
  .caps { list-style: none; margin: 0; padding: 0; display: grid; gap: 10px; }
  .caps li { display: grid; gap: 2px; }
  pre { background: var(--surface); border: 1px solid var(--line); border-radius: var(--radius); padding: 10px; }
  hr { border: 0; border-top: 1px solid var(--line); width: 100%; }
</style>
