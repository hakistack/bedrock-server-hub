import type { AppError } from '$lib/types/server';

/** Extract a human-readable message from anything a command might throw. */
export function errorMessage(err: unknown): string {
  if (err && typeof err === 'object' && 'message' in err) {
    return String((err as AppError).message);
  }
  if (typeof err === 'string') return err;
  return 'Ocurrió un error inesperado.';
}
