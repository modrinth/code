"use client";

import { useState, useRef, useEffect } from "react";
import Link from "next/link";
import Logo from "@/components/ui/Logo";
import LanguageToggle from "@/components/layout/LanguageToggle";
import { useAuth } from "@/hooks/useAuth";
import { useLocale } from "@/hooks/useLocale";
import { resolveSiteAvatarUrl } from "@/lib/avatar";

export default function Header() {
  const { user, logout } = useAuth();
  const { dict } = useLocale();
  const [open, setOpen] = useState(false);
  const [mobileOpen, setMobileOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  const NAV = [
    { href: "/", label: dict.header.home },
    { href: "/download", label: dict.header.download },
  ];

  useEffect(() => {
    function handleClick(e: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    document.addEventListener("click", handleClick);
    return () => document.removeEventListener("click", handleClick);
  }, []);

  useEffect(() => {
    if (!mobileOpen) return;
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") setMobileOpen(false);
    }
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, [mobileOpen]);

  const isStaff = user?.role === "admin" || user?.role === "moderator";

  return (
    <header className="sticky top-0 z-50 border-b border-line/90 bg-bg/75 backdrop-blur-md">
      <div className="max-w-6xl mx-auto px-4 sm:px-6 h-16 flex items-center justify-between gap-3">
        <div className="flex items-center gap-7 min-w-0">
          <Link href="/" className="flex items-center shrink-0" aria-label={dict.header.logoHome}>
            <Logo size={26} wordClassName="text-xl" />
          </Link>
          <nav className="hidden sm:flex items-center gap-5" aria-label="Main">
            {NAV.map((item) => (
              <Link
                key={item.href}
                href={item.href}
                className="text-sm text-muted hover:text-accent transition-colors"
              >
                {item.label}
              </Link>
            ))}
          </nav>
        </div>

        <div className="flex items-center gap-2 sm:gap-3">
          {user ? (
            <>
              <Link href="/download" className="hidden sm:inline-flex btn btn-ghost btn-sm">
                {dict.header.download}
              </Link>
              <div className="relative" ref={dropdownRef}>
                <button
                  type="button"
                  onClick={(e) => {
                    e.stopPropagation();
                    setOpen((v) => !v);
                  }}
                  aria-expanded={open}
                  aria-haspopup="menu"
                  className="flex items-center gap-2 rounded-[10px] border border-line bg-panel px-3 py-1.5 text-sm hover:border-accent transition-colors cursor-pointer min-h-11"
                >
                  <span className="grid h-7 w-7 place-items-center overflow-hidden rounded-full bg-panel-2">
                    {/* eslint-disable-next-line @next/next/no-img-element */}
                    <img
                      src={resolveSiteAvatarUrl(user.avatar_url)}
                      alt=""
                      className="h-full w-full object-cover"
                    />
                  </span>
                  <span className="max-w-[10rem] truncate hidden sm:inline">
                    {user.nickname || user.email || "Player"}
                  </span>
                </button>
                {open && (
                  <div
                    role="menu"
                    className="absolute right-0 mt-2 w-52 overflow-hidden rounded-[14px] border border-line bg-panel shadow-[0_16px_40px_-20px_rgba(0,0,0,0.75)] fade-up"
                  >
                    <Link href="/profile" className="block px-4 py-3 text-sm hover:bg-panel-2 transition-colors" role="menuitem">
                      {dict.header.cabinet}
                    </Link>
                    {isStaff && (
                      <Link href="/admin" className="block px-4 py-3 text-sm hover:bg-panel-2 transition-colors border-t border-line" role="menuitem">
                        {dict.header.admin}
                      </Link>
                    )}
                    <button
                      type="button"
                      role="menuitem"
                      onClick={() => {
                        setOpen(false);
                        logout();
                      }}
                      className="w-full text-left px-4 py-3 text-sm text-danger hover:bg-panel-2 transition-colors border-t border-line cursor-pointer"
                    >
                      {dict.header.logout}
                    </button>
                  </div>
                )}
              </div>
            </>
          ) : (
            <>
              <Link href="/login" className="hidden sm:inline text-sm text-muted hover:text-accent transition-colors">
                {dict.header.login}
              </Link>
              <Link href="/register" className="hidden sm:inline text-sm text-muted hover:text-accent transition-colors">
                {dict.header.register}
              </Link>
              <Link href="/download" className="btn btn-primary btn-sm">
                {dict.header.download}
              </Link>
            </>
          )}

          <LanguageToggle />

          <button
            type="button"
            className="sm:hidden inline-flex items-center justify-center rounded-[10px] border border-line bg-panel h-11 w-11 text-text hover:border-accent transition-colors cursor-pointer"
            aria-label={mobileOpen ? dict.header.closeMenu : dict.header.openMenu}
            aria-expanded={mobileOpen}
            aria-controls="mobile-nav"
            onClick={() => setMobileOpen((v) => !v)}
          >
            <svg className="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" aria-hidden="true">
              {mobileOpen ? (
                <path strokeLinecap="round" strokeWidth={2} d="M6 6l12 12M18 6L6 18" />
              ) : (
                <path strokeLinecap="round" strokeWidth={2} d="M4 7h16M4 12h16M4 17h16" />
              )}
            </svg>
          </button>
        </div>
      </div>

      {mobileOpen && (
        <nav
          id="mobile-nav"
          className="sm:hidden border-t border-line bg-bg/95 backdrop-blur-md px-4 py-3 flex flex-col gap-1 fade-up"
          aria-label="Mobile"
        >
          {NAV.map((item) => (
            <Link
              key={item.href}
              href={item.href}
              onClick={() => setMobileOpen(false)}
              className="rounded-[10px] px-3 py-3 text-sm text-muted hover:text-accent hover:bg-panel-2 transition-colors"
            >
              {item.label}
            </Link>
          ))}
          {!user && (
            <>
              <Link
                href="/login"
                onClick={() => setMobileOpen(false)}
                className="rounded-[10px] px-3 py-3 text-sm text-muted hover:text-accent hover:bg-panel-2 transition-colors"
              >
                {dict.header.login}
              </Link>
              <Link
                href="/register"
                onClick={() => setMobileOpen(false)}
                className="rounded-[10px] px-3 py-3 text-sm text-muted hover:text-accent hover:bg-panel-2 transition-colors"
              >
                {dict.header.register}
              </Link>
            </>
          )}
        </nav>
      )}
    </header>
  );
}
