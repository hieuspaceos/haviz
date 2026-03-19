/// Shared Hono context variable types.
/// Import `AppVariables` wherever a typed `Hono` or `Context` instance is needed.
import type { User } from '@supabase/supabase-js';

export type AppVariables = {
  user: User;
};
