<script lang="ts">
  import { logs } from '../stores/app';
</script>

<div class="border-t border-[var(--border)] bg-[var(--bg-secondary)] h-36 overflow-y-auto px-3 py-2">
  <div class="text-[10px] font-semibold text-[var(--text-secondary)] uppercase tracking-wide mb-1">Log</div>
  {#each $logs as log}
    <div class="text-xs font-mono" class:text-[var(--success)]={log.type === 'ok'} class:text-[var(--danger)]={log.type === 'err'}>
      <span class="text-[var(--text-secondary)]">{log.time}</span> {log.msg}
    </div>
  {/each}
  {#if $logs.length === 0}
    <div class="text-xs text-[var(--text-secondary)]">No activity yet</div>
  {/if}
</div>
