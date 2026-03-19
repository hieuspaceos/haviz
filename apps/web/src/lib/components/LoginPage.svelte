<script lang="ts">
  import { supabase } from '../auth/supabase-client.js';

  let email = '';
  let password = '';
  let fullName = '';
  let mode: 'login' | 'signup' = 'login';
  let errorMsg = '';
  let loading = false;

  async function handleSubmit() {
    errorMsg = '';
    loading = true;
    try {
      if (mode === 'login') {
        const { error } = await supabase.auth.signInWithPassword({ email, password });
        if (error) errorMsg = error.message;
      } else {
        const { error } = await supabase.auth.signUp({
          email,
          password,
          options: { data: { full_name: fullName } },
        });
        if (error) errorMsg = error.message;
      }
    } catch (e) {
      errorMsg = 'Unexpected error. Please try again.';
    } finally {
      loading = false;
    }
  }

  function toggleMode() {
    mode = mode === 'login' ? 'signup' : 'login';
    errorMsg = '';
  }
</script>

<div class="h-screen flex items-center justify-center bg-[var(--bg-primary)]">
  <div class="w-full max-w-sm p-8 rounded-xl bg-[var(--bg-secondary)] border border-[var(--border)] shadow-xl">
    <!-- Header -->
    <div class="mb-6 text-center">
      <h1 class="text-2xl font-bold text-[var(--accent)]">Haviz</h1>
      <p class="text-xs text-[var(--text-secondary)] mt-1">Revenue Intelligence</p>
    </div>

    <h2 class="text-sm font-semibold text-[var(--text-primary)] mb-4">
      {mode === 'login' ? 'Sign in to your account' : 'Create an account'}
    </h2>

    <form on:submit|preventDefault={handleSubmit} class="flex flex-col gap-3">
      {#if mode === 'signup'}
        <input
          type="text"
          placeholder="Full name"
          bind:value={fullName}
          class="w-full px-3 py-2 rounded bg-[var(--bg-tertiary)] border border-[var(--border)] text-sm text-[var(--text-primary)] placeholder-[var(--text-secondary)] focus:outline-none focus:border-[var(--accent)]"
        />
      {/if}

      <input
        type="email"
        placeholder="Email"
        required
        bind:value={email}
        class="w-full px-3 py-2 rounded bg-[var(--bg-tertiary)] border border-[var(--border)] text-sm text-[var(--text-primary)] placeholder-[var(--text-secondary)] focus:outline-none focus:border-[var(--accent)]"
      />

      <input
        type="password"
        placeholder="Password"
        required
        bind:value={password}
        class="w-full px-3 py-2 rounded bg-[var(--bg-tertiary)] border border-[var(--border)] text-sm text-[var(--text-primary)] placeholder-[var(--text-secondary)] focus:outline-none focus:border-[var(--accent)]"
      />

      {#if errorMsg}
        <p class="text-xs text-[var(--danger)]">{errorMsg}</p>
      {/if}

      <button
        type="submit"
        disabled={loading}
        class="mt-1 w-full py-2 rounded bg-[var(--accent)] text-white text-sm font-medium hover:opacity-90 disabled:opacity-50 transition-opacity"
      >
        {loading ? 'Please wait...' : mode === 'login' ? 'Sign in' : 'Sign up'}
      </button>
    </form>

    <p class="mt-4 text-center text-xs text-[var(--text-secondary)]">
      {mode === 'login' ? "Don't have an account?" : 'Already have an account?'}
      <button
        type="button"
        on:click={toggleMode}
        class="ml-1 text-[var(--accent)] hover:underline"
      >
        {mode === 'login' ? 'Sign up' : 'Sign in'}
      </button>
    </p>
  </div>
</div>
