import Link from "next/link";
import { notFound } from "next/navigation";
import Header from "@/components/layout/Header";
import Footer from "@/components/layout/Footer";

// Draft legal texts for a future open-source public release. Placeholders for
// real юр. данные marked as [указать позже]. Not legal advice; a reasonable
// skeleton, not a copy of anyone's EULA.
type Doc = { title: string; intro: string; sections: { h: string; p: string }[] };

const DOCS: Record<string, Doc> = {
  terms: {
    title: "Пользовательское соглашение",
    intro:
      "Черновик для будущего публичного релиза Owyx. Используя сайт и лаунчер Owyx, вы соглашаетесь с условиями ниже. Реквизиты правообладателя — [указать позже].",
    sections: [
      { h: "1. О сервисе", p: "Owyx — лаунчер Minecraft и сайт-аккаунт. Игра возможна без аккаунта (по нику); аккаунт даёт скин и профильные плюшки." },
      { h: "2. Аккаунт", p: "Вы отвечаете за сохранность пароля и действия под своим аккаунтом. Мы можем ограничить доступ при нарушении правил сервера или закона." },
      { h: "3. Контент и поведение", p: "Запрещены обход блокировок, вредоносные действия, нарушение прав третьих лиц. Ник и скин не должны нарушать правила сообщества." },
      { h: "4. Отказ от гарантий", p: "Сервис предоставляется «как есть». Мы стремимся к стабильности, но не гарантируем бесперебойную работу." },
      { h: "5. Изменения", p: "Условия могут обновляться; актуальная версия — на этой странице. Дата вступления в силу — [указать позже]." },
    ],
  },
  privacy: {
    title: "Политика конфиденциальности",
    intro:
      "Черновик для будущего публичного релиза Owyx. Оператор персональных данных — [указать позже].",
    sections: [
      { h: "1. Какие данные", p: "Ник, email, хеш пароля, роль, дата регистрации, загруженный скин. Технические логи (IP, время) — для безопасности." },
      { h: "2. Зачем", p: "Для входа, профиля, косметики и связи лаунчера с сайтом. Мы не продаём ваши данные." },
      { h: "3. Email", p: "Используется для входа, подтверждения адреса и смены почты/пароля. Рассылки — только по согласию (в разработке)." },
      { h: "4. Хранение и удаление", p: "Данные хранятся, пока существует аккаунт. Запрос на удаление — через контакт [указать позже]." },
      { h: "5. Третьи стороны", p: "Discord/капча/почтовый провайдер обрабатывают данные по своим политикам, когда вы ими пользуетесь." },
    ],
  },
  eula: {
    title: "Лицензия лаунчера (EULA)",
    intro:
      "Черновик лицензионного соглашения лаунчера Owyx для будущего open-source релиза. Minecraft — товарный знак Mojang/Microsoft; Owyx не аффилирован с ними.",
    sections: [
      { h: "1. Лицензия", p: "Вам предоставляется право использовать лаунчер Owyx для личной игры. Условия открытого исходного кода — [указать лицензию позже, напр. MIT/GPL]." },
      { h: "2. Ограничения", p: "Не выдавайте лаунчер за официальный клиент Mojang и не используйте его для обхода лицензии Minecraft." },
      { h: "3. Сторонний код", p: "Лаунчер использует идеи и код open-source клиента Modrinth (Theseus) — форк под себя, не аффилирован с Modrinth." },
      { h: "4. Обновления", p: "Лаунчер может обновляться. Мы не несём ответственности за модификации из непроверенных источников." },
    ],
  },
  offer: {
    title: "Публичная оферта",
    intro:
      "Заглушка. Платные функции пока не предоставляются. Реквизиты и условия оплаты — [указать позже].",
    sections: [
      { h: "1. Предмет", p: "На текущем этапе Owyx не продаёт товары и услуги. Раздел появится, когда будут платные возможности." },
      { h: "2. Возвраты", p: "Условия возврата будут описаны здесь при запуске платных функций." },
    ],
  },
};

export function generateStaticParams() {
  return Object.keys(DOCS).map((slug) => ({ slug }));
}

export async function generateMetadata({ params }: { params: Promise<{ slug: string }> }) {
  const { slug } = await params;
  const doc = DOCS[slug];
  return { title: doc ? `${doc.title} — Owyx` : "Owyx" };
}

export default async function LegalPage({ params }: { params: Promise<{ slug: string }> }) {
  const { slug } = await params;
  const doc = DOCS[slug];
  if (!doc) notFound();

  return (
    <>
      <Header />
      <main id="main-content" className="relative flex-1">
        <article className="max-w-3xl mx-auto px-4 sm:px-6 py-16">
          <p className="text-xs text-muted mb-2">
            <Link href="/" className="link-accent">Главная</Link> · Документы
          </p>
          <h1 className="text-3xl sm:text-4xl font-bold tracking-tight mb-3">{doc.title}</h1>
          <div className="panel p-4 mb-8 text-sm text-muted">
            <span className="badge badge-accent mr-2">черновик</span>
            {doc.intro}
          </div>
          <div className="space-y-6">
            {doc.sections.map((s) => (
              <section key={s.h}>
                <h2 className="text-lg font-semibold text-text mb-1">{s.h}</h2>
                <p className="text-sm text-muted leading-relaxed">{s.p}</p>
              </section>
            ))}
          </div>
        </article>
      </main>
      <Footer />
    </>
  );
}
