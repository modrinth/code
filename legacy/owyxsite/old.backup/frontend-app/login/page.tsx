"use client";

import Script from "next/script";
import Link from "next/link";
import { useState, useEffect, useCallback } from "react";
import { useRouter } from "next/navigation";
import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";
import ParticlesBackground from "@/components/effects/ParticlesBackground";

export default function LoginPage() {
  const router = useRouter();

  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [remember, setRemember] = useState(false);
  const [showPassword, setShowPassword] = useState(false);
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState<{ text: string; type: "success" | "error" } | null>(null);
  const [turnstileToken, setTurnstileToken] = useState<string | null>(null);

  // Если уже залогинен — редирект
  useEffect(() => {
    const token = localStorage.getItem("auth_token");
    const expires = localStorage.getItem("token_expires");
    if (token && expires && new Date() < new Date(expires)) {
      router.replace("/profile");
    }
  }, [router]);

  // Глобальный callback для Turnstile
  useEffect(() => {
    (window as unknown as Record<string, unknown>)["onTurnstileSuccess"] = (token: string) => {
      setTurnstileToken(token);
    };
    return () => {
      delete (window as unknown as Record<string, unknown>)["onTurnstileSuccess"];
    };
  }, []);

  const showMsg = useCallback((text: string, type: "success" | "error") => {
    setMessage({ text, type });
    if (type === "error") {
      setTimeout(() => setMessage(null), 5000);
    }
  }, []);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();

    if (!turnstileToken) {
      showMsg("Пожалуйста, пройдите проверку капчи", "error");
      return;
    }

    setLoading(true);
    try {
      const res = await fetch("/api/auth/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email, password, remember, turnstileToken }),
      });

      const result = await res.json();

      if (res.ok) {
        showMsg("Успешный вход! Перенаправляем...", "success");
        localStorage.setItem("auth_token", result.token);
        localStorage.setItem("remember_me", String(remember));
        const ttl = remember ? 7 * 24 * 60 * 60 * 1000 : 24 * 60 * 60 * 1000;
        localStorage.setItem("token_expires", new Date(Date.now() + ttl).toISOString());
        setTimeout(() => router.replace("/profile"), 1000);
      } else {
        showMsg(result.error || "Ошибка входа", "error");
      }
    } catch {
      showMsg("Ошибка соединения с сервером", "error");
    } finally {
      setLoading(false);
    }
  }

  return (
    <>
      <Script
        src="https://challenges.cloudflare.com/turnstile/v0/api.js"
        strategy="lazyOnload"
      />
      <ParticlesBackground />
      <Header />

      <main className="relative z-10 min-h-[calc(100vh-64px)] flex items-center justify-center py-20 px-4">
        <div className="w-full max-w-md">
          <div className="glass-effect rounded-2xl p-8 border border-[#FFAA00]/30 gold-glow">
            {/* Заголовок */}
            <div className="text-center mb-8">
              <div className="animate-float inline-block">
                <svg className="w-16 h-16 text-[#FFAA00] mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M11 16l-4-4m0 0l4-4m-4 4h14m-5 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h7a3 3 0 013 3v1" />
                </svg>
              </div>
              <h1 className="text-4xl font-bold text-shadow mb-2">
                <span className="text-[#FFAA00]">Вход</span>
              </h1>
              <p className="text-gray-300">Войдите в свой аккаунт</p>
            </div>

            {/* Сообщение */}
            {message && (
              <div className={`mb-6 p-4 rounded-xl border text-sm ${
                message.type === "success"
                  ? "bg-green-900/50 text-green-300 border-green-500/50"
                  : "bg-red-900/50 text-red-300 border-red-500/50"
              }`}>
                {message.text}
              </div>
            )}

            <form onSubmit={handleSubmit} className="space-y-6">
              {/* Email */}
              <div>
                <label className="block text-[#FFFF55] font-bold text-base mb-2">
                  Email
                </label>
                <input
                  type="email"
                  required
                  value={email}
                  onChange={e => setEmail(e.target.value)}
                  className="w-full px-4 py-3 rounded-xl bg-[#2a2a2a]/50 border-2 border-[#FFAA00]/30 text-white placeholder-gray-400 text-base focus:border-[#FFAA00] focus:outline-none focus:shadow-[0_0_20px_rgba(255,170,0,0.2)] transition-all"
                  placeholder="your@email.com"
                  id="login-email"
                />
              </div>

              {/* Пароль */}
              <div>
                <label className="block text-[#FFFF55] font-bold text-base mb-2">
                  Пароль
                </label>
                <div className="relative">
                  <input
                    type={showPassword ? "text" : "password"}
                    required
                    value={password}
                    onChange={e => setPassword(e.target.value)}
                    className="w-full px-4 py-3 pr-12 rounded-xl bg-[#2a2a2a]/50 border-2 border-[#FFAA00]/30 text-white placeholder-gray-400 text-base focus:border-[#FFAA00] focus:outline-none focus:shadow-[0_0_20px_rgba(255,170,0,0.2)] transition-all"
                    placeholder="••••••••"
                    id="login-password"
                  />
                  <button
                    type="button"
                    onClick={() => setShowPassword(!showPassword)}
                    className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-[#FFFF55] transition-colors"
                    id="toggle-password-btn"
                  >
                    {showPassword ? (
                      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
                      </svg>
                    ) : (
                      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                      </svg>
                    )}
                  </button>
                </div>
              </div>

              {/* Запомнить + Забыл пароль */}
              <div className="flex items-center justify-between">
                <label className="flex items-center gap-2 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={remember}
                    onChange={e => setRemember(e.target.checked)}
                    className="rounded border-[#FFAA00]/30 text-[#FFAA00]"
                    id="remember-checkbox"
                  />
                  <span className="text-gray-300 text-sm">Запомнить меня</span>
                </label>
                <Link href="/forgot-password" className="text-[#FFAA00] hover:text-[#FFFF55] transition-colors text-sm">
                  Забыли пароль?
                </Link>
              </div>

              {/* Turnstile */}
              <div className="flex justify-center py-2">
                <div
                  className="cf-turnstile"
                  data-sitekey="0x4AAAAAAB9h5o9JoTW0XgIv"
                  data-theme="dark"
                  data-callback="onTurnstileSuccess"
                />
              </div>

              {/* Кнопка */}
              <button
                type="submit"
                disabled={loading}
                className="btn-minecraft w-full py-3 rounded-xl text-lg font-bold hover:scale-105 transition-all duration-300 animate-glow disabled:opacity-60 disabled:cursor-not-allowed disabled:transform-none"
                id="login-submit-btn"
              >
                {loading ? (
                  <span className="flex items-center justify-center gap-2">
                    <svg className="animate-spin w-5 h-5" fill="none" viewBox="0 0 24 24">
                      <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                      <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                    </svg>
                    Вход...
                  </span>
                ) : "Войти"}
              </button>
            </form>

            <div className="mt-8 text-center">
              <p className="text-gray-300">
                Нет аккаунта?{" "}
                <Link href="/register" className="text-[#FFAA00] hover:text-[#FFFF55] font-bold transition-colors">
                  Зарегистрироваться
                </Link>
              </p>
            </div>
          </div>
        </div>
      </main>

      <Footer />
    </>
  );
}
