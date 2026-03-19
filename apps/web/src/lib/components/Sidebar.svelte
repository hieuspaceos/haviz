<script lang="ts">
  import './sidebar.css';
  import { api } from '../api/client';
  import { zaloConversations, addLog } from '../stores/app';

  let activeTab = 'inbox';
  let collapsed = true; // Default: collapsed

  const tabs = [
    {
      id: 'inbox', label: 'Inbox',
      icon: `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>`
    },
    {
      id: 'contacts', label: 'Contacts',
      icon: `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>`
    },
    {
      id: 'templates', label: 'Templates',
      icon: `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>`
    },
    {
      id: 'settings', label: 'Settings',
      icon: `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.07 4.93a10 10 0 0 1 0 14.14M4.93 4.93a10 10 0 0 0 0 14.14"/><path d="M12 2v2M12 20v2M2 12h2M20 12h2"/></svg>`
    },
  ];

  export let onNavigate: (path: string) => void = () => {};

  async function refresh() {
    try {
      let data: any;
      try {
        data = await api.zalo.conversations();
      } catch {
        data = await api.conversations();
      }
      $zaloConversations = data.conversations
        ?.filter((c: any) => c.name?.trim() || c.contact_name?.trim())
        .map((c: any) => ({
          name: c.name || c.contact_name,
          time: c.time || '',
          preview: c.preview || c.last_message_preview || '',
          sender: c.sender || '',
        })) || [];
      addLog(`${$zaloConversations.length} conversations loaded`);
    } catch (e) {
      addLog('Failed to load conversations', 'err');
    }
  }

  async function openConv(index: number) {
    try {
      await api.zalo.open(index);
      addLog(`Opened conversation #${index}`);
    } catch (e) {
      addLog('Failed to open', 'err');
    }
  }

  function getInitials(name: string): string {
    return name.split(' ').map(w => w[0]).join('').slice(0, 2).toUpperCase();
  }

  refresh();
  const interval = setInterval(refresh, 15000);
</script>

<aside
  class="sidebar"
  class:sidebar--expanded={!collapsed}
  class:sidebar--collapsed={collapsed}
>
  <!-- Toggle -->
  <button
    class="sidebar-toggle"
    on:click={() => collapsed = !collapsed}
    title={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
  >
    {#if collapsed}
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 18 15 12 9 6"/></svg>
    {:else}
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="15 18 9 12 15 6"/></svg>
    {/if}
  </button>

  <!-- Nav tabs -->
  <nav class="sidebar-nav" class:sidebar-nav--col={collapsed}>
    {#each tabs as tab}
      <button
        class="nav-tab"
        class:nav-tab--active={activeTab === tab.id}
        class:nav-tab--collapsed={collapsed}
        on:click={() => { activeTab = tab.id; if (collapsed) collapsed = false; onNavigate('/' + tab.id); }}
        title={tab.label}
      >
        <span class="nav-tab-icon">{@html tab.icon}</span>
        {#if !collapsed}<span class="nav-tab-label">{tab.label}</span>{/if}
      </button>
    {/each}
  </nav>

  <!-- Conversation list -->
  {#if !collapsed}
    <div class="conv-list">
      {#each $zaloConversations as conv, i}
        <button class="conv-item" on:click={() => openConv(i + 1)}>
          <div class="conv-avatar">{getInitials(conv.name)}</div>
          <div class="conv-info">
            <div class="conv-row">
              <span class="conv-name">{conv.name}</span>
              <span class="conv-time">{conv.time}</span>
            </div>
            {#if conv.preview}
              <div class="conv-preview">
                {#if conv.sender}<span class="conv-sender">{conv.sender}:</span>{/if}
                {conv.preview}
              </div>
            {/if}
          </div>
        </button>
      {/each}

      {#if $zaloConversations.length === 0}
        <div class="conv-empty">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" style="margin-bottom:6px;opacity:0.4"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
          No conversations.<br/>Is Zalo Web loaded?
        </div>
      {/if}
    </div>

    <div class="sidebar-footer">
      <button class="btn btn-ghost btn-sm" style="width:100%;justify-content:center;" on:click={refresh}>
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/>
          <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
        </svg>
        Refresh
      </button>
    </div>
  {/if}
</aside>
