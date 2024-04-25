export function inSecureEnvironment(): boolean {
  return location.protocol !== 'http:';
}
