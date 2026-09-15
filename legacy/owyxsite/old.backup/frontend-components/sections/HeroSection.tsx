import Link from "next/link";
import ServerStatus from "@/components/server/ServerStatus";
import CopyIPButton from "@/components/server/CopyIPButton";

export default function HeroSection() {
  return (
    <section className="py-20 text-center relative hero-with-skins min-h-screen flex items-center">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 w-full relative z-10">
        <div className="glass-effect rounded-2xl p-8 sm:p-12 mb-12 mx-auto gold-glow max-w-4xl">
          <div className="animate-float">
            <h1
              className="text-5xl sm:text-6xl md:text-7xl font-bold text-[#FFAA00] text-shadow mb-6"
              id="hero-server-name"
            >
              Owyx
            </h1>
            <p
              className="text-lg sm:text-xl md:text-2xl text-gray-300 mb-8 max-w-3xl mx-auto text-shadow"
              id="hero-server-description"
            >
              Приватный Minecraft сервер с дружелюбным сообществом
            </p>
          </div>

          {/* Server status + IP */}
          <div className="mb-8 space-y-6">
            <ServerStatus />

            {/* Separator + IP */}
            <div className="flex items-center justify-center">
              <div className="hidden md:block text-[#FFAA00] text-2xl mr-8">│</div>
              <CopyIPButton ip="play.owyx.site" />
            </div>
          </div>

          {/* CTA */}
          <div>
            <Link
              href="/profile"
              className="btn-minecraft px-10 py-4 rounded-lg text-lg font-bold hover:scale-105 transition-all duration-300 animate-glow"
              id="cta-apply"
            >
              <svg
                className="inline w-5 h-5 mr-2"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M18 9v3m0 0v3m0-3h3m-3 0h-3m-2-5a4 4 0 11-8 0 4 4 0 018 0zM3 20a6 6 0 0112 0v1H3v-1z"
                />
              </svg>
              Подать заявку
            </Link>
            <p className="text-gray-400 text-base mt-4">
              Для подачи заявки необходимо зарегистрироваться в личном кабинете
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}
