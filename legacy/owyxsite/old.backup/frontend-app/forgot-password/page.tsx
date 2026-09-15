"use client";

import { useState } from "react";
import Link from "next/link";
import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";

type MessageType = "success" | "error" | null;

export default function ForgotPasswordPage() {
  const [email, setEmail] = useState("");
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState<{ text: string; type: MessageType }>({ text: "", type: null });
  const [sent, setSent] = useState(false);

  function showMessage(text: string, type: MessageType) {
    setMessage({ text, type });
    if (type === "error") {
      setTimeout(() => setMessage({ text: "", type: null }), 5000);
    }
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!email.trim()) return;

    setLoading(true);
    try {
      const res = await fetch("/api/auth/forgot-password", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email }),
      });
      const result = await res.json();
      if (res.ok) {
        setSent(true);
        showMessage("Ссылка для восстановления отправлена на email!", "success");
      } else {
        showMessage(result.error || "Ошибка отправки письма", "error");
      }
    } catch {
      showMessage("Ошибка соединения с сервером", "error");
    } finally {
      setLoading(false);
    }
  }

  return (
    <>
      <Header />
      <main className="relative z-10 flex-1 min-h-screen flex items-center justify-center py-24 px-4">
        <div className="w-full max-w-md">
          <div className="glass-effect rounded-2xl p-8 border border-[#FFAA00]/30 gold-glow">
            {/* Иконка */}
            <div className="text-center mb-8">
              <div className="animate-float inline-block">
                <i className="fas fa-key text-6xl text-[#FFAA00] mb-4 block" />
              </div>
              <h1 className="text-3xl font-bold text-shadow">
                <span className="text-[#FFAA00]">Восстановление</span>{" "}
                <span className="text-white">пароля</span>
              </h1>
              <p className="text-gray-300 mt-2">
                Введите email для получения ссылки восстановления
              </p>
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

            {sent ? (
              /* Состояние после отправки */
              <div className="text-center space-y-6">
                <div className="bg-green-900/30 border border-green-500/30 rounded-xl p-6">
                  <i className="fas fa-envelope-circle-check text-4xl text-green-400 mb-3 block" />
                  <p className="text-white font-semibold mb-1">Письмо отправлено!</p>
                  <p className="text-gray-400 text-sm">
                    Проверьте почту <strong className="text-[#FFAA00]">{email}</strong> и
                    перейдите по ссылке из письма
                  </p>
                </div>
                <p className="text-gray-500 text-sm">
                  Не получили письмо?{" "}
                  <button
                    onClick={() => setSent(false)}
                    className="text-[#FFAA00] hover:text-[#FFFF55] transition-colors"
                  >
                    Отправить снова
                  </button>
                </p>
              </div>
            ) : (
              /* Форма */
              <form onSubmit={handleSubmit} className="space-y-6">
                <div>
                  <label className="block text-[#FFFF55] font-bold text-sm mb-2">
                    <i className="fas fa-envelope mr-2" />
                    Email адрес
                  </label>
                  <input
                    type="email"
                    id="forgot-email"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    required
                    disabled={loading}
                    placeholder="your@email.com"
                    className="w-full px-4 py-3 rounded-xl text-white placeholder-gray-400
                               bg-[#2a2a2a]/50 border-2 border-[#FFAA00]/30
                               focus:border-[#FFAA00] focus:shadow-[0_0_20px_rgba(255,170,0,0.2)]
                               focus:outline-none transition-all duration-300
                               disabled:opacity-50 disabled:cursor-not-allowed"
                  />
                </div>

                <button
                  type="submit"
                  id="forgot-submit"
                  disabled={loading || !email.trim()}
                  className="btn-minecraft w-full py-3 rounded-xl text-lg font-bold
                             hover:scale-105 transition-all duration-300
                             disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none"
                >
                  {loading ? (
                    <>
                      <i className="fas fa-spinner fa-spin mr-2" />
                      Отправка...
                    </>
                  ) : (
                    <>
                      <i className="fas fa-paper-plane mr-2" />
                      Отправить ссылку
                    </>
                  )}
                </button>
              </form>
            )}

            <div className="mt-8 text-center">
              <p className="text-gray-300 text-sm">
                Вспомнили пароль?{" "}
                <Link
                  href="/login"
                  className="text-[#FFAA00] hover:text-[#FFFF55] font-bold transition-colors"
                >
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
