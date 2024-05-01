export type Empty = Record<string, never>;

export function inSecureEnvironment(): boolean {
  return location.protocol !== 'http:';
}
