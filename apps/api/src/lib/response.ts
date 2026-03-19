/**
 * Uniform JSON response helpers for all Hono route handlers.
 * All API responses follow { ok, data } or { ok, error } shape.
 */

export function ok<T>(data: T) {
  return { ok: true as const, data };
}

export function err(message: string, status: number = 400) {
  return new Response(
    JSON.stringify({ ok: false, error: message }),
    {
      status,
      headers: { 'Content-Type': 'application/json' },
    }
  );
}
