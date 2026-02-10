/**
 * Lazy load Tauri invoke command
 * Compatible with both Tauri and browser environments
 */
export async function invokeCommand<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}
