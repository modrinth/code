"use client";

import Script from "next/script";
import Link from "next/link";
import { useState, useEffect, useCallback } from "react";
import { useRouter } from "next/navigation";
import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";
import ParticlesBackground from "@/components/effects/ParticlesBackground";

function getPasswordStrength(password: string) {
  if (!password) return { level: 0, label: "", color: "" };
  let score = 0;
  const hints: string[] = [];
  if (password.length >= 8) score++; else hints.push("мин. 8 символов");
  if (/[a-z]/.test(password)) score++; else hints.push("строчные буквы");
  if (/[A-Z]/.test(password)) score++; else hints.push("заглавные буквы");
  if (/[0-9]/.test(password)) score++; else hints.push("цифры");
  if (/[^A-Za-z0-9]/.test(password)) score++; else hints.push("спецсимволы");

  if (score <= 1) return { level: 20, label: `Слабый: нужны ${hints.slice(0, 2).join(", ")}`, color: "bg-red-500" };
  if (score === 2) return { level: 40, label: `Средний: нужны ${hints.join(", ")}`, color: "bg-orange-400" };
  if (score === 3) return { level: 60, label: "Хороший", color: "bg-yellow-400" };
  if (score === 4) return { level: 80, label: "Сильный", color: "bg-green-400" };
  return { level: 100, label: "Отличный пароль!", color: "bg-green-500" };
}

