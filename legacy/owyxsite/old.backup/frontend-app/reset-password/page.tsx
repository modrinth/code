"use client";

import { useState, useEffect, Suspense } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import Link from "next/link";
import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";

type MessageType = "success" | "error" | null;

function ResetPasswordForm() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const token = searchParams.get("token");

  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [loading, setLoading] = useState(false);
  const [tokenValid, setTokenValid] = useState<boolean | null>(null);
  const [showPassword, setShowPassword] = useState(false);
  const [showConfirmPassword, setShowConfirmPassword] = useState(false);
  const [message, setMessage] = useState<{ text: string; type: MessageType }>({ text: "", type: null });

  function showMessage(text: string, type: MessageType) {
    setMessage({ text, type });
    if (type === "error") setTimeout(() => setMessage({ text: "", type: null }), 5000);
  }

  // Проверка токена при загрузке
  useEffect(() => {
    if (!token) {
      setTokenValid(false);
      return;
    }
    async function verifyToken() {
      try {
        const res = await fetch(`/api/auth/verify-reset-token?token=${token}`);
        setTokenValid(res.ok);
        if (!res.ok) {
          const data = await res.json();
          showMessage(data.error ?? "Недействительный или истёкший токен", "error");
        }
      } catch {
        setTokenValid(false);
        showMessage("Ошибка проверки токена", "error");
      }
    }
    verifyToken();
  }, [token]);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!token) return;

    if (password !== confirmPassword) {
      showMessage("Пароли не совпадают", "error");
      return;
    }
    if (password.length < 8) {
      showMessage("Пароль должен содержать минимум 8 символов", "error");
      return;
    }

    setLoading(true);
    try {
      const res = await fetch("/api/auth/reset-password", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ token, password }),
      });
      const result = await res.json();
      if (res.ok) {
        showMessage("Пароль успешно изменён! Перенаправление...", "success");
        setTimeout(() => router.replace("/login"), 2000);
      } else {
        showMessage(result.error ?? "Ошибка смены пароля", "error");
      }
    } catch {
      showMessage("Ошибка соединения с сервером", "error");
    } finally {
      setLoading(false);
    }
  }

  // Проверка силы пароля
  function getPasswordStrength(pwd: string): { score: number; label: string; color: string } {
    if (pwd.length === 0) return { score: 0, label: "", color: "" };
    let score = 0;
    if (pwd.length >= 8) score++;
    if (pwd.length >= 12) score++;
    if (/[A-Z]/.test(pwd)) score++;
    if (/[0-9]/.test(pwd)) score++;
    if (/[^A-Za-z0-9]/.test(pwd)) score++;
    if (score <= 1) return { score, label: "Слабый", color: "bg-red-500" };
    if (score <= 3) return { score, label: "Средний", color: "bg-yellow-500" };
    return { score, label: "Сильный", color: "bg-green-500" };
  }

  const strength = getPasswordStrength(password);

  return (
    <div className="w-full max-w-md">
      <div className="glass-effect rounded-2xl p-8 border border-[#FFAA00]/30 gold-glow">
        {/* Иконка */}
        <div className="text-center mb-8">
          <div className="animate-float inline-block">
            <i className="fas fa-lock-open text-6xl text-[#FFAA00] mb-4 block" />
          </div>
          <h1 className="text-3xl font-bold text-shadow">
            <span className="text-[#FFAA00]">Новый</span>{" "}
            <span className="text-white">пароль</span>
          </h1>
          <p className="text-gray-300 mt-2">Введите новый пароль для вашего аккаунта</p>
        </div>

        {/* Сообщение */}
        {message.type && (
          <div
            className={`mb-6 p-4 rounded-xl border text-center ${
              message.type === "success"
                ? "bg-green-900/50 text-green-300 border-green-500/50"
                : "bg-red-900/50 text-red-300 border-red-500/50"
            }`}
          >
            <i
              className={`fas ${
                message.type === "success" ? "fa-check-circle" : "fa-exclamation-circle"
              } mr-2`}
            />
            {message.text}
          </div>
        )}

        {/* Невалидный токен */}
        {tokenValid === false ? (
          <div className="text-center space-y-4">
            <i className="fas fa-times-circle text-5xl text-red-400 block mb-4" />
            <p className="text-gray-400">Ссылка для сброса пароля недействительна или истекла.</p>
            <Link
              href="/forgot-password"
              className="btn-minecraft inline-block px-6 py-3 rounded-xl font-bold hover:scale-105 transition-all duration-300"
            >
              <i className="fas fa-redo mr-2" />
              Запросить новую ссылку
            </Link>
          </div>
        ) : tokenValid === null ? (
          <div className="text-center py-8">
            <i className="fas fa-spinner fa-spin text-[#FFAA00] text-3xl mb-4 block" />
            <p className="text-gray-300">Проверяем токен...</p>
          </div>
        ) : (
          <form onSubmit={handleSubmit} className="space-y-5">
            {/* Новый пароль */}
            <div>
              <label className="block text-[#FFFF55] font-bold text-sm mb-2">
                <i className="fas fa-key mr-2" />
                Новый пароль
              </label>
              <div className="relative">
                <input
                  type={showPassword ? "text" : "password"}
                  id="new-password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  required
                  minLength={8}
                  disabled={loading}
                  placeholder="Введите новый пароль"
                  className="w-full px-4 py-3 pr-12 rounded-xl text-white placeholder-gray-400
                             bg-[#2a2a2a]/50 border-2 border-[#FFAA00]/30
                             focus:border-[#FFAA00] focus:shadow-[0_0_20px_rgba(255,170,0,0.2)]
                             focus:outline-none transition-all duration-300
                             disabled:opacity-50"
                />
                <button
                  type="button"
                  onClick={() => setShowPassword(!showPassword)}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-[#FFAA00] transition-colors"
                >
                  <i className={`fas ${showPassword ? "fa-eye-slash" : "fa-eye"}`} />
                </button>
              </div>
              {/* Индикатор силы */}
              {password.length > 0 && (
                <div className="mt-2">
                  <div className="flex gap-1 mb-1">
                    {[1, 2, 3, 4, 5].map((i) => (
                      <div
                        key={i}
                        className={`h-1 flex-1 rounded-full transition-colors ${
                          i <= strength.score ? strength.color : "bg-gray-700"
                        }`}
                      />
                    ))}
                  </div>
                  <p className="text-xs text-gray-400">
                    Надёжность: <span className="font-medium">{strength.label}</span>
                  </p>
                </div>
              )}
            </div>

            {/* Подтверждение пароля */}
            <div>
              <label className="block text-[#FFFF55] font-bold text-sm mb-2">
                <i className="fas fa-key mr-2" />
                Повторите пароль
              </label>
              <div className="relative">
                <input
                  type={showConfirmPassword ? "text" : "password"}
                  id="confirm-password"
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  required
                  disabled={loading}
                  placeholder="Повторите новый пароль"
                  className={`w-full px-4 py-3 pr-12 rounded-xl text-white placeholder-gray-400
                             bg-[#2a2a2a]/50 border-2 transition-all duration-300 focus:outline-none
                             disabled:opacity-50
                             ${
                               confirmPassword && password !== confirmPassword
                                 ? "border-red-500 focus:border-red-500"
                                 : confirmPassword && password === confirmPassword
                                 ? "border-green-500 focus:border-green-500"
                                 : "border-[#FFAA00]/30 focus:border-[#FFAA00] focus:shadow-[0_0_20px_rgba(255,170,0,0.2)]"
                             }`}
                />
                <button
                  type="button"
                  onClick={() => setShowConfirmPassword(!showConfirmPassword)}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-[#FFAA00] transition-colors"
                >
                  <i className={`fas ${showConfirmPassword ? "fa-eye-slash" : "fa-eye"}`} />
                </button>
              </div>
              {confirmPassword && password !== confirmPassword && (
                <p className="text-red-400 text-xs mt-1">
                  <i className="fas fa-times mr-1" />
                  Пароли не совпадают
                </p>
              )}
            </div>

            <button
              type="submit"
              id="reset-submit"
              disabled={loading || !password || !confirmPassword || password !== confirmPassword}
              className="btn-minecraft w-full py-3 rounded-xl text-lg font-bold
                         hover:scale-105 transition-all duration-300
                         disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none"
            >
              {loading ? (
                <>
                  <i className="fas fa-spinner fa-spin mr-2" />
                  Сохранение...
                </>
              ) : (
                <>
                  <i className="fas fa-save mr-2" />
                  Сохранить пароль
                </>
              )}
            </button>
          </form>
        )}

        <div className="mt-6 text-center">
          <Link
            href="/login"
            className="text-[#FFAA00] hover:text-[#FFFF55] font-bold transition-colors text-sm"
          >
            <i className="fas fa-arrow-left mr-2" />
            Вернуться к входу
          </Link>
        </div>
      </div>
    </div>
  );
}

export default function ResetPasswordPage() {
  return (
    <>
      <Header />
      <main className="relative z-10 flex-1 min-h-screen flex items-center justify-center py-24 px-4">
        <Suspense
          fallback={
            <div className="text-center">
              <i className="fas fa-spinner fa-spin text-[#FFAA00] text-3xl" />
            </div>
          }
        >
          <ResetPasswordForm />
        </Suspense>
      </main>
      <Footer />
    </>
  );
}
