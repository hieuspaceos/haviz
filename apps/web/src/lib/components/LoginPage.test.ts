/**
 * Tests for LoginPage.svelte — verifies form fields and toggle behavior.
 * Stubs Supabase client to avoid real auth calls.
 */
import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, cleanup } from '@testing-library/svelte';

// Stub supabase before component import to avoid real network
vi.mock('../auth/supabase-client.js', () => ({
  supabase: {
    auth: {
      signInWithPassword: vi.fn().mockResolvedValue({ error: null }),
      signUp: vi.fn().mockResolvedValue({ error: null }),
    },
  },
}));

const { default: LoginPage } = await import('./LoginPage.svelte');

// Unmount and clear DOM after each test
afterEach(() => cleanup());

describe('LoginPage', () => {
  it('renders email input field', () => {
    const { getByPlaceholderText } = render(LoginPage);
    expect(getByPlaceholderText('Email')).toBeTruthy();
  });

  it('renders password input field', () => {
    const { getByPlaceholderText } = render(LoginPage);
    expect(getByPlaceholderText('Password')).toBeTruthy();
  });

  it('renders submit button with sign in label in login mode', () => {
    const { getByRole } = render(LoginPage);
    expect(getByRole('button', { name: 'Sign in' })).toBeTruthy();
  });

  it('shows signup toggle link', () => {
    const { getByRole } = render(LoginPage);
    // The toggle button at the bottom reads "Sign up"
    const buttons = document.querySelectorAll('button[type="button"]');
    expect(buttons.length).toBeGreaterThan(0);
    expect(buttons[0].textContent?.trim()).toBe('Sign up');
  });

  it('renders the Haviz brand heading', () => {
    const { getByRole } = render(LoginPage);
    expect(getByRole('heading', { name: 'Haviz' })).toBeTruthy();
  });
});
