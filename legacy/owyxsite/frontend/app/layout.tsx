import type { Metadata } from "next";
import { DM_Sans, Geist_Mono, Outfit } from "next/font/google";
import { AuthProvider } from "@/hooks/useAuth";
import "./globals.css";

/* Display = Outfit (geometric, not Inter). Body = DM Sans — matches launcher. */
const outfit = Outfit({
  variable: "--font-outfit",
  subsets: ["latin", "latin-ext"],
  display: "swap",
});

const dmSans = DM_Sans({
  variable: "--font-dm-sans",
  subsets: ["latin", "latin-ext"],
  display: "swap",
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
  display: "swap",
});

export const metadata: Metadata = {
  title: "Owyx — приватный Minecraft",
  description:
    "Owyx — экосистема приватного Minecraft: скачай лаунчер и играй, или войди аккаунтом Owyx для скина и плюшек. Без заявок.",
  keywords: ["owyx", "minecraft", "приватный сервер", "лаунчер", "своё сообщество"],
  authors: [{ name: "ebluffy" }],
  openGraph: {
    title: "Owyx — приватный Minecraft",
    description:
      "Скачай лаунчер Owyx → играй по нику или войди аккаунтом сайта.",
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
      className={`${outfit.variable} ${dmSans.variable} ${geistMono.variable} h-full antialiased`}
    >
      <body className="min-h-screen overflow-x-clip flex flex-col">
        <a href="#main-content" className="skip-link">
          К содержимому
        </a>
        <div className="owyx-space" aria-hidden="true" />
        <AuthProvider>{children}</AuthProvider>
      </body>
    </html>
  );
}
