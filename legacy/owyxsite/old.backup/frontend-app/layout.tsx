import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";

const inter = Inter({
  variable: "--font-geist-sans",
  subsets: ["latin", "cyrillic"],
});

export const metadata: Metadata = {
  title: "Owyx — Приватный Minecraft сервер",
  description:
    "Присоединяйтесь к Owyx! Приватный Minecraft сервер с дружелюбным сообществом и качественным геймплеем. Вход только по заявке.",
  keywords: ["minecraft", "сервер", "приватный", "owyx", "геймплей", "сообщество"],
  authors: [{ name: "ebluffy" }],
  openGraph: {
    title: "Owyx — Приватный Minecraft сервер",
    description: "Приватный Minecraft сервер с дружелюбным сообществом и качественным геймплеем.",
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
    <html lang="ru" className={`${inter.variable} h-full antialiased`}>
      <body className="minecraft-bg min-h-screen overflow-x-hidden relative flex flex-col">
        {children}
      </body>
    </html>
  );
}
