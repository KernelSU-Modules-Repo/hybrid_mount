<script>
  import { store } from '../lib/store.svelte';
  import { ICONS } from '../lib/constants';
  import { onMount } from 'svelte';
  
  import './LogsTab.css';

  onMount(() => {
    store.loadLogs();
  });
</script>

<div class="log-container">
  {#if store.loading.logs}
    <div style="padding: 20px; text-align: center;">{store.L.logs.loading}</div>
  {:else if store.logs.length === 0}
    <div style="padding: 20px; text-align: center;">{store.L.logs.empty}</div>
  {:else}
    {#each store.logs as line}
      <span class="log-entry">
        <span class="log-{line.type}">{line.text}</span>
      </span>
    {/each}
  {/if}
</div>

<div class="bottom-actions">
  <button class="btn-filled" onclick={() => store.loadLogs()} disabled={store.loading.logs}>
    <svg viewBox="0 0 24 24" width="18" height="18"><path d={ICONS.refresh} fill="currentColor"/></svg>
    {store.L.logs.refresh}
  </button>
</div>