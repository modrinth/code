import Link from "next/link";

export default function Footer() {
  return (
    <footer className="glass-effect border-t border-[#FFAA00]/30 py-12 mt-auto">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex flex-col md:flex-row justify-between items-center space-y-6 md:space-y-0">
          {/* Server name */}
          <div className="text-center md:text-left">
            <h3 className="text-2xl font-bold text-[#FFFF55] mb-2 text-shadow">
              <svg className="inline w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
              </svg>
              Owyx
            </h3>
            <p className="text-gray-300 leading-relaxed max-w-md">
              Приватный Minecraft сервер с качественным геймплеем и дружелюбным
              сообществом.
            </p>
          </div>

          {/* Nav links */}
          <div className="flex flex-wrap justify-center md:justify-end items-center space-x-8">
            <Link
              href="/profile"
              className="text-gray-300 hover:text-[#FFFF55] transition-colors font-medium"
            >
              Подать заявку
            </Link>
            <Link
              href="/profile"
              className="text-gray-300 hover:text-[#FFFF55] transition-colors font-medium"
            >
              Личный кабинет
            </Link>
            <Link
              href="/online"
              className="text-gray-300 hover:text-[#FFFF55] transition-colors font-medium"
            >
              Онлайн
            </Link>
            <a
              href="https://map.owyx.site/"
              target="_blank"
              rel="noopener noreferrer"
              className="text-gray-300 hover:text-[#FFFF55] transition-colors font-medium"
            >
              Карта
            </a>
          </div>
        </div>

        <div className="border-t border-[#FFAA00]/30 mt-8 pt-8 text-center">
          <p className="text-gray-400">
            Owyx © {new Date().getFullYear()}, Создано с{" "}
            <span className="text-[#FFAA00]">❤️</span> от{" "}
            <strong className="text-[#FFFF55]">ebluffy</strong>
          </p>
        </div>
      </div>
    </footer>
  );
}
