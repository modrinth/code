import type { Metadata } from "next";
import { Geist_Mono, Onest, Unbounded } from "next/font/google";
import { AuthProvider } from "@/hooks/useAuth";
import { LocaleProvider } from "@/hooks/useLocale";
import SkipLink from "@/components/layout/SkipLink";
import SpaceParticles from "@/components/brand/SpaceParticles";
import "./globals.css";

/* Onest = UI/body with Cyrillic (PR #27). Unbounded = display headings. */
const onest = Onest({
  variable: "--font-onest",
  subsets: ["latin", "cyrillic"],
  display: "swap",
});

const unbounded = Unbounded({
  variable: "--font-unbounded",
  subsets: ["latin", "cyrillic"],
  display: "swap",
  weight: ["500", "600", "700", "800"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
  display: "swap",
});

export const metadata: Metadata = {
  title: "Owyx™",
  description: "Owyx™ — Minecraft launcher and site account. Download, sign in, play.",
  keywords: ["owyx", "minecraft", "launcher", "owyx.site"],
  authors: [{ name: "ebluffy" }],
  icons: {
    icon: "/favicon-owyx.svg",
  },
  openGraph: {
    title: "Owyx™",
    description: "Owyx™ launcher and site account.",
    type: "website",
    locale: "ru_RU",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html
      lang="ru"
      className={`${onest.variable} ${unbounded.variable} ${geistMono.variable} h-full antialiased`}
    >
      <body className="min-h-screen overflow-x-clip flex flex-col">
        <div className="owyx-space" aria-hidden="true" />
        <div className="owyx-vignette" aria-hidden="true" />
        <SpaceParticles />
        <LocaleProvider>
          <SkipLink />
          <AuthProvider>{children}</AuthProvider>
        </LocaleProvider>
      </body>
    </html>
  );
}
