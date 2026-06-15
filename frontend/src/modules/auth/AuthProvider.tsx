import { createContext, useContext, useEffect, useMemo, useState } from "react";
import { api, clearTokens, getAccessToken, getRefreshToken, setTokens } from "../../lib/api";
import type { CurrentUser } from "../../lib/types";

type AuthContextValue = {
  isAuthenticated: boolean;
  user: CurrentUser | null;
  login: (email: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
};

const AuthContext = createContext<AuthContextValue | undefined>(undefined);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [isAuthenticated, setAuthenticated] = useState(Boolean(getAccessToken()));
  const [user, setUser] = useState<CurrentUser | null>(null);

  useEffect(() => {
    if (!isAuthenticated || user) return;
    let alive = true;
    api
      .me()
      .then((currentUser) => {
        if (alive) setUser(currentUser as CurrentUser);
      })
      .catch(() => {
        clearTokens();
        if (alive) setAuthenticated(false);
      });
    return () => {
      alive = false;
    };
  }, [isAuthenticated, user]);

  const value = useMemo<AuthContextValue>(
    () => ({
      isAuthenticated,
      user,
      login: async (email: string, password: string) => {
        const tokens = await api.login(email, password);
        setTokens(tokens);
        setAuthenticated(true);
        try {
          const currentUser = await api.me();
          setUser(currentUser as CurrentUser);
        } catch {
          setUser(null);
        }
      },
      logout: async () => {
        const refreshToken = getRefreshToken();
        if (refreshToken) {
          try {
            await api.logout(refreshToken);
          } catch {
            // Local logout must still complete if token revoke request fails.
          }
        }
        clearTokens();
        setAuthenticated(false);
        setUser(null);
      }
    }),
    [isAuthenticated, user]
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error("useAuth must be used inside AuthProvider");
  }
  return context;
}
