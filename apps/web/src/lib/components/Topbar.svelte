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

<header class="topbar">
  <div class="topbar-brand">
    <span class="topbar-logo">Haviz</span>
    <span class="topbar-tagline">Revenue Intelligence</span>
  </div>

  <div class="topbar-actions">
    <button class="btn btn-ghost btn-sm" on:click={takeScreenshot} title="Take screenshot">
      <!-- Camera SVG icon -->
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"/>
        <circle cx="12" cy="13" r="4"/>
      </svg>
      Screenshot
    </button>

    {#if $agentStatus}
      <span class="status-badge status-online">
        <span class="status-dot"></span>
        Agent v{$agentStatus.version}
      </span>
    {:else}
      <span class="status-badge status-offline">
        <span class="status-dot"></span>
        Offline
      </span>
    {/if}
  </div>
</header>

<style>
  .topbar {
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-subtle);
    box-shadow: 0 1px 0 var(--border-subtle), 0 2px 8px rgba(0,0,0,0.3);
    flex-shrink: 0;
    z-index: 10;
  }

  .topbar-brand {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }

  .topbar-logo {
    font-size: 16px;
    font-weight: 700;
    color: var(--accent);
    letter-spacing: -0.02em;
    text-shadow: 0 0 20px var(--accent-glow);
  }

  .topbar-tagline {
    font-size: 10px;
    color: var(--text-secondary);
    letter-spacing: 0.04em;
  }

  .topbar-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .status-badge {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 4px 10px;
    font-size: 10px;
    font-weight: 500;
    border-radius: 99px;
    border: 1px solid transparent;
  }

  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .status-online {
    background: rgba(63, 185, 80, 0.12);
    border-color: rgba(63, 185, 80, 0.3);
    color: var(--success);
  }
  .status-online .status-dot {
    background: var(--success);
    box-shadow: 0 0 6px var(--success);
  }

  .status-offline {
    background: rgba(248, 81, 73, 0.12);
    border-color: rgba(248, 81, 73, 0.3);
    color: var(--danger);
  }
  .status-offline .status-dot { background: var(--danger); }
</style>
