import { type PropsWithChildren, useState } from "react";
import { authenticationClient } from "@/lib/rpc/authentication";
import { setAccessToken as setRpcToken } from "@/lib/rpc/utilities";
import { getCookie, setCookie, deleteCookie, jwtExpiry } from "@/lib/cookies";
import { AuthenticationContext } from "@/lib/auth-context";
import logger from "@/lib/logger";

export type { AuthContext } from "@/lib/auth-context";

const log = logger.getSubLogger({ name: "AuthProvider" });

// Token storage strategy:
//  - Access token  → React state (memory) only. Never written to a cookie so
//    it is not readable via document.cookie by injected scripts.
//  - Refresh token → cookie (persistent). Reissues the access token on page
//    load via handleRefresh(), called from the login route's beforeLoad.

export default function AuthenticationProvider({ children }: PropsWithChildren) {
  const [accessToken, setAccessTokenState] = useState<string | undefined>(undefined);

  const isAuthenticated = !!accessToken;

  async function handleLogin(password: string): Promise<void> {
    log.debug("Attempting login");
    try {
      const { response } = await authenticationClient().login({ password });
      setCookie("refresh_token", response.refreshToken, jwtExpiry(response.refreshToken));
      setRpcToken(response.accessToken);
      setAccessTokenState(response.accessToken);
      log.info("Login successful");
    } catch (error) {
      log.error("Login failed:", error);
      throw error;
    }
  }

  async function handleLogout(): Promise<void> {
    log.debug("Logging out");
    deleteCookie("refresh_token");
    setRpcToken(undefined);
    setAccessTokenState(undefined);
    log.info("Logged out");
  }

  async function handleRefresh(): Promise<boolean> {
    const refreshToken = getCookie("refresh_token");
    if (!refreshToken) {
      log.debug("No refresh token cookie found");
      return false;
    }
    log.debug("Attempting token refresh");
    try {
      const { response } = await authenticationClient().refresh({ refreshToken });
      setCookie("refresh_token", response.refreshToken, jwtExpiry(response.refreshToken));
      setRpcToken(response.accessToken);
      setAccessTokenState(response.accessToken);
      log.info("Token refresh successful");
      return true;
    } catch (error) {
      log.warn("Token refresh failed, clearing stale cookie:", error);
      deleteCookie("refresh_token");
      return false;
    }
  }

  return (
    <AuthenticationContext.Provider
      value={{ isAuthenticated, accessToken, handleLogin, handleLogout, handleRefresh }}
    >
      {children}
    </AuthenticationContext.Provider>
  );
}
