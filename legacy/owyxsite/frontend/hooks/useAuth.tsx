"use client";

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { useRouter } from "next/navigation";

export interface AuthUser {
  id?: number;
  email?: string;
  nickname?: string;
  first_name?: string;
  role?: string;
  avatar_url?: string;
  trust_level?: number;
  age?: number;
  discord?: string;
  bio?: string;
  created_at?: string;
  status?: string;
  skin_url?: string | null;
  skin_model?: string;
  is_email_verified?: boolean;
}

interface UseAuthOptions {
  /** Redirect to /login when there is no session. */
  requireAuth?: boolean;
  /** Redirect to /profile when a session already exists. */
  redirectIfAuth?: boolean;
}

export interface UseAuthReturn {
  user: AuthUser | null;
  loading: boolean;
  isAuth: boolean;
  logout: () => void;
  refreshAuth: () => Promise<void>;
}

const AuthContext = createContext<UseAuthReturn | null>(null);

const listeners = new Set<() => void>();

export function notifyAuthChanged() {
  listeners.forEach((listener) => listener());
}

function clearSession() {
  localStorage.removeItem("auth_token");
  localStorage.removeItem("remember_me");
  localStorage.removeItem("token_expires");
}

export function AuthProvider({ children }: { children: ReactNode }) {
  const router = useRouter();
  const [user, setUser] = useState<AuthUser | null>(null);
  const [loading, setLoading] = useState(true);
  const inFlight = useRef<Promise<void> | null>(null);

  const checkAuth = useCallback(async () => {
    if (inFlight.current) return inFlight.current;
    const run = (async () => {
      // Yield so the initial effect never setStates synchronously.
      await Promise.resolve();
      try {
        const token = localStorage.getItem("auth_token");
        const tokenExpires = localStorage.getItem("token_expires");

        if (token && tokenExpires && new Date() > new Date(tokenExpires)) {
          clearSession();
          setUser(null);
          setLoading(false);
          return;
        }
        if (!token) {
          setUser(null);
          setLoading(false);
          return;
        }

        const res = await fetch("/api/auth/verify", {
          headers: { Authorization: `Bearer ${token}` },
        });
        if (res.ok) {
          const data = await res.json();
          setUser(data.user ?? null);
          setLoading(false);
          return;
        }
        if (res.status === 401 || res.status === 403) {
          clearSession();
          setUser(null);
          setLoading(false);
          return;
        }
        // Transient 5xx: keep an existing session, don't invent one.
        setLoading(false);
      } catch {
        setLoading(false);
      } finally {
        inFlight.current = null;
      }
    })();
    inFlight.current = run;
    return run;
  }, []);

  useEffect(() => {
    void checkAuth();
    const onAuthChanged = () => {
      void checkAuth();
    };
    const onPageShow = (event: PageTransitionEvent) => {
      if (event.persisted) void checkAuth();
    };
    listeners.add(onAuthChanged);
    window.addEventListener("pageshow", onPageShow);
    return () => {
      listeners.delete(onAuthChanged);
      window.removeEventListener("pageshow", onPageShow);
    };
  }, [checkAuth]);

  const logout = useCallback(() => {
    const token = localStorage.getItem("auth_token");
    if (token) {
      fetch("/api/auth/logout", {
        method: "POST",
        headers: { Authorization: `Bearer ${token}` },
      }).catch(() => {});
    }
    clearSession();
    setUser(null);
    setLoading(false);
    router.replace("/login");
  }, [router]);

  const value = useMemo<UseAuthReturn>(
    () => ({
      user,
      loading,
      isAuth: Boolean(user),
      logout,
      refreshAuth: checkAuth,
    }),
    [user, loading, logout, checkAuth]
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth(options: UseAuthOptions = {}): UseAuthReturn {
  const ctx = useContext(AuthContext);
  if (!ctx) {
    throw new Error("useAuth must be used inside AuthProvider");
  }

  const { requireAuth = false, redirectIfAuth = false } = options;
  const router = useRouter();
  const { user, loading } = ctx;

  useEffect(() => {
    if (loading) return;
    if (requireAuth && !user) {
      router.replace("/login");
    } else if (redirectIfAuth && user) {
      router.replace("/profile");
    }
  }, [user, loading, requireAuth, redirectIfAuth, router]);

  return ctx;
}
