<script>
  import { store } from '../lib/store.svelte';
  import { ICONS } from '../lib/constants';
  import { onMount } from 'svelte';
  
  import './ModulesTab.css';

  onMount(() => {
    store.loadModules();
  });
</script>

<div class="md3-card" style="padding: 16px;">
  <p style="margin: 0; font-size: 14px; color: var(--md-sys-color-on-surface-variant); line-height: 1.5;">
    {store.L.modules.desc}
  </p>
</div>

{#if store.modules.length === 0}
  <div style="text-align:center; padding: 40px; opacity: 0.6">
    {store.loading.modules ? store.L.modules.scanning : store.L.modules.empty}
  </div>
{:else}
  <div class="rules-list">
    {#each store.modules as mod (mod.id)}
      <div class="rule-card">
        <div class="rule-info">
          <div style="display:flex; flex-direction:column;">
            <span class="module-name">{mod.name}</span>
            <span class="module-id">{mod.id}</span>
          </div>
        </div>
        <div class="text-field" style="margin-bottom:0; width: 140px; flex-shrink: 0;">
          <select bind:value={mod.mode}>
            <option value="auto">{store.L.modules.modeAuto}</option>
            <option value="magic">{store.L.modules.modeMagic}</option>
          </select>
        </div>
      </div>
    {/each}
  </div>
{/if}

<div class="bottom-actions">
  <button class="btn-tonal" onclick={() => store.loadModules()} disabled={store.loading.modules} title={store.L.modules.reload}>
    <svg viewBox="0 0 24 24" width="20" height="20"><path d={ICONS.refresh} fill="currentColor"/></svg>
  </button>
  <button class="btn-filled" onclick={() => store.saveModules()} disabled={store.saving.modules}>
    <svg viewBox="0 0 24 24" width="18" height="18"><path d={ICONS.save} fill="currentColor"/></svg>
    {store.saving.modules ? store.L.common.saving : store.L.modules.save}
  </button>
</div>