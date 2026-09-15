"use client";

import { useEffect, useState, useRef } from "react";
import Link from "next/link";

interface UserData {
  nickname?: string;
  email?: string;
  avatar_url?: string;
  role?: string;
}

export default function Header() {
  const [user, setUser] = useState<UserData | null>(null);
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    checkAuth();
  }, []);

  // Close dropdown on outside click
  useEffect(() => {
    function handleClick(e: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setDropdownOpen(false);
      }
    }
    document.addEventListener("click", handleClick);
    return () => document.removeEventListener("click", handleClick);
  }, []);

  async function checkAuth() {
    try {
      const token = localStorage.getItem("auth_token");
      const tokenExpires = localStorage.getItem("token_expires");

      if (token && tokenExpires) {
        if (new Date() > new Date(tokenExpires)) {
          localStorage.removeItem("auth_token");
          localStorage.removeItem("remember_me");
          localStorage.removeItem("token_expires");
          setUser(null);
          return;
        }
      }

      if (!token) {
        setUser(null);
        return;
      }

      const res = await fetch("/api/auth/verify", {
        headers: { Authorization: `Bearer ${token}` },
      });

      if (res.ok) {
        const data = await res.json();
        setUser(data.user);
      } else {
        localStorage.removeItem("auth_token");
        localStorage.removeItem("remember_me");
        localStorage.removeItem("token_expires");
        setUser(null);
      }
    } catch {
      setUser(null);
    }
  }

  async function handleLogout() {
    const token = localStorage.getItem("auth_token");
    if (token) {
      try {
        await fetch("/api/auth/logout", {
          method: "POST",
          headers: { Authorization: `Bearer ${token}` },
        });
      } catch {
        // ignore
      }
    }
    localStorage.removeItem("auth_token");
    localStorage.removeItem("remember_me");
    localStorage.removeItem("token_expires");
    setUser(null);
    setDropdownOpen(false);
  }

  return (
    <nav className="glass-effect sticky top-0 z-50 border-b border-mc-gold/30">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex justify-between items-center h-16">
          {/* Logo */}
          <Link href="/" className="flex items-center">
            <span className="text-2xl font-bold text-[#FFFF55] text-shadow">
              Owyx
            </span>
          </Link>

          {/* Center nav */}
          <div className="hidden sm:flex items-center space-x-6">
            <Link
              href="/"
              className="nav-item text-[#FFAA00] font-medium text-shadow"
            >
              Главная
            </Link>
            <Link
              href="/online"
              className="nav-item text-white hover:text-[#FFAA00] transition-colors text-shadow"
            >
              Онлайн
            </Link>
            <Link
              href="/forum"
              className="nav-item text-white hover:text-[#FFAA00] transition-colors text-shadow"
            >
              Форум
            </Link>
            <Link
              href="/chat"
              className="nav-item text-white hover:text-[#FFAA00] transition-colors text-shadow"
            >
              Чат
            </Link>
            <Link
              href="/shop"
              className="nav-item text-white hover:text-[#FFAA00] transition-colors text-shadow"
            >
              Магазин
            </Link>
            <a
              href="https://map.owyx.site/"
              target="_blank"
              rel="noopener noreferrer"
              className="nav-item text-white hover:text-[#FFAA00] transition-colors text-shadow"
            >
              Карта
            </a>
          </div>

          {/* Right side */}
          <div className="flex items-center space-x-4">
            {/* Social links */}
            <div className="flex items-center space-x-2">
              <a
                href="https://discord.gg/owyx"
                target="_blank"
                rel="noopener noreferrer"
                className="text-gray-400 hover:text-purple-400 transition-colors text-lg"
                title="Discord"
                id="discord-link"
              >
                <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M20.317 4.37a19.791 19.791 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.736 19.736 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 0 0 .031.057 19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028c.462-.63.874-1.295 1.226-1.994a.076.076 0 0 0-.041-.106 13.107 13.107 0 0 1-1.872-.892.077.077 0 0 1-.008-.128 10.2 10.2 0 0 0 .372-.292.074.074 0 0 1 .077-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 0 1 .078.01c.12.098.246.198.373.292a.077.077 0 0 1-.006.127 12.299 12.299 0 0 1-1.873.892.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.839 19.839 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.03zM8.02 15.33c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.956 2.418-2.157 2.418zm7.975 0c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.955-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.946 2.418-2.157 2.418z" />
                </svg>
              </a>
              <a
                href="https://t.me/owyx"
                target="_blank"
                rel="noopener noreferrer"
                className="text-gray-400 hover:text-blue-400 transition-colors text-lg"
                title="Telegram"
                id="telegram-link"
              >
                <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M11.944 0A12 12 0 0 0 0 12a12 12 0 0 0 12 12 12 12 0 0 0 12-12A12 12 0 0 0 12 0 12 12 0 0 0 11.944 0Zm4.962 7.224c.1-.002.321.023.465.14a.506.506 0 0 1 .171.325c.016.093.036.306.02.472-.18 1.898-.962 6.502-1.36 8.627-.168.9-.499 1.201-.82 1.23-.696.065-1.225-.46-1.9-.902-1.056-.693-1.653-1.124-2.678-1.8-1.185-.78-.417-1.21.258-1.91.177-.184 3.247-2.977 3.307-3.23.007-.032.014-.15-.056-.212s-.174-.041-.249-.024c-.106.024-1.793 1.14-5.061 3.345-.48.33-.913.49-1.302.48-.428-.008-1.252-.241-1.865-.44-.752-.245-1.349-.374-1.297-.789.027-.216.325-.437.893-.663 3.498-1.524 5.83-2.529 6.998-3.014 3.332-1.386 4.025-1.627 4.476-1.635z" />
                </svg>
              </a>
            </div>

            {/* Divider */}
            <div className="h-6 w-px bg-[#FFAA00]/30" />

            {/* Auth buttons */}
            {user ? (
              <div className="relative" ref={dropdownRef}>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    setDropdownOpen(!dropdownOpen);
                  }}
                  className="flex items-center space-x-2 bg-gradient-to-r from-[#FFAA00] to-yellow-500 text-[#1a1a1a] px-4 py-2 rounded-lg font-bold hover:scale-105 transition-all duration-300 shadow-lg hover:shadow-[#FFAA00]/50 cursor-pointer"
                  id="user-profile-btn"
                >
                  <div className="w-8 h-8 bg-[#0a0a0a] rounded-full overflow-hidden border-2 border-[#FFAA00]">
                    <img
                      src={user.avatar_url || "/images/default-avatar.png"}
                      alt="Аватар"
                      className="w-full h-full object-cover"
                    />
                  </div>
                  <span className="font-medium">
                    {user.nickname || user.email || "Игрок"}
                  </span>
                </button>

                {dropdownOpen && (
                  <div className="absolute top-full right-0 mt-2 w-48 bg-[#2a2a2a] rounded-lg shadow-xl border border-[#FFAA00]/30 dropdown-animate z-60">
                    <Link
                      href="/profile"
                      className="block px-4 py-3 text-white hover:bg-[#404040] transition-colors border-b border-[#FFAA00]/20"
                      id="profile-dropdown-link"
                    >
                      <svg className="inline w-4 h-4 mr-2 text-[#FFAA00]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                      </svg>
                      Личный кабинет
                    </Link>
                    {(user.role === "admin" || user.role === "moderator") && (
                      <Link
                        href="/admin"
                        className="block px-4 py-3 text-white hover:bg-[#404040] transition-colors border-b border-[#FFAA00]/20"
                        id="admin-dropdown-link"
                      >
                        <svg className="inline w-4 h-4 mr-2 text-[#FFAA00]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4M7.835 4.697a3.42 3.42 0 001.946-.806 3.42 3.42 0 014.438 0 3.42 3.42 0 001.946.806 3.42 3.42 0 013.138 3.138 3.42 3.42 0 00.806 1.946 3.42 3.42 0 010 4.438 3.42 3.42 0 00-.806 1.946 3.42 3.42 0 01-3.138 3.138 3.42 3.42 0 00-1.946.806 3.42 3.42 0 01-4.438 0 3.42 3.42 0 00-1.946-.806 3.42 3.42 0 01-3.138-3.138 3.42 3.42 0 00-.806-1.946 3.42 3.42 0 010-4.438 3.42 3.42 0 00.806-1.946 3.42 3.42 0 013.138-3.138z" />
                        </svg>
                        Админ панель
                      </Link>
                    )}
                    <button
                      onClick={handleLogout}
                      className="w-full text-left px-4 py-3 text-red-400 hover:bg-[#404040] transition-colors cursor-pointer"
                      id="logout-btn"
                    >
                      <svg className="inline w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
                      </svg>
                      Выйти
                    </button>
                  </div>
                )}
              </div>
            ) : (
              <Link
                href="/login"
                className="bg-gradient-to-r from-[#2a2a2a] to-[#404040] text-[#FFAA00] px-6 py-2 rounded-lg border border-[#FFAA00]/30 hover:from-[#FFAA00] hover:to-[#FFFF55] hover:text-[#0a0a0a] transition-all duration-300 font-medium shadow-lg"
                id="login-btn"
              >
                <svg className="inline w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 16l-4-4m0 0l4-4m-4 4h14m-5 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h7a3 3 0 013 3v1" />
                </svg>
                Войти
              </Link>
            )}
          </div>
        </div>
      </div>
    </nav>
  );
}
