<script lang="ts">
  import Topbar from './lib/components/Topbar.svelte';
  import Sidebar from './lib/components/Sidebar.svelte';
  import InboxView from './lib/components/InboxView.svelte';
  import LogPanel from './lib/components/LogPanel.svelte';
  import LoginPage from './lib/components/LoginPage.svelte';
  import { authStore } from './lib/auth/auth-guard.js';
</script>

{#if $authStore.loading}
  <!-- Blank screen while session is being checked — avoids flash of login -->
  <div class="h-screen bg-[var(--bg-primary)]"></div>
{:else if !$authStore.user}
  <LoginPage />
{:else}
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
{/if}
