import {
  createContext,
  type PropsWithChildren,
  useContext,
  useState,
} from "react";
import { authenticationClient } from "@/lib/rpc/authentication";
import { setAccessToken as setRpcToken } from "@/lib/rpc/utilities";
import { getCookie, setCookie, deleteCookie, jwtExpiry } from "@/lib/cookies";
import logger from "@/lib/logger";

const log = logger.getSubLogger({ name: "AuthProvider" });

export type AuthContext = {
  isAuthenticated: boolean;
  accessToken: string | undefined;
  handleLogin: (password: string) => Promise<void>;
  handleLogout: () => Promise<void>;
};

const AuthenticationContext = createContext<AuthContext | undefined>(undefined);

export default function AuthenticationProvider({
  children,
}: PropsWithChildren) {
  const [accessToken, setAccessTokenState] = useState<string | undefined>(() =>
    getCookie("access_token"),
  );

  const isAuthenticated = !!accessToken;

  async function handleLogin(password: string): Promise<void> {
    log.debug("Attempting login");
    try {
      const { response } = await authenticationClient().login({ password });
      setCookie(
        "access_token",
        response.accessToken,
        jwtExpiry(response.accessToken),
      );
      setCookie(
        "refresh_token",
        response.refreshToken,
        jwtExpiry(response.refreshToken),
      );
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
    deleteCookie("access_token");
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

export function useAuthentication(): AuthContext {
  const context = useContext(AuthenticationContext);
  if (!context) {
    throw new Error(
      "useAuthentication must be used inside AuthenticationProvider",
    );
  }
  return context;
}
