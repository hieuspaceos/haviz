<script lang="ts">
  import { api } from '../api/client';
  import { addLog, screenshotUrl } from '../stores/app';

  let searchQuery = '';
  let searchResults: any[] = [];
  let messageText = '';
  let sending = false;

  async function search() {
    if (!searchQuery.trim()) return;
    addLog(`Searching: ${searchQuery}`);
    try {
      const data = await api.zalo.search(searchQuery);
      searchResults = data.conversations?.filter((c: any) => c.name?.trim()) || [];
      addLog(`${searchResults.length} results`);
    } catch (e) {
      addLog('Search failed', 'err');
    }
  }

  async function openResult(index: number) {
    try {
      await api.zalo.open(index);
      addLog(`Opened #${index}`);
      searchResults = [];
      searchQuery = '';
    } catch (e) {
      addLog('Open failed', 'err');
    }
  }

  async function sendMessage() {
    if (!messageText.trim() || sending) return;
    sending = true;
    addLog(`Sending: ${messageText.slice(0, 40)}...`);
    try {
      await api.zalo.send(messageText);
      addLog('Sent!');
      messageText = '';
    } catch (e) {
      addLog('Send failed', 'err');
    }
    sending = false;
  }

  function handleKey(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  }
</script>

<div class="flex-1 flex flex-col p-4 gap-4 overflow-y-auto">
  <!-- Search -->
  <div>
    <h3 class="text-xs font-semibold text-[var(--text-secondary)] mb-2 uppercase tracking-wide">Search User</h3>
    <div class="flex gap-2">
      <input
        type="text"
        bind:value={searchQuery}
        placeholder="Nhập tên cần tìm..."
        class="flex-1 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-secondary)] focus:border-[var(--accent)] focus:outline-none"
        on:keydown={(e) => e.key === 'Enter' && search()}
      />
      <button class="px-4 py-2 text-sm font-medium rounded-lg bg-[var(--accent)] text-white hover:opacity-90" on:click={search}>
        Search
      </button>
    </div>

    {#if searchResults.length > 0}
      <div class="mt-2 border border-[var(--border)] rounded-lg overflow-hidden">
        {#each searchResults as r, i}
          <button
            class="w-full text-left px-3 py-2 hover:bg-[var(--bg-tertiary)] transition-colors border-b border-[var(--border)] last:border-0"
            on:click={() => openResult(i + 1)}
          >
            <div class="text-sm font-medium">{r.name}</div>
            {#if r.preview}<div class="text-xs text-[var(--text-secondary)]">{r.preview}</div>{/if}
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Send Message -->
  <div>
    <h3 class="text-xs font-semibold text-[var(--text-secondary)] mb-2 uppercase tracking-wide">Send Message (current conversation)</h3>
    <textarea
      bind:value={messageText}
      placeholder="Nhập tin nhắn... (Enter để gửi)"
      rows="3"
      class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-secondary)] focus:border-[var(--accent)] focus:outline-none resize-none"
      on:keydown={handleKey}
    ></textarea>
    <button
      class="mt-2 px-4 py-2 text-sm font-medium rounded-lg bg-[var(--success)] text-white hover:opacity-90 disabled:opacity-50"
      disabled={sending || !messageText.trim()}
      on:click={sendMessage}
    >
      {sending ? 'Sending...' : 'Send'}
    </button>
  </div>

  <!-- Screenshot Preview -->
  {#if $screenshotUrl}
    <div>
      <h3 class="text-xs font-semibold text-[var(--text-secondary)] mb-2 uppercase tracking-wide">Screenshot</h3>
      <img src={$screenshotUrl} alt="Screenshot" class="w-full rounded-lg border border-[var(--border)]" />
    </div>
  {/if}
</div>
