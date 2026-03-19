import type { Context, Next } from 'hono';

/**
 * Global error handler middleware.
 * Catches all unhandled errors and returns structured JSON.
 */
export async function errorHandler(c: Context, next: Next) {
  try {
    await next();
  } catch (error) {
    console.error('[ErrorHandler]', error);

    const message =
      error instanceof Error ? error.message : 'Internal server error';

    return c.json({ ok: false, error: message }, 500);
  }
}
