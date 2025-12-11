<script lang="ts">
  import { onMount } from 'svelte';
  import { store } from '../lib/store.svelte';
  import { API } from '../lib/api';
  import { ICONS } from '../lib/constants';
  import './InfoTab.css';
  import Skeleton from '../components/Skeleton.svelte';
  const REPO_OWNER = 'YuzakiKokuban';
  const REPO_NAME = 'meta-hybrid_mount';
  const DONATE_LINK = `https:
  const CACHE_KEY = 'hm_contributors_cache';
  const CACHE_DURATION = 1000 * 60 * 60;
  interface Contributor {
    login: string;
    avatar_url: string;
    html_url: string;
    type: string;
    url: string;
    name?: string;
    bio?: string;
  }
  let contributors = $state<Contributor[]>([]);
  let loading = $state(true);
  let error = $state(false);
  let version = $state(store.version);
  onMount(async () => {
    try {
        const v = await API.getVersion();
        if (v) version = v;
    } catch (e) {
        console.error("Failed to fetch version", e);
    }
    await fetchContributors();
  });
  async function fetchContributors() {
    const cached = localStorage.getItem(CACHE_KEY);
    if (cached) {
      try {
        const { data, timestamp } = JSON.parse(cached);
        if (Date.now() - timestamp < CACHE_DURATION) {
          contributors = data;
          loading = false;
          return;
        }
      } catch (e) {
        localStorage.removeItem(CACHE_KEY);
      }
    }
    try {
      const res = await fetch(`https:
      if (!res.ok) throw new Error('Failed to fetch list');
      const basicList = await res.json();
      const filteredList = basicList.filter((user: Contributor) => {
        const isBotType = user.type === 'Bot';
        const hasBotName = user.login.toLowerCase().includes('bot');
        return !isBotType && !hasBotName;
      });
      const detailPromises = filteredList.map(async (user: Contributor) => {
        try {
            const detailRes = await fetch(user.url);
            if (detailRes.ok) {
                const detail = await detailRes.json();
                return { ...user, bio: detail.bio, name: detail.name || user.login };
            }
        } catch (e) {
            console.warn('Failed to fetch detail for', user.login);
        }
        return user;
      });
      contributors = await Promise.all(detailPromises);
      localStorage.setItem(CACHE_KEY, JSON.stringify({
        data: contributors,
        timestamp: Date.now()
      }));
    } catch (e) {
      console.error(e);
      error = true;
    } finally {
      loading = false;
    }
  }
  function handleLink(e: Event, url: string) {
    e.preventDefault();
    API.openLink(url);
  }
</script>
<div class="info-container">
  <div class="project-header">
    <div class="app-logo">
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 400" width="100%" height="100%">
        <g transform="translate(200, 230) scale(1.3)">
          <g>
            <rect x="-145" y="20" width="290" height="70" rx="16" ry="16" fill="var(--md-sys-color-surface-variant)" />
            <text x="0" y="65" font-family="var(--md-ref-typeface-mono)" font-size="26" font-weight="bold" fill="var(--md-sys-color-on-surface-variant)" text-anchor="middle" letter-spacing="1">/system</text>
          </g>
          <g transform="translate(-115, -95)">
            <defs>
              <clipPath id="logoBlockClip">
                <rect x="0" y="0" width="100" height="100" rx="16" ry="16" />
              </clipPath>
            </defs>
            <rect x="0" y="0" width="100" height="100" 
                  rx="16" ry="16" fill="var(--md-sys-color-primary-container)" />
            <g clip-path="url(#logoBlockClip)">
              <rect x="0" y="0" width="50" height="50" fill="var(--md-sys-color-primary)" />
              <rect x="50" y="50" width="50" height="50" fill="var(--md-sys-color-primary)" />
            </g>
          </g>
          <g transform="translate(15, -95)">
             <rect x="0" y="0" width="100" height="100" rx="16" ry="16" fill="var(--md-sys-color-tertiary-container)" />
          </g>
        </g>
      </svg>
    </div>
    <span class="app-name">{store.L.common.appName}</span>
    <span class="app-version">{version}</span>
  </div>
  <div class="action-grid">
    <a href={`https:
       class="action-card" 
       onclick={(e) => handleLink(e, `https:
        <svg viewBox="0 0 24 24" class="action-icon"><path d={ICONS.github} /></svg>
        <span class="action-label">{store.L.info.projectLink}</span>
    </a>
    <a href={DONATE_LINK} 
       class="action-card"
       onclick={(e) => handleLink(e, DONATE_LINK)}>
        <svg viewBox="0 0 24 24" class="action-icon donate-icon"><path d={ICONS.donate} /></svg>
        <span class="action-label">{store.L.info.donate}</span>
    </a>
  </div>
  <div>
    <div class="section-title">{store.L.info.contributors}</div>
    <div class="contributors-list">
        {#if loading}
            {#each Array(3) as _}
                <div class="contributor-bar">
                    <Skeleton width="48px" height="48px" borderRadius="50%" />
                    <div class="c-info">
                        <div class="skeleton-spacer">
                             <Skeleton width="120px" height="16px" />
                        </div>
                        <Skeleton width="200px" height="12px" />
                    </div>
                </div>
            {/each}
        {:else if error}
            <div class="error-message">
                {store.L.info.loadFail}
            </div>
        {:else}
            {#each contributors as user}
                <a href={user.html_url} 
                   class="contributor-bar"
                   onclick={(e) => handleLink(e, user.html_url)}>
                    <img src={user.avatar_url} alt={user.login} class="c-avatar" />
                    <div class="c-info">
                        <span class="c-name">{user.name || user.login}</span>
                        <span class="c-bio">
                            {user.bio || store.L.info.noBio}
                        </span>
                    </div>
                    <svg viewBox="0 0 24 24" class="c-link-icon"><path d={ICONS.share} /></svg>
                </a>
            {/each}
        {/if}
    </div>
  </div>
</div>