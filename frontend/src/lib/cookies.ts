/// Returns the value of a named cookie, or `undefined` if it is not set.
export function getCookie(name: string): string | undefined {
  const row = document.cookie.split("; ").find((r) => r.startsWith(`${name}=`));
  return row?.slice(name.length + 1);
}

/// Sets a cookie with `path=/` and `SameSite=Strict`. Pass `expires` to make
/// it persistent; omit it for a session cookie.
export function setCookie(name: string, value: string, expires?: Date): void {
  const parts = [`${name}=${value}`, "path=/", "SameSite=Strict"];
  if (expires) parts.push(`expires=${expires.toUTCString()}`);
  document.cookie = parts.join("; ");
}

/// Deletes a cookie by setting its expiry to the Unix epoch.
export function deleteCookie(name: string): void {
  document.cookie = `${name}=; path=/; SameSite=Strict; expires=Thu, 01 Jan 1970 00:00:00 GMT`;
}

/// Decodes a JWT and returns its `exp` claim as a `Date`, or `undefined` if
/// the token is missing the claim or cannot be decoded.
export function jwtExpiry(token: string): Date | undefined {
  try {
    const { exp } = JSON.parse(atob(token.split(".")[1] ?? "")) as { exp?: number };
    return typeof exp === "number" ? new Date(exp * 1000) : undefined;
  } catch {
    return undefined;
  }
}
