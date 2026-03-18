<script lang="ts">
  import { api } from '../api/client';
  import { agentStatus, screenshotUrl, addLog } from '../stores/app';

  async function checkStatus() {
    try {
      const data = await api.status();
      $agentStatus = { ok: data.ok, version: data.version };
    } catch {
      $agentStatus = null;
    }
  }

  async function takeScreenshot() {
    try {
      addLog('Taking screenshot...');
      const blob = await api.screenshot();
      $screenshotUrl = URL.createObjectURL(blob);
      addLog('Screenshot captured');
    } catch (e) {
      addLog('Screenshot failed', 'err');
    }
  }

  checkStatus();
  setInterval(checkStatus, 10000);
</script>

<header class="h-12 flex items-center justify-between px-4 border-b border-[var(--border)] bg-[var(--bg-secondary)]">
  <div class="flex items-center gap-3">
    <h1 class="text-base font-bold text-[var(--accent)]">Haviz</h1>
    <span class="text-[10px] text-[var(--text-secondary)]">Revenue Intelligence</span>
  </div>

  <div class="flex items-center gap-2">
    <button
      class="px-3 py-1.5 text-xs rounded bg-[var(--bg-tertiary)] hover:bg-[var(--border)] transition-colors"
      on:click={takeScreenshot}
    >
      📸 Screenshot
    </button>

    {#if $agentStatus}
      <span class="px-2 py-1 text-[10px] rounded-full bg-[var(--success)] text-white">
        Agent v{$agentStatus.version}
      </span>
    {:else}
      <span class="px-2 py-1 text-[10px] rounded-full bg-[var(--danger)] text-white">
        Offline
      </span>
    {/if}
  </div>
</header>
