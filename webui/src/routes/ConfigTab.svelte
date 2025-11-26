<script>
  import { store } from '../lib/store.svelte';
  import { ICONS, DEFAULT_CONFIG } from '../lib/constants';
  
  import './ConfigTab.css';

  let partitionInput = $state(store.config.partitions.join(', '));

  function save() {
    store.config.partitions = partitionInput.split(',').map(s => s.trim()).filter(Boolean);
    store.saveConfig();
  }
</script>

<div class="md3-card">
  <div class="switch-row">
    <span>{store.L.config.verboseLabel}</span>
    <label class="md3-switch">
      <input type="checkbox" bind:checked={store.config.verbose}>
      <span class="track"><span class="thumb"></span></span>
    </label>
  </div>
</div>

<div class="md3-card">
  <div class="text-field">
    <input type="text" id="c-moduledir" bind:value={store.config.moduledir} placeholder={DEFAULT_CONFIG.moduledir} />
    <label for="c-moduledir">{store.L.config.moduleDir}</label>
  </div>
  <div class="text-field">
    <input type="text" id="c-tempdir" bind:value={store.config.tempdir} placeholder={store.L.config.autoPlaceholder} />
    <label for="c-tempdir">{store.L.config.tempDir}</label>
  </div>
  <div class="text-field">
    <input type="text" id="c-mountsource" bind:value={store.config.mountsource} placeholder={DEFAULT_CONFIG.mountsource} />
    <label for="c-mountsource">{store.L.config.mountSource}</label>
  </div>
  <div class="text-field">
    <input type="text" id="c-partitions" bind:value={partitionInput} placeholder="mi_ext, my_stock" />
    <label for="c-partitions">{store.L.config.partitions}</label>
  </div>
</div>

<div class="bottom-actions">
  <button class="btn-tonal" onclick={() => store.loadConfig()} disabled={store.loading.config}>{store.L.config.reload}</button>
  <button class="btn-filled" onclick={save} disabled={store.saving.config}>
    <svg viewBox="0 0 24 24" width="18" height="18"><path d={ICONS.save} fill="currentColor"/></svg>
    {store.saving.config ? store.L.common.saving : store.L.config.save}
  </button>
</div>