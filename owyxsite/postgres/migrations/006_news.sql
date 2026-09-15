-- News/updates shown on the home page and manageable from the admin panel.
-- Idempotent: safe to run repeatedly.
--   psql -U owyx_user -d owyx_db -f owyxsite/postgres/migrations/006_news.sql

CREATE TABLE IF NOT EXISTS public.news (
    id SERIAL PRIMARY KEY,
    title character varying(200) NOT NULL,
    tag character varying(40) DEFAULT 'Новость',
    summary text NOT NULL DEFAULT '',
    published boolean DEFAULT true,
    author_id integer,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);

-- Seed a couple of useful items on a fresh DB (only if empty).
INSERT INTO public.news (title, tag, summary, published)
SELECT * FROM (VALUES
    ('Вход по аккаунту Owyx', 'Лаунчер',
     'В лаунчере можно войти аккаунтом сайта — ник, роль и скин подтягиваются автоматически. Offline-игра по нику осталась.', true),
    ('Скины и плащи', 'Косметика',
     'Загружай свой скин в профиле — лаунчер подхватит его при входе. Плащи и профильные плюшки на подходе.', true)
) AS seed(title, tag, summary, published)
WHERE NOT EXISTS (SELECT 1 FROM public.news);
