/// Auth store — tracks Supabase session/user state reactively.
/// Initialises from existing session on load; listens for auth state changes.
import { writable } from 'svelte/store';
import type { User, Session } from '@supabase/supabase-js';
import { supabase } from './supabase-client.js';

interface AuthState {
  user: User | null;
  session: Session | null;
  loading: boolean;
}

const initialState: AuthState = { user: null, session: null, loading: true };

export const authStore = writable<AuthState>(initialState);

// Hydrate from existing session (e.g. page refresh)
supabase.auth.getSession().then(({ data }) => {
  authStore.set({
    user: data.session?.user ?? null,
    session: data.session ?? null,
    loading: false,
  });
});

// React to login/logout events
supabase.auth.onAuthStateChange((_event, session) => {
  authStore.set({
    user: session?.user ?? null,
    session: session ?? null,
    loading: false,
  });
});

/** Sign out the current user and clear local session. */
export async function signOut(): Promise<void> {
  await supabase.auth.signOut();
  // store is updated via onAuthStateChange above
}