export default function RegisterPage() {
  const router = useRouter();

  const [form, setForm] = useState({
    first_name: "",
    minecraft_nick: "",
    email: "",
    password: "",
    confirm_password: "",
    terms: false,
  });
  const [loading, setLoading] = useState(false);
  const [showPassword, setShowPassword] = useState(false);
  const [message, setMessage] = useState<{ text: string; type: "success" | "error" } | null>(null);
  const [turnstileToken, setTurnstileToken] = useState<string | null>(null);

  // Redirect if already authed
  useEffect(() => {
    const token = localStorage.getItem("auth_token");
    const expires = localStorage.getItem("token_expires");
    if (token && expires && new Date() < new Date(expires)) {
      router.replace("/profile");
    }
  }, [router]);

  useEffect(() => {
    (window as unknown as Record<string, unknown>)["onTurnstileSuccess"] = (token: string) => {
      setTurnstileToken(token);
    };
    return () => { delete (window as unknown as Record<string, unknown>)["onTurnstileSuccess"]; };
  }, []);

  const showMsg = useCallback((text: string, type: "success" | "error") => {
    setMessage({ text, type });
    if (type === "error") setTimeout(() => setMessage(null), 6000);
  }, []);

  const strength = getPasswordStrength(form.password);
  const passwordsMatch = form.confirm_password
    ? form.password === form.confirm_password
    : null;

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();

    if (!turnstileToken) {
      showMsg("Пожалуйста, пройдите проверку капчи", "error");
      return;
    }
    if (form.password !== form.confirm_password) {
      showMsg("Пароли не совпадают", "error");
      return;
    }

    setLoading(true);
    try {
      const { confirm_password, ...payload } = form;
      void confirm_password;
      const res = await fetch("/api/auth/register", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ ...payload, turnstileToken }),
      });

      const result = await res.json();

      if (res.ok) {
        showMsg("Регистрация успешна! Перенаправляем на вход...", "success");
        setTimeout(() => router.replace("/login"), 2000);
      } else {
        if (result.details?.length) {
          showMsg(`Ошибка: ${result.details.map((d: { msg: string }) => d.msg).join(", ")}`, "error");
        } else {
          showMsg(result.error || "Ошибка регистрации", "error");
        }
      }
    } catch {
      showMsg("Ошибка соединения с сервером", "error");
    } finally {
      setLoading(false);
    }
  }

  const inputClass =
    "w-full px-4 py-3 rounded-xl bg-[#2a2a2a]/80 border-2 border-[#FFAA00]/30 text-white placeholder-gray-400 focus:border-[#FFAA00] focus:outline-none focus:shadow-[0_0_20px_rgba(255,170,0,0.2)] transition-all";

  return (
    <>
      <Script src="https://challenges.cloudflare.com/turnstile/v0/api.js" strategy="lazyOnload" />
      <ParticlesBackground />
      <Header />

      <main className="relative z-10 min-h-[calc(100vh-64px)] flex items-center justify-center py-12 px-4">
        <div className="w-full max-w-lg">
          <div className="glass-effect rounded-2xl p-8 border border-[#FFAA00]/30 gold-glow">
            {/* Заголовок */}
            <div className="text-center mb-8">
              <div className="animate-float inline-block">
                <svg className="w-16 h-16 text-[#FFAA00] mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M18 9v3m0 0v3m0-3h3m-3 0h-3m-2-5a4 4 0 11-8 0 4 4 0 018 0zM3 20a6 6 0 0112 0v1H3v-1z" />
                </svg>
              </div>
              <h1 className="text-4xl font-bold text-shadow mb-2">
                <span className="text-[#FFAA00]">Регистрация</span>
              </h1>
              <p className="text-gray-300">Создайте аккаунт для игры на сервере</p>
              <p className="text-gray-500 text-sm mt-1">Дополнительные настройки доступны в профиле</p>
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

            <form onSubmit={handleSubmit} className="space-y-5">
              {/* Имя (необязательно) */}
              <div>
                <label className="block text-[#FFFF55] font-semibold mb-2 text-sm">
                  Имя <span className="text-gray-400 font-normal">(необязательно)</span>
                </label>
                <input
                  type="text"
                  value={form.first_name}
                  onChange={e => setForm(f => ({ ...f, first_name: e.target.value }))}
                  className={inputClass}
                  placeholder="Ваше имя"
                  id="reg-first-name"
                />
              </div>

              {/* Minecraft ник */}
              <div>
                <label className="block text-[#FFFF55] font-semibold mb-2 text-sm">
                  Minecraft ник <span className="text-red-400">*</span>
                </label>
                <input
                  type="text"
                  required
                  value={form.minecraft_nick}
                  onChange={e => setForm(f => ({ ...f, minecraft_nick: e.target.value }))}
                  className={inputClass}
                  placeholder="YourNickname"
                  id="reg-minecraft-nick"
                />
                <p className="text-xs text-gray-500 mt-1">Этот ник будет использоваться на сервере</p>
              </div>

              {/* Email */}
              <div>
                <label className="block text-[#FFFF55] font-semibold mb-2 text-sm">
                  Email <span className="text-red-400">*</span>
                </label>
                <input
                  type="email"
                  required
                  value={form.email}
                  onChange={e => setForm(f => ({ ...f, email: e.target.value }))}
                  className={inputClass}
                  placeholder="your@email.com"
                  id="reg-email"
                />
              </div>

              {/* Пароль */}
              <div>
                <label className="block text-[#FFFF55] font-semibold mb-2 text-sm">
                  Пароль <span className="text-red-400">*</span>
                </label>
                <div className="relative">
                  <input
                    type={showPassword ? "text" : "password"}
                    required
                    minLength={8}
                    value={form.password}
                    onChange={e => setForm(f => ({ ...f, password: e.target.value }))}
                    className={`${inputClass} pr-12`}
                    placeholder="••••••••"
                    id="reg-password"
                  />
                  <button
                    type="button"
                    onClick={() => setShowPassword(v => !v)}
                    className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-[#FFFF55] transition-colors"
                    id="reg-toggle-password"
                  >
                    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2}
                        d={showPassword
                          ? "M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21"
                          : "M15 12a3 3 0 11-6 0 3 3 0 016 0zM2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
                        }
                      />
                    </svg>
                  </button>
                </div>
                {/* Индикатор силы */}
                {form.password && (
                  <>
                    <div className="h-1 bg-[#333] rounded-full mt-2 overflow-hidden">
                      <div
                        className={`h-full rounded-full transition-all duration-300 ${strength.color}`}
                        style={{ width: `${strength.level}%` }}
                      />
                    </div>
                    <p className={`text-xs mt-1 ${
                      strength.level >= 80 ? "text-green-400"
                      : strength.level >= 60 ? "text-yellow-400"
                      : "text-red-400"
                    }`}>
                      {strength.label}
                    </p>
                  </>
                )}
              </div>

              {/* Подтверждение пароля */}
              <div>
                <label className="block text-[#FFFF55] font-semibold mb-2 text-sm">
                  Подтверждение пароля <span className="text-red-400">*</span>
                </label>
                <input
                  type="password"
                  required
                  value={form.confirm_password}
                  onChange={e => setForm(f => ({ ...f, confirm_password: e.target.value }))}
                  className={inputClass}
                  placeholder="••••••••"
                  id="reg-confirm-password"
                />
                {passwordsMatch === false && (
                  <p className="text-xs text-red-400 mt-1">Пароли не совпадают</p>
                )}
                {passwordsMatch === true && (
                  <p className="text-xs text-green-400 mt-1">Пароли совпадают ✓</p>
                )}
              </div>

              {/* Согласие */}
              <div className="flex items-start gap-3 p-4 glass-effect rounded-xl border border-[#FFAA00]/20">
                <input
                  type="checkbox"
                  required
                  id="terms-checkbox"
                  checked={form.terms}
                  onChange={e => setForm(f => ({ ...f, terms: e.target.checked }))}
                  className="mt-1 w-4 h-4 rounded border-[#FFAA00]/30 text-[#FFAA00] bg-[#2a2a2a] cursor-pointer"
                />
                <label htmlFor="terms-checkbox" className="text-gray-300 text-sm leading-relaxed cursor-pointer">
                  Я согласен с{" "}
                  <a href="https://discord.gg/owyx" target="_blank" rel="noopener noreferrer"
                    className="text-[#FFAA00] hover:text-[#FFFF55] transition-colors font-medium">
                    правилами сервера (Discord)
                  </a>
                </label>
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
                className="btn-minecraft w-full py-4 rounded-xl text-lg font-bold transition-all duration-300 disabled:opacity-60 disabled:cursor-not-allowed"
                id="reg-submit-btn"
              >
                {loading ? (
                  <span className="flex items-center justify-center gap-2">
                    <svg className="animate-spin w-5 h-5" fill="none" viewBox="0 0 24 24">
                      <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                      <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                    </svg>
                    Регистрация...
                  </span>
                ) : "Зарегистрироваться"}
              </button>
            </form>

            <div className="mt-8 text-center glass-effect rounded-xl p-4 border border-[#FFAA00]/20">
              <p className="text-gray-300">
                Уже есть аккаунт?{" "}
                <Link href="/login" className="text-[#FFAA00] hover:text-[#FFFF55] font-bold transition-colors">
                  Войти
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
