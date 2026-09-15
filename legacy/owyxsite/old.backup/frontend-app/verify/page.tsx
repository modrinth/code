"use client";

import { useState, useEffect, Suspense } from "react";
import { useSearchParams } from "next/navigation";
import Link from "next/link";
import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";

type VerifyState = "loading" | "success" | "error";

function VerifyEmailContent() {
  const searchParams = useSearchParams();
  const token = searchParams.get("token");

  const [state, setVerifyState] = useState<VerifyState>("loading");
  const [errorMessage, setErrorMessage] = useState("Произошла ошибка при подтверждении email");

  useEffect(() => {
    if (!token) {
      setVerifyState("error");
      setErrorMessage("Отсутствует токен подтверждения email");
      return;
    }

    async function verifyEmail() {
      try {
        const res = await fetch(`/api/auth/verify-email-token?token=${token}`);
        const result = await res.json();
        if (res.ok) {
          setVerifyState("success");
        } else {
          setVerifyState("error");
          setErrorMessage(result.error ?? "Недействительный токен подтверждения");
        }
      } catch {
        setVerifyState("error");
        setErrorMessage("Ошибка соединения с сервером");
      }
    }

    verifyEmail();
  }, [token]);

  return (
    <div className="w-full max-w-md">
      <div className="glass-effect rounded-2xl p-8 border border-[#FFAA00]/30 gold-glow">
        {state === "loading" && (
          <div className="text-center space-y-4">
            <div className="animate-float inline-block">
              <i className="fas fa-spinner fa-spin text-6xl text-[#FFAA00] block mb-4" />
            </div>
            <h1 className="text-3xl font-bold text-shadow">
              <span className="text-[#FFAA00]">Подтверждение</span>{" "}
              <span className="text-white">Email</span>
            </h1>
            <p className="text-gray-300">Проверяем токен подтверждения...</p>
          </div>
        )}

        {state === "success" && (
          <div className="text-center space-y-6">
            <div className="animate-float inline-block">
              <i className="fas fa-check-circle text-6xl text-green-400 block mb-4" />
            </div>
            <div>
              <h1 className="text-3xl font-bold text-shadow mb-2">
                <span className="text-green-400">Email</span>{" "}
                <span className="text-white">подтверждён!</span>
              </h1>
              <p className="text-gray-300">Ваш email адрес успешно подтверждён</p>
            </div>

            <div className="bg-green-900/30 border border-green-500/30 rounded-xl p-4">
              <p className="text-green-300 text-sm">
                <i className="fas fa-info-circle mr-2" />
                Теперь вы можете войти в свой аккаунт и подать заявку на вступление в сервер
              </p>
            </div>

            <Link
              href="/login"
              className="btn-minecraft inline-block px-8 py-3 rounded-xl text-lg font-bold hover:scale-105 transition-all duration-300"
            >
              <i className="fas fa-sign-in-alt mr-2" />
              Войти в аккаунт
            </Link>
          </div>
        )}

        {state === "error" && (
          <div className="text-center space-y-6">
            <div className="animate-float inline-block">
              <i className="fas fa-times-circle text-6xl text-red-400 block mb-4" />
            </div>
            <div>
              <h1 className="text-3xl font-bold text-shadow mb-2">
                <span className="text-red-400">Ошибка</span>{" "}
                <span className="text-white">подтверждения</span>
              </h1>
              <p className="text-gray-300">{errorMessage}</p>
            </div>

            <div className="bg-red-900/30 border border-red-500/30 rounded-xl p-4">
              <p className="text-red-300 text-sm">
                <i className="fas fa-exclamation-triangle mr-2" />
                Ссылка могла устареть. Попробуйте зарегистрироваться снова или обратитесь к
                администратору.
              </p>
            </div>

            <div className="flex flex-col sm:flex-row gap-3 justify-center">
              <Link
                href="/login"
                className="btn-minecraft inline-block px-6 py-3 rounded-xl font-bold hover:scale-105 transition-all duration-300"
              >
                <i className="fas fa-sign-in-alt mr-2" />
                Войти
              </Link>
              <Link
                href="/register"
                className="inline-block px-6 py-3 rounded-xl border border-[#FFAA00]/50 text-[#FFAA00] hover:border-[#FFAA00] hover:bg-[#FFAA00]/10 transition-all duration-300 font-bold"
              >
                <i className="fas fa-user-plus mr-2" />
                Регистрация
              </Link>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export default function VerifyEmailPage() {
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
          <VerifyEmailContent />
        </Suspense>
      </main>
      <Footer />
    </>
  );
}
