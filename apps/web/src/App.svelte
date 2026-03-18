<script lang="ts">
  import Topbar from './lib/components/Topbar.svelte';
  import Sidebar from './lib/components/Sidebar.svelte';
  import InboxView from './lib/components/InboxView.svelte';
  import LogPanel from './lib/components/LogPanel.svelte';
</script>

<div class="h-screen flex flex-col">
  <Topbar />
  <div class="flex flex-1 overflow-hidden">
    <Sidebar />
    <main class="flex-1 flex flex-col overflow-hidden">
      <InboxView />
      <LogPanel />
    </main>
  </div>
</div>
