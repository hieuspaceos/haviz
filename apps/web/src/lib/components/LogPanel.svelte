<script lang="ts">
  import { logs } from '../stores/app';

  let collapsed = false;
</script>

<div class="log-panel" class:log-panel--collapsed={collapsed}>
  <div class="log-header">
    <div class="log-title">
      <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14 2 14 8 20 8"/>
        <line x1="16" y1="13" x2="8" y2="13"/>
        <line x1="16" y1="17" x2="8" y2="17"/>
      </svg>
      <span class="section-label">Activity Log</span>
      {#if $logs.length > 0}
        <span class="log-count">{$logs.length}</span>
      {/if}
    </div>
    <button class="log-toggle" on:click={() => collapsed = !collapsed} title={collapsed ? 'Expand log' : 'Collapse log'}>
      {#if collapsed}
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="18 15 12 9 6 15"/></svg>
      {:else}
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"/></svg>
      {/if}
    </button>
  </div>

  {#if !collapsed}
    <div class="log-body">
      {#if $logs.length === 0}
        <div class="log-empty">No activity yet</div>
      {:else}
        {#each $logs as log}
          <div class="log-entry">
            <span
              class="log-dot"
              class:log-dot--ok={log.type === 'ok'}
              class:log-dot--err={log.type === 'err'}
              class:log-dot--info={log.type !== 'ok' && log.type !== 'err'}
            ></span>
            <span class="log-time">{log.time}</span>
            <span
              class="log-msg"
              class:log-msg--ok={log.type === 'ok'}
              class:log-msg--err={log.type === 'err'}
            >{log.msg}</span>
          </div>
        {/each}
      {/if}
    </div>
  {/if}
</div>

<style>
  .log-panel {
    border-top: 1px solid var(--border-subtle);
    background: var(--bg-secondary);
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    transition: height 200ms ease;
  }

  .log-panel:not(.log-panel--collapsed) {
    height: 140px;
  }

  .log-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .log-title {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text-secondary);
  }

  .log-count {
    font-size: 10px;
    font-weight: 600;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    color: var(--text-secondary);
    padding: 1px 6px;
    border-radius: 99px;
    line-height: 1.4;
  }

  .log-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: var(--text-secondary);
    cursor: pointer;
    transition: color 200ms, background 200ms;
  }
  .log-toggle:hover { color: var(--text-primary); background: var(--bg-tertiary); }

  .log-body {
    flex: 1;
    overflow-y: auto;
    padding: 6px 12px;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .log-entry {
    display: flex;
    align-items: center;
    gap: 7px;
    font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', ui-monospace, monospace;
    font-size: 11px;
    line-height: 1.5;
  }

  .log-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .log-dot--ok { background: var(--success); box-shadow: 0 0 4px var(--success); }
  .log-dot--err { background: var(--danger); box-shadow: 0 0 4px var(--danger); }
  .log-dot--info { background: var(--text-secondary); }

  .log-time {
    color: var(--text-secondary);
    flex-shrink: 0;
    opacity: 0.7;
  }

  .log-msg { color: var(--text-primary); }
  .log-msg--ok { color: var(--success); }
  .log-msg--err { color: var(--danger); }

  .log-empty {
    font-size: 11px;
    color: var(--text-secondary);
    font-family: ui-monospace, monospace;
    padding: 4px 0;
  }
</style>
