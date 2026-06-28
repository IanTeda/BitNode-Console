import { createContext, useContext } from "react";

export type AuthContext = {
  isAuthenticated: boolean;
  accessToken: string | undefined;
  handleLogin: (password: string) => Promise<void>;
  handleLogout: () => Promise<void>;
};

export const AuthenticationContext = createContext<AuthContext | undefined>(undefined);

export function useAuthentication(): AuthContext {
  const context = useContext(AuthenticationContext);
  if (!context) {
    throw new Error("useAuthentication must be used inside AuthenticationProvider");
  }
  return context;
}
