import { type PropsWithChildren, useState } from "react";
import { authenticationClient } from "@/lib/rpc/authentication";
import { setAccessToken as setRpcToken } from "@/lib/rpc/utilities";
import { setCookie, deleteCookie, jwtExpiry } from "@/lib/cookies";
import { AuthenticationContext } from "@/lib/auth-context";
import logger from "@/lib/logger";

export type { AuthContext } from "@/lib/auth-context";

const log = logger.getSubLogger({ name: "AuthProvider" });

// Token storage strategy:
//  - Access token  → React state (memory) only. Never written to a cookie so
//    it is not readable via document.cookie by injected scripts.
//  - Refresh token → cookie (persistent). Used to reissue the access token on
//    page load once the backend implements the Refresh RPC.
//
// TODO: on mount, call authenticationClient().refresh() with the refresh_token
//       cookie to silently restore the session. Until the backend implements
//       Refresh, the user must log in again after every page reload.

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

  return (
    <AuthenticationContext.Provider
      value={{ isAuthenticated, accessToken, handleLogin, handleLogout }}
    >
      {children}
    </AuthenticationContext.Provider>
  );
}
