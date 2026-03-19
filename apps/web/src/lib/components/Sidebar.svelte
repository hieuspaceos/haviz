<script lang="ts">
  import { api } from '../api/client';
  import { zaloConversations, addLog } from '../stores/app';

  let activeTab = 'inbox';
  const tabs = [
    { id: 'inbox', label: 'Inbox', icon: '💬' },
    { id: 'contacts', label: 'Contacts', icon: '👥' },
    { id: 'templates', label: 'Templates', icon: '📋' },
    { id: 'settings', label: 'Settings', icon: '⚙️' },
  ];

  export let onNavigate: (path: string) => void = () => {};

  async function refresh() {
    try {
      // Try Zalo WebView IPC first, fallback to local SQLite
      let data: any;
      try {
        data = await api.zalo.conversations();
      } catch {
        data = await api.conversations();
      }
      $zaloConversations = data.conversations?.filter((c: any) => c.name?.trim() || c.contact_name?.trim())
        .map((c: any) => ({ name: c.name || c.contact_name, time: c.time || '', preview: c.preview || c.last_message_preview || '', sender: c.sender || '' })) || [];
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

  refresh();
  const interval = setInterval(refresh, 15000);
</script>

<aside class="w-72 h-full flex flex-col border-r border-[var(--border)] bg-[var(--bg-secondary)]">
  <!-- Nav tabs -->
  <nav class="flex border-b border-[var(--border)]">
    {#each tabs as tab}
      <button
        class="flex-1 py-3 text-xs text-center transition-colors"
        class:text-[var(--accent)]={activeTab === tab.id}
        class:text-[var(--text-secondary)]={activeTab !== tab.id}
        on:click={() => { activeTab = tab.id; onNavigate('/' + tab.id); }}
      >
        <div class="text-lg">{tab.icon}</div>
        {tab.label}
      </button>
    {/each}
  </nav>

  <!-- Conversation list -->
  <div class="flex-1 overflow-y-auto">
    {#each $zaloConversations as conv, i}
      <button
        class="w-full text-left px-3 py-2.5 border-b border-[var(--border)] hover:bg-[var(--bg-tertiary)] transition-colors"
        on:click={() => openConv(i + 1)}
      >
        <div class="flex justify-between items-baseline">
          <span class="text-sm font-medium truncate flex-1">{conv.name}</span>
          <span class="text-[10px] text-[var(--text-secondary)] ml-2 whitespace-nowrap">{conv.time}</span>
        </div>
        {#if conv.preview}
          <div class="text-xs text-[var(--text-secondary)] truncate mt-0.5">
            {#if conv.sender}<span class="text-[var(--text-primary)]">{conv.sender}:</span>{/if}
            {conv.preview}
          </div>
        {/if}
      </button>
    {/each}
    {#if $zaloConversations.length === 0}
      <div class="p-4 text-sm text-[var(--text-secondary)] text-center">
        No conversations.<br>Is Zalo Web loaded?
      </div>
    {/if}
  </div>

  <!-- Refresh button -->
  <div class="p-2 border-t border-[var(--border)]">
    <button class="w-full py-2 text-xs rounded bg-[var(--bg-tertiary)] hover:bg-[var(--border)] transition-colors" on:click={refresh}>
      Refresh
    </button>
  </div>
</aside>
