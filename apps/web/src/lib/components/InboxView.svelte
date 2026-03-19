<script lang="ts">
  import './inbox-view.css';
  import { icon } from './icons';
  import { api } from '../api/client';
  import { addLog, screenshotUrl } from '../stores/app';

  let searchQuery = '';
  let searchResults: any[] = [];
  let messageText = '';
  let sending = false;
  let chatMessages: any[] = [];
  let loadingMessages = false;
  let aiDraft = '';
  let aiLoading = false;

  async function search() {
    if (!searchQuery.trim()) return;
    addLog(`Searching: ${searchQuery}`);
    try {
      const data = await api.zalo.search(searchQuery);
      searchResults = data.conversations?.filter((c: any) => c.name?.trim()) || [];
      addLog(`${searchResults.length} results`);
    } catch (e) { addLog('Search failed', 'err'); }
  }

  async function openResult(index: number) {
    try {
      await api.zalo.open(index);
      addLog(`Opened #${index}`);
      searchResults = []; searchQuery = '';
    } catch (e) { addLog('Open failed', 'err'); }
  }

  async function sendMessage() {
    if (!messageText.trim() || sending) return;
    if (!searchQuery.trim()) { addLog('Nhập tên người nhận trong Search trước', 'err'); return; }
    sending = true;
    addLog(`Sending to ${searchQuery}: ${messageText.slice(0, 40)}...`);
    try {
      await api.zalo.searchAndSend(searchQuery, messageText);
      addLog('Sent!'); messageText = '';
    } catch (e) { addLog('Send failed', 'err'); }
    sending = false;
  }

  async function approveDraft() {
    if (!aiDraft.trim() || !searchQuery.trim() || sending) return;
    sending = true;
    addLog(`Approving & sending to ${searchQuery}...`);
    try {
      await api.zalo.searchAndSend(searchQuery, aiDraft);
      addLog('Sent!'); aiDraft = '';
    } catch (e) { addLog('Send failed', 'err'); }
    sending = false;
  }

  async function generateDraft() {
    if (chatMessages.length === 0) { addLog('Load Messages first', 'err'); return; }
    aiLoading = true;
    addLog('Generating AI draft...');
    try {
      const msgs = chatMessages
        .filter(m => m.content && m.content.length > 1)
        .map(m => ({
          sender: m.sender || m.class || 'Unknown',
          content: m.content,
          direction: (m.class || '').includes('conv-dbname') ? 'outbound' : 'inbound',
        }));
      const data = await api.ai.draft(msgs);
      if (data.ok && data.draft) {
        aiDraft = data.draft; messageText = data.draft; addLog('AI Draft ready!');
      } else { addLog('AI Draft failed: ' + (data.error || 'unknown'), 'err'); }
    } catch (e) { addLog('AI Draft error', 'err'); }
    aiLoading = false;
  }

  async function loadMessages() {
    loadingMessages = true;
    addLog('Loading messages...');
    try {
      const data = await api.zalo.messages();
      chatMessages = data.messages || [];
      addLog(`${chatMessages.length} messages loaded`);
    } catch (e) { addLog('Load messages failed', 'err'); }
    loadingMessages = false;
  }

  function handleKey(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); sendMessage(); }
  }

  function isOutbound(msg: any): boolean {
    return (msg.class || '').includes('conv-dbname');
  }

  function focusDraft() {
    (document.querySelector('#draftTextarea') as HTMLElement)?.focus();
    addLog('Editing draft...');
  }
</script>

<div class="inbox">

  <section class="inbox-card">
    <div class="card-header">{@html icon.search}<span class="section-label">Search User</span></div>
    <div class="search-row">
      <input type="text" bind:value={searchQuery} placeholder="Enter recipient name..."
        class="input-base" on:keydown={(e) => e.key === 'Enter' && search()} />
      <button class="btn btn-primary" on:click={search}>Search</button>
    </div>
    {#if searchResults.length > 0}
      <div class="search-results">
        {#each searchResults as r, i}
          <button class="search-result-item" on:click={() => openResult(i + 1)}>
            <div class="result-name">{r.name}</div>
            {#if r.preview}<div class="result-preview">{r.preview}</div>{/if}
          </button>
        {/each}
      </div>
    {/if}
  </section>

  <section class="inbox-card">
    <div class="card-header">
      {@html icon.send}<span class="section-label">Send Message</span>
      <span class="card-hint">Search user above first</span>
    </div>
    <textarea bind:value={messageText} placeholder="Type message... (Enter to send, Shift+Enter for newline)"
      rows="3" class="input-base textarea" on:keydown={handleKey}></textarea>
    <div class="card-actions">
      <button class="btn btn-success" disabled={sending || !messageText.trim()} on:click={sendMessage}>
        {#if sending}{@html icon.spinner} Sending...{:else}Send{/if}
      </button>
    </div>
  </section>

  <section class="inbox-card ai-card">
    <div class="card-header">
      {@html icon.layers}<span class="section-label" style="color:var(--purple)">AI Draft Reply</span>
      <button class="btn btn-purple btn-sm" disabled={aiLoading || chatMessages.length === 0} on:click={generateDraft}>
        {aiLoading ? 'Generating...' : 'Generate Draft'}
      </button>
    </div>
    {#if aiDraft}
      <div class="ai-draft-box">
        <textarea id="draftTextarea" bind:value={aiDraft} rows="3" class="input-base textarea ai-draft-input"></textarea>
        <div class="card-actions">
          <button class="btn btn-success btn-sm" disabled={sending} on:click={approveDraft}>{sending ? 'Sending...' : 'Approve & Send'}</button>
          <button class="btn btn-warning btn-sm" on:click={focusDraft}>Edit</button>
          <button class="btn btn-purple btn-sm" disabled={aiLoading} on:click={generateDraft}>{aiLoading ? '...' : 'Regenerate'}</button>
          <button class="btn btn-danger btn-sm" on:click={() => { aiDraft = ''; addLog('Draft rejected'); }}>Reject</button>
        </div>
      </div>
    {:else if chatMessages.length === 0}
      <p class="hint-text">Load messages first, then generate a draft.</p>
    {/if}
  </section>

  <section class="inbox-card">
    <div class="card-header">
      {@html icon.chat}<span class="section-label">Chat Messages</span>
      <button class="btn btn-primary btn-sm" disabled={loadingMessages} on:click={loadMessages}>
        {loadingMessages ? 'Loading...' : 'Load Messages'}
      </button>
    </div>
    {#if chatMessages.length > 0}
      <div class="chat-window">
        {#each chatMessages as msg}
          <div class="bubble-row" class:bubble-row--out={isOutbound(msg)}>
            <div class="bubble" class:bubble--out={isOutbound(msg)} class:bubble--in={!isOutbound(msg)}>
              {#if msg.sender && !isOutbound(msg)}<span class="bubble-sender">{msg.sender}</span>{/if}
              <div class="bubble-content">{msg.content}</div>
              {#if msg.time}<span class="bubble-time">{msg.time}</span>{/if}
            </div>
          </div>
        {/each}
      </div>
    {:else if !loadingMessages}
      <p class="hint-text">Open a conversation in Zalo sidebar, then click "Load Messages".</p>
    {/if}
  </section>

  {#if $screenshotUrl}
    <section class="inbox-card">
      <div class="card-header">{@html icon.camera}<span class="section-label">Screenshot</span></div>
      <img src={$screenshotUrl} alt="Screenshot" class="screenshot-img" />
    </section>
  {/if}

</div>
