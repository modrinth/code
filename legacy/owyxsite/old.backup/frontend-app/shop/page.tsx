"use client";

import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";

const PLACEHOLDER_ITEMS = [
  {
    id: 1,
    name: "VIP на 30 дней",
    price: "199 ₽",
    desc: "Префикс, цветной ник, доступ к /fly в лобби",
  },
  {
    id: 2,
    name: "Набор ресурсов",
    price: "99 ₽",
    desc: "Стартовый кит: инструменты, еда, блоки",
  },
  {
    id: 3,
    name: "Донат серверу",
    price: "от 50 ₽",
    desc: "Поддержка развития — через Boosty (скоро)",
  },
];

export default function ShopPage() {
  return (
    <>
      <Header />
      <main className="min-h-[calc(100vh-64px)] py-10 px-4">
        <div className="max-w-4xl mx-auto">
          <h1 className="text-3xl font-bold text-[#FFFF55] text-shadow mb-2">Магазин</h1>
          <p className="text-gray-400 mb-8">
            Каталог привилегий. Оплата (Boosty / ЮKassa) будет подключена на этапе 5.
          </p>

          <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-4">
            {PLACEHOLDER_ITEMS.map((item) => (
              <div
                key={item.id}
                className="glass-effect rounded-xl p-6 border border-[#FFAA00]/20 flex flex-col"
              >
                <h2 className="text-lg font-bold text-[#FFAA00] mb-2">{item.name}</h2>
                <p className="text-gray-400 text-sm flex-1 mb-4">{item.desc}</p>
                <div className="flex items-center justify-between">
                  <span className="text-[#FFFF55] font-bold">{item.price}</span>
                  <button
                    disabled
                    className="px-4 py-2 rounded-lg bg-[#2a2a2a] text-gray-500 text-sm border border-[#FFAA00]/20 cursor-not-allowed"
                    title="Платежи пока не подключены"
                  >
                    Скоро
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      </main>
      <Footer />
    </>
  );
}
