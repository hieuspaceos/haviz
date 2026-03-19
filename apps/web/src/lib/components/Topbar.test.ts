/**
 * Tests for Topbar.svelte — verifies title text and status indicator render.
 * Stubs api client and stores to avoid real network calls.
 */
import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, cleanup } from '@testing-library/svelte';
import { writable } from 'svelte/store';

// Keep a reference to the store so we can control its value per-test
const agentStatusStore = writable<{ ok: boolean; version: string } | null>(null);

vi.mock('../stores/app', () => ({
  agentStatus: agentStatusStore,
  screenshotUrl: writable(null),
  addLog: vi.fn(),
}));

// Prevent api.status from resolving and updating the store during tests
vi.mock('../api/client', () => ({
  api: {
    status: vi.fn().mockReturnValue(new Promise(() => { /* never resolves */ })),
    screenshot: vi.fn().mockReturnValue(new Promise(() => {})),
  },
}));

const { default: Topbar } = await import('./Topbar.svelte');

afterEach(() => {
  cleanup();
  agentStatusStore.set(null);
});

describe('Topbar', () => {
  it('renders the Haviz brand title', () => {
    const { getByText } = render(Topbar);
    expect(getByText('Haviz')).toBeTruthy();
  });

  it('renders the Revenue Intelligence subtitle', () => {
    const { getByText } = render(Topbar);
    expect(getByText('Revenue Intelligence')).toBeTruthy();
  });

  it('shows Offline status when agentStatus is null', () => {
    agentStatusStore.set(null);
    const { getByText } = render(Topbar);
    expect(getByText('Offline')).toBeTruthy();
  });

  it('shows Agent version badge when agentStatus is set', async () => {
    agentStatusStore.set({ ok: true, version: '0.1.0' });
    const { getByText } = render(Topbar);
    expect(getByText('Agent v0.1.0')).toBeTruthy();
  });
});
