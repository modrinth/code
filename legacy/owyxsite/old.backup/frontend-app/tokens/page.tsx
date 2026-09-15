"use client";

import { useState, useEffect, useCallback, useRef } from "react";
import { useRouter } from "next/navigation";
import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";
import { useAuth } from "@/hooks/useAuth";

interface TokenData {
  token: string;
  expiresAt: string;
}

export default function GameTokenPage() {
  const { loading: authLoading, isAuth } = useAuth({ requireAuth: true });
  const router = useRouter();

  const [tokenData, setTokenData] = useState<TokenData | null>(null);
  const [generating, setGenerating] = useState(false);
  const [copied, setCopied] = useState(false);
  const [countdown, setCountdown] = useState<string>("");
  const [error, setError] = useState<string | null>(null);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // Redirect if not auth (backup)
  useEffect(() => {
    if (!authLoading && !isAuth) {
      router.replace("/login");
    }
  }, [authLoading, isAuth, router]);

  function formatTimeRemaining(expiresAt: string): string {
    const now = new Date();
    const expires = new Date(expiresAt);
    const diff = expires.getTime() - now.getTime();
    if (diff <= 0) return "Истёк";
    const minutes = Math.floor(diff / 60000);
    const seconds = Math.floor((diff % 60000) / 1000);
    return `${minutes}м ${seconds}с`;
  }

  const startCountdown = useCallback((expiresAt: string) => {
    if (intervalRef.current) clearInterval(intervalRef.current);
    setCountdown(formatTimeRemaining(expiresAt));
    intervalRef.current = setInterval(() => {
      const remaining = formatTimeRemaining(expiresAt);
      setCountdown(remaining);
      if (remaining === "Истёк") {
        clearInterval(intervalRef.current!);
        intervalRef.current = null;
        setTokenData(null);
        setCountdown("");
      }
    }, 1000);
  }, []);

  useEffect(() => {
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, []);

  async function generateToken() {
    setGenerating(true);
    setError(null);
    const authToken = localStorage.getItem("auth_token");

    try {
      const res = await fetch("/api/auth/generate-game-token", {
        method: "POST",
        headers: {
          Authorization: `Bearer ${authToken}`,
          "Content-Type": "application/json",
        },
      });
      const data = await res.json();

      if (res.ok && data.success) {
        const td: TokenData = { token: data.token, expiresAt: data.expiresAt };
        setTokenData(td);
        startCountdown(td.expiresAt);
      } else {
        setError(data.error ?? "Ошибка генерации токена");
      }
    } catch {
      setError("Ошибка соединения с сервером");
    } finally {
      setGenerating(false);
    }
  }

  async function copyToken() {
    if (!tokenData) return;
    try {
      await navigator.clipboard.writeText(tokenData.token);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      setError("Не удалось скопировать токен");
    }
  }

  const isExpired = countdown === "Истёк";

  if (authLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <i className="fas fa-spinner fa-spin text-[#FFAA00] text-4xl" />
      </div>
    );
  }

  return (
    <>
      <Header />
      <main className="relative z-10 flex-1 pt-20 pb-12">
        <div className="max-w-4xl mx-auto px-4 py-8">
          {/* Заголовок */}
          <div className="text-center mb-8">
            <h1 className="text-4xl font-bold text-[#FFAA00] mb-3 text-shadow">
              <i className="fas fa-key mr-3" />
              Игровой токен
            </h1>
            <p className="text-gray-300 text-lg">
              Токен для авторизации на игровом сервере
            </p>
          </div>

          {/* Карточка токена */}
          <div className="glass-effect rounded-2xl p-8 mb-8 text-center">
            {/* Ошибка */}
            {error && (
              <div className="mb-6 p-4 rounded-xl bg-red-900/50 text-red-300 border border-red-500/50">
                <i className="fas fa-exclamation-circle mr-2" />
                {error}
              </div>
            )}

            {tokenData && !isExpired ? (
              <>
                {/* Токен */}
                <div className="relative mb-4">
                  <div className="bg-[#2a2a2a]/50 border border-[#FFAA00]/30 rounded-lg p-4 pr-32 font-mono text-sm break-all text-[#FFFF55] text-left min-h-[60px] flex items-center select-all">
                    {tokenData.token}
                  </div>
                  <button
                    id="copy-token-btn"
                    onClick={copyToken}
                    className={`absolute top-1/2 -translate-y-1/2 right-3 px-3 py-1.5 rounded-md transition-all font-medium text-sm ${
                      copied
                        ? "bg-green-500 text-white"
                        : "bg-[#FFAA00] hover:bg-[#FFFF55] text-black"
                    }`}
                  >
                    <i className={`fas ${copied ? "fa-check" : "fa-copy"} mr-1`} />
                    {copied ? "Скопировано!" : "Копировать"}
                  </button>
                </div>

                {/* Время */}
                <div className="grid grid-cols-2 gap-4 text-sm text-gray-400 mb-6">
                  <div className="bg-[#2a2a2a]/30 rounded-lg p-3">
                    <i className="fas fa-hourglass-half mr-2 text-[#FFAA00]" />
                    Истекает через:{" "}
                    <span
                      className={`font-mono font-bold ${
                        parseInt(countdown) < 2 ? "text-red-400" : "text-[#FFAA00]"
                      }`}
                    >
                      {countdown}
                    </span>
                  </div>
                  <div className="bg-[#2a2a2a]/30 rounded-lg p-3">
                    <i className="fas fa-calendar mr-2 text-[#FFAA00]" />
                    До:{" "}
                    <span className="text-[#FFAA00] font-medium">
                      {new Date(tokenData.expiresAt).toLocaleTimeString("ru-RU", {
                        hour: "2-digit",
                        minute: "2-digit",
                        second: "2-digit",
                      })}
                    </span>
                  </div>
                </div>

                <button
                  id="regenerate-token-btn"
                  onClick={generateToken}
                  disabled={generating}
                  className="text-gray-400 hover:text-[#FFAA00] text-sm transition-colors underline disabled:opacity-50"
                >
                  <i className="fas fa-redo mr-1" />
                  Сгенерировать новый токен
                </button>
              </>
            ) : (
              <>
                {isExpired && (
                  <div className="mb-4 p-3 rounded-xl bg-orange-900/30 border border-orange-500/30 text-orange-300 text-sm">
                    <i className="fas fa-hourglass-end mr-2" />
                    Токен истёк. Сгенерируйте новый.
                  </div>
                )}

                <div className="mb-6">
                  <div className="w-20 h-20 bg-[#FFAA00]/10 rounded-full flex items-center justify-center mx-auto mb-4 border border-[#FFAA00]/30">
                    <i className="fas fa-key text-4xl text-[#FFAA00]" />
                  </div>
                  <p className="text-gray-400">
                    Нажмите кнопку ниже, чтобы сгенерировать токен для входа на сервер
                  </p>
                </div>

                <button
                  id="generate-token-btn"
                  onClick={generateToken}
                  disabled={generating}
                  className="btn-minecraft px-8 py-3 rounded-lg text-lg font-bold hover:scale-105 transition-all duration-300 disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none"
                >
                  {generating ? (
                    <>
                      <i className="fas fa-spinner fa-spin mr-2" />
                      Генерация...
                    </>
                  ) : (
                    <>
                      <i className="fas fa-magic mr-2" />
                      Сгенерировать токен
                    </>
                  )}
                </button>
              </>
            )}
          </div>

          {/* Инструкция */}
          <div className="glass-effect rounded-2xl p-8">
            <h2 className="text-2xl font-bold text-[#FFAA00] mb-6">
              <i className="fas fa-info-circle mr-3" />
              Как использовать токен
            </h2>

            <div className="space-y-4">
              {[
                { step: 1, text: 'Нажмите кнопку "Сгенерировать токен" выше' },
                { step: 2, text: "Скопируйте полученный токен в буфер обмена" },
                {
                  step: 3,
                  text: (
                    <>
                      Подключитесь к серверу Minecraft{" "}
                      <code className="bg-[#2a2a2a]/50 text-[#FFAA00] font-mono px-2 py-0.5 rounded">
                        mc.owyx.site
                      </code>
                    </>
                  ),
                },
                {
                  step: 4,
                  text: (
                    <>
                      Введите команду:{" "}
                      <code className="bg-[#2a2a2a]/50 text-[#FFAA00] font-mono px-2 py-0.5 rounded">
                        /auth ваш_токен
                      </code>
                    </>
                  ),
                },
                { step: 5, text: "Готово! Вы авторизованы на сервере" },
              ].map(({ step, text }) => (
                <div key={step} className="flex items-start gap-4">
                  <div className="bg-[#FFAA00] text-black rounded-full w-8 h-8 flex items-center justify-center font-bold text-sm flex-shrink-0 mt-0.5">
                    {step}
                  </div>
                  <div className="text-gray-300 pt-1">{text}</div>
                </div>
              ))}
            </div>

            <div className="mt-8 p-4 bg-orange-900/20 border border-orange-500/30 rounded-lg">
              <h3 className="text-orange-400 font-bold mb-3">
                <i className="fas fa-exclamation-triangle mr-2" />
                Важные замечания:
              </h3>
              <ul className="text-gray-300 space-y-1.5 text-sm">
                <li className="flex items-start gap-2">
                  <i className="fas fa-circle text-[4px] text-orange-400 mt-2 flex-shrink-0" />
                  Токен действует ограниченное время
                </li>
                <li className="flex items-start gap-2">
                  <i className="fas fa-circle text-[4px] text-orange-400 mt-2 flex-shrink-0" />
                  Не передавайте ваш токен другим игрокам
                </li>
                <li className="flex items-start gap-2">
                  <i className="fas fa-circle text-[4px] text-orange-400 mt-2 flex-shrink-0" />
                  После истечения токена сгенерируйте новый
                </li>
                <li className="flex items-start gap-2">
                  <i className="fas fa-circle text-[4px] text-orange-400 mt-2 flex-shrink-0" />
                  Один токен можно использовать только один раз
                </li>
              </ul>
            </div>
          </div>
        </div>
      </main>
      <Footer />
    </>
  );
}
