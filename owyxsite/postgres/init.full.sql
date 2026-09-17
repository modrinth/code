--
-- PostgreSQL database dump
--

\restrict sM3QiBjqwUNDlfPEt5IU9ijbKAsxY9oDXmqmVvGwhGtVYa9bbVR9GJxDIFcgAKX

-- Dumped from database version 14.20 (Ubuntu 14.20-0ubuntu0.22.04.1)
-- Dumped by pg_dump version 14.20 (Ubuntu 14.20-0ubuntu0.22.04.1)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: uuid-ossp; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;


--
-- Name: EXTENSION "uuid-ossp"; Type: COMMENT; Schema: -; Owner: 
--

COMMENT ON EXTENSION "uuid-ossp" IS 'generate universally unique identifiers (UUIDs)';


--
-- Name: check_ban_expiry(); Type: FUNCTION; Schema: public; Owner: owyx
--

CREATE FUNCTION public.check_ban_expiry() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
            BEGIN
                IF NEW.ban_until IS NOT NULL AND NEW.ban_until <= NOW() THEN
                    NEW.is_banned = FALSE;
                    NEW.ban_reason = NULL;
                    NEW.ban_until = NULL;
                END IF;
                RETURN NEW;
            END;
            $$;


ALTER FUNCTION public.check_ban_expiry() OWNER TO owyx;

--
-- Name: sync_user_total_minutes(); Type: FUNCTION; Schema: public; Owner: owyx
--

CREATE FUNCTION public.sync_user_total_minutes() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE users 
    SET total_minutes = NEW.time_played_minutes
    WHERE id = NEW.user_id;
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.sync_user_total_minutes() OWNER TO owyx;

--
-- Name: update_forum_counts(); Type: FUNCTION; Schema: public; Owner: root
--

CREATE FUNCTION public.update_forum_counts() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- При создании нового поста обновляем счётчики темы
    IF TG_OP = 'INSERT' AND TG_TABLE_NAME = 'forum_posts' THEN
        UPDATE forum_topics 
        SET 
            posts_count = posts_count + 1,
            last_post_at = NEW.created_at,
            updated_at = NOW()
        WHERE id = NEW.topic_id;
        
        -- Обновляем счётчики категории
        UPDATE forum_categories
        SET posts_count = posts_count + 1
        WHERE id = (SELECT category_id FROM forum_topics WHERE id = NEW.topic_id);
        
    -- При удалении поста уменьшаем счётчики
    ELSIF TG_OP = 'DELETE' AND TG_TABLE_NAME = 'forum_posts' THEN
        UPDATE forum_topics 
        SET posts_count = GREATEST(posts_count - 1, 0)
        WHERE id = OLD.topic_id;
        
        UPDATE forum_categories
        SET posts_count = GREATEST(posts_count - 1, 0)
        WHERE id = (SELECT category_id FROM forum_topics WHERE id = OLD.topic_id);
        
    -- При создании темы обновляем счётчик категории
    ELSIF TG_OP = 'INSERT' AND TG_TABLE_NAME = 'forum_topics' THEN
        UPDATE forum_categories
        SET topics_count = topics_count + 1
        WHERE id = NEW.category_id;
        
    -- При удалении темы уменьшаем счётчик
    ELSIF TG_OP = 'DELETE' AND TG_TABLE_NAME = 'forum_topics' THEN
        UPDATE forum_categories
        SET topics_count = GREATEST(topics_count - 1, 0)
        WHERE id = OLD.category_id;
    END IF;
    
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.update_forum_counts() OWNER TO root;

--
-- Name: update_post_votes(); Type: FUNCTION; Schema: public; Owner: root
--

CREATE FUNCTION public.update_post_votes() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        IF NEW.vote_type = 'upvote' THEN
            UPDATE forum_posts SET upvotes = upvotes + 1 WHERE id = NEW.post_id;
        ELSE
            UPDATE forum_posts SET downvotes = downvotes + 1 WHERE id = NEW.post_id;
        END IF;
    ELSIF TG_OP = 'UPDATE' THEN
        -- Если изменили тип голоса
        IF OLD.vote_type = 'upvote' AND NEW.vote_type = 'downvote' THEN
            UPDATE forum_posts 
            SET upvotes = GREATEST(upvotes - 1, 0), downvotes = downvotes + 1 
            WHERE id = NEW.post_id;
        ELSIF OLD.vote_type = 'downvote' AND NEW.vote_type = 'upvote' THEN
            UPDATE forum_posts 
            SET downvotes = GREATEST(downvotes - 1, 0), upvotes = upvotes + 1 
            WHERE id = NEW.post_id;
        END IF;
    ELSIF TG_OP = 'DELETE' THEN
        IF OLD.vote_type = 'upvote' THEN
            UPDATE forum_posts SET upvotes = GREATEST(upvotes - 1, 0) WHERE id = OLD.post_id;
        ELSE
            UPDATE forum_posts SET downvotes = GREATEST(downvotes - 1, 0) WHERE id = OLD.post_id;
        END IF;
    END IF;
    
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.update_post_votes() OWNER TO root;

--
-- Name: update_user_age_from_application(); Type: FUNCTION; Schema: public; Owner: owyx
--

CREATE FUNCTION public.update_user_age_from_application() RETURNS trigger
    LANGUAGE plpgsql
    AS $_$
      BEGIN
          IF NEW.status = 'approved' AND NEW.age IS NOT NULL AND NEW.age ~ '^[0-9]+$' THEN
              UPDATE users
              SET age = NEW.age::integer
              WHERE id = NEW.user_id AND age IS NULL;
          END IF;
          RETURN NEW;
      END;
      $_$;


ALTER FUNCTION public.update_user_age_from_application() OWNER TO owyx;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: admin_logs; Type: TABLE; Schema: public; Owner: root
--

CREATE TABLE public.admin_logs (
    id integer NOT NULL,
    admin_id integer,
    action character varying(100) NOT NULL,
    details text,
    target_user_id integer,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


ALTER TABLE public.admin_logs OWNER TO root;

--
-- Name: admin_logs_id_seq; Type: SEQUENCE; Schema: public; Owner: root
--

CREATE SEQUENCE public.admin_logs_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.admin_logs_id_seq OWNER TO root;

--
-- Name: admin_logs_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: root
--

ALTER SEQUENCE public.admin_logs_id_seq OWNED BY public.admin_logs.id;


--
-- Name: api_tokens; Type: TABLE; Schema: public; Owner: owyx
--

CREATE TABLE public.api_tokens (
    id integer NOT NULL,
    token_name character varying(100) NOT NULL,
    token_hash text NOT NULL,
    token_prefix character varying(20) NOT NULL,
    user_id integer,
    permissions jsonb DEFAULT '[]'::jsonb,
    is_active boolean DEFAULT true,
    last_used_at timestamp without time zone,
    expires_at timestamp without time zone,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    created_by integer,
    description text
);


ALTER TABLE public.api_tokens OWNER TO owyx;

--
-- Name: TABLE api_tokens; Type: COMMENT; Schema: public; Owner: owyx
--

COMMENT ON TABLE public.api_tokens IS 'Долгосрочные API токены для внешних приложений и плагинов';


--
-- Name: COLUMN api_tokens.token_name; Type: COMMENT; Schema: public; Owner: owyx
--

COMMENT ON COLUMN public.api_tokens.token_name IS 'Человекочитаемое название токена';


--
-- Name: COLUMN api_tokens.token_hash; Type: COMMENT; Schema: public; Owner: owyx
--

COMMENT ON COLUMN public.api_tokens.token_hash IS 'Хеш токена для безопасности';


--
-- Name: COLUMN api_tokens.token_prefix; Type: COMMENT; Schema: public; Owner: owyx
--

COMMENT ON COLUMN public.api_tokens.token_prefix IS 'Первые символы токена для идентификации';


--
-- Name: COLUMN api_tokens.permissions; Type: COMMENT; Schema: public; Owner: owyx
--

COMMENT ON COLUMN public.api_tokens.permissions IS 'JSON массив разрешений для токена';


--
-- Name: COLUMN api_tokens.expires_at; Type: COMMENT; Schema: public; Owner: owyx
--

COMMENT ON COLUMN public.api_tokens.expires_at IS 'Дата истечения токена (NULL = никогда не истекает)';


--
-- Name: api_tokens_id_seq; Type: SEQUENCE; Schema: public; Owner: owyx
--

CREATE SEQUENCE public.api_tokens_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.api_tokens_id_seq OWNER TO owyx;

--
-- Name: api_tokens_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: owyx
--

ALTER SEQUENCE public.api_tokens_id_seq OWNED BY public.api_tokens.id;


--
-- Name: applications; Type: TABLE; Schema: public; Owner: root
--

CREATE TABLE public.applications (
    id integer NOT NULL,
    user_id integer,
    minecraft_nick character varying(50) NOT NULL,
    age character varying(10) NOT NULL,
    discord character varying(100) NOT NULL,
    email character varying(255) NOT NULL,
    experience character varying(500) NOT NULL,
    motivation text NOT NULL,
    plans text NOT NULL,
    status character varying(20) DEFAULT 'pending'::character varying,
    submitted_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    reviewed_at timestamp without time zone,
    reviewed_by integer,
    review_comment text,
    ip_address character varying(45),
    user_agent text
);


ALTER TABLE public.applications OWNER TO root;

--
-- Name: applications_id_seq; Type: SEQUENCE; Schema: public; Owner: root
--

CREATE SEQUENCE public.applications_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.applications_id_seq OWNER TO root;

--
-- Name: applications_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: root
--

ALTER SEQUENCE public.applications_id_seq OWNED BY public.applications.id;


--
-- Name: authplugin; Type: TABLE; Schema: public; Owner: root
--

CREATE TABLE public.authplugin (
    id integer NOT NULL,
    username character varying(32) NOT NULL,
    password_hash character varying(255) NOT NULL,
    uuid character varying(36),
    ip_address character varying(45),
    last_login timestamp without time zone,
    registered_at timestamp without time zone DEFAULT now(),
    login_type integer DEFAULT 0,
    email character varying(128),
    email_verification_code character varying(64),
    email_verification_expires timestamp without time zone,
    failed_attempts integer DEFAULT 0,
    last_failed_attempt timestamp without time zone
);


ALTER TABLE public.authplugin OWNER TO root;

--
-- Name: TABLE authplugin; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON TABLE public.authplugin IS 'Таблица для хранения данных авторизации игроков в плагине';


--
-- Name: COLUMN authplugin.login_type; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.authplugin.login_type IS '0 = обычная авторизация (логин/пароль), 1 = авторизация через сайт';


--
-- Name: COLUMN authplugin.email_verification_code; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.authplugin.email_verification_code IS 'Код подтверждения для смены режима авторизации';


--
-- Name: COLUMN authplugin.email_verification_expires; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.authplugin.email_verification_expires IS 'Время истечения кода подтверждения (обычно 15 минут)';


--
-- Name: COLUMN authplugin.failed_attempts; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.authplugin.failed_attempts IS 'Количество неудачных попыток входа';


--
-- Name: COLUMN authplugin.last_failed_attempt; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.authplugin.last_failed_attempt IS 'Время последней неудачной попытки входа';


--
-- Name: authplugin_id_seq; Type: SEQUENCE; Schema: public; Owner: root
--

CREATE SEQUENCE public.authplugin_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.authplugin_id_seq OWNER TO root;

--
-- Name: authplugin_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: root
--

ALTER SEQUENCE public.authplugin_id_seq OWNED BY public.authplugin.id;


--
-- Name: daily_stats; Type: TABLE; Schema: public; Owner: owyx
--

CREATE TABLE public.daily_stats (
    id integer NOT NULL,
    user_id integer NOT NULL,
    stat_date date NOT NULL,
    playtime_minutes integer DEFAULT 0,
    logins_count integer DEFAULT 0,
    blocks_broken integer DEFAULT 0,
    blocks_placed integer DEFAULT 0,
    distance_walked integer DEFAULT 0,
    deaths_count integer DEFAULT 0,
    mobs_killed integer DEFAULT 0,
    created_at timestamp without time zone DEFAULT now(),
    updated_at timestamp without time zone DEFAULT now()
);


ALTER TABLE public.daily_stats OWNER TO owyx;

--
-- Name: daily_stats_id_seq; Type: SEQUENCE; Schema: public; Owner: owyx
--

CREATE SEQUENCE public.daily_stats_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.daily_stats_id_seq OWNER TO owyx;

--
-- Name: daily_stats_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: owyx
--

ALTER SEQUENCE public.daily_stats_id_seq OWNED BY public.daily_stats.id;


--
-- Name: discord_oauth; Type: TABLE; Schema: public; Owner: owyx
--

CREATE TABLE public.discord_oauth (
    id integer NOT NULL,
    user_id integer NOT NULL,
    discord_id character varying(20) NOT NULL,
    discord_username character varying(100) NOT NULL,
    discord_discriminator character varying(10),
    discord_avatar character varying(100),
    access_token text NOT NULL,
    refresh_token text,
    expires_at timestamp without time zone,
    created_at timestamp without time zone DEFAULT now(),
    updated_at timestamp without time zone DEFAULT now()
);


ALTER TABLE public.discord_oauth OWNER TO owyx;

--
-- Name: discord_oauth_id_seq; Type: SEQUENCE; Schema: public; Owner: owyx
--

CREATE SEQUENCE public.discord_oauth_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.discord_oauth_id_seq OWNER TO owyx;

--
-- Name: discord_oauth_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: owyx
--

ALTER SEQUENCE public.discord_oauth_id_seq OWNED BY public.discord_oauth.id;


--
-- Name: email_templates; Type: TABLE; Schema: public; Owner: root
--

CREATE TABLE public.email_templates (
    id integer NOT NULL,
    template_name character varying(100) NOT NULL,
    template_key character varying(50) NOT NULL,
    template_subject character varying(255) NOT NULL,
    template_html text NOT NULL,
    template_text text,
    is_active boolean DEFAULT true,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_by integer,
    template_variables text
);


ALTER TABLE public.email_templates OWNER TO root;

--
-- Name: email_templates_id_seq; Type: SEQUENCE; Schema: public; Owner: root
--

CREATE SEQUENCE public.email_templates_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.email_templates_id_seq OWNER TO root;

--
-- Name: email_templates_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: root
--

ALTER SEQUENCE public.email_templates_id_seq OWNED BY public.email_templates.id;


--
-- Name: email_verification_tokens; Type: TABLE; Schema: public; Owner: owyx
--

CREATE TABLE public.email_verification_tokens (
    id integer NOT NULL,
    user_id integer NOT NULL,
    token character varying(255) NOT NULL,
    expires_at timestamp without time zone NOT NULL,
    used_at timestamp without time zone,
    created_at timestamp without time zone DEFAULT now(),
    used boolean DEFAULT false
);


ALTER TABLE public.email_verification_tokens OWNER TO owyx;

--
-- Name: email_verification_tokens_id_seq; Type: SEQUENCE; Schema: public; Owner: owyx
--

CREATE SEQUENCE public.email_verification_tokens_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.email_verification_tokens_id_seq OWNER TO owyx;

--
-- Name: email_verification_tokens_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: owyx
--

ALTER SEQUENCE public.email_verification_tokens_id_seq OWNED BY public.email_verification_tokens.id;


--
-- Name: forum_attachments; Type: TABLE; Schema: public; Owner: root
--

CREATE TABLE public.forum_attachments (
    id integer NOT NULL,
    post_id integer,
    topic_id integer,
    uploader_id integer NOT NULL,
    filename character varying(255) NOT NULL,
    original_filename character varying(255) NOT NULL,
    file_path text NOT NULL,
    file_size bigint NOT NULL,
    mime_type character varying(100) NOT NULL,
    created_at timestamp without time zone DEFAULT now(),
    CONSTRAINT attachment_belongs_to_post_or_topic CHECK ((((post_id IS NOT NULL) AND (topic_id IS NULL)) OR ((post_id IS NULL) AND (topic_id IS NOT NULL))))
);


ALTER TABLE public.forum_attachments OWNER TO root;

--
-- Name: TABLE forum_attachments; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON TABLE public.forum_attachments IS 'Файлы, прикреплённые к постам или темам';


--
-- Name: CONSTRAINT attachment_belongs_to_post_or_topic ON forum_attachments; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON CONSTRAINT attachment_belongs_to_post_or_topic ON public.forum_attachments IS 'Вложение должно быть привязано либо к посту, либо к теме';


--
-- Name: forum_attachments_id_seq; Type: SEQUENCE; Schema: public; Owner: root
--

CREATE SEQUENCE public.forum_attachments_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.forum_attachments_id_seq OWNER TO root;

--
-- Name: forum_attachments_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: root
--

ALTER SEQUENCE public.forum_attachments_id_seq OWNED BY public.forum_attachments.id;


--
-- Name: forum_categories; Type: TABLE; Schema: public; Owner: root
--

CREATE TABLE public.forum_categories (
    id integer NOT NULL,
    title character varying(100) NOT NULL,
    slug character varying(100) NOT NULL,
    description text,
    icon character varying(50) DEFAULT '📝'::character varying,
    color character varying(7) DEFAULT '#FFAA00'::character varying,
    parent_id integer,
    "position" integer DEFAULT 0,
    is_locked boolean DEFAULT false,
    created_at timestamp without time zone DEFAULT now(),
    updated_at timestamp without time zone DEFAULT now(),
    created_by integer,
    topics_count integer DEFAULT 0,
    posts_count integer DEFAULT 0
);


ALTER TABLE public.forum_categories OWNER TO root;

--
-- Name: TABLE forum_categories; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON TABLE public.forum_categories IS 'Категории форума с иерархией';


--
-- Name: COLUMN forum_categories.slug; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.forum_categories.slug IS 'URL-friendly название категории';


--
-- Name: COLUMN forum_categories.is_locked; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.forum_categories.is_locked IS 'Запрещено создавать новые темы';


--
-- Name: forum_categories_id_seq; Type: SEQUENCE; Schema: public; Owner: root
--

CREATE SEQUENCE public.forum_categories_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.forum_categories_id_seq OWNER TO root;

--
-- Name: forum_categories_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: root
--

ALTER SEQUENCE public.forum_categories_id_seq OWNED BY public.forum_categories.id;


--
-- Name: forum_post_edits; Type: TABLE; Schema: public; Owner: root
--

CREATE TABLE public.forum_post_edits (
    id integer NOT NULL,
    post_id integer NOT NULL,
    edited_by integer NOT NULL,
    old_content text NOT NULL,
    new_content text NOT NULL,
    edit_reason text,
    created_at timestamp without time zone DEFAULT now()
);


ALTER TABLE public.forum_post_edits OWNER TO root;

--
-- Name: TABLE forum_post_edits; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON TABLE public.forum_post_edits IS 'История редактирования постов для прозрачности';


--
-- Name: forum_post_edits_id_seq; Type: SEQUENCE; Schema: public; Owner: root
--

CREATE SEQUENCE public.forum_post_edits_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.forum_post_edits_id_seq OWNER TO root;

--
-- Name: forum_post_edits_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: root
--

ALTER SEQUENCE public.forum_post_edits_id_seq OWNED BY public.forum_post_edits.id;


--
-- Name: forum_posts; Type: TABLE; Schema: public; Owner: root
--

CREATE TABLE public.forum_posts (
    id integer NOT NULL,
    topic_id integer NOT NULL,
    author_id integer NOT NULL,
    content text NOT NULL,
    parent_post_id integer,
    created_at timestamp without time zone DEFAULT now(),
    updated_at timestamp without time zone DEFAULT now(),
    upvotes integer DEFAULT 0,
    downvotes integer DEFAULT 0,
    is_edited boolean DEFAULT false,
    is_deleted boolean DEFAULT false,
    deleted_by integer,
    deleted_at timestamp without time zone,
    deletion_reason text
);


ALTER TABLE public.forum_posts OWNER TO root;

--
-- Name: TABLE forum_posts; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON TABLE public.forum_posts IS 'Сообщения (посты) в темах форума';


--
-- Name: COLUMN forum_posts.parent_post_id; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.forum_posts.parent_post_id IS 'ID родительского поста для вложенных ответов';


--
-- Name: COLUMN forum_posts.is_deleted; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.forum_posts.is_deleted IS 'Мягкое удаление (soft delete)';


--
-- Name: forum_posts_id_seq; Type: SEQUENCE; Schema: public; Owner: root
--

CREATE SEQUENCE public.forum_posts_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.forum_posts_id_seq OWNER TO root;

--
-- Name: forum_posts_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: root
--

ALTER SEQUENCE public.forum_posts_id_seq OWNED BY public.forum_posts.id;


--
-- Name: forum_topics; Type: TABLE; Schema: public; Owner: root
--

CREATE TABLE public.forum_topics (
    id integer NOT NULL,
    category_id integer NOT NULL,
    title character varying(200) NOT NULL,
    slug character varying(200) NOT NULL,
    content text NOT NULL,
    author_id integer NOT NULL,
    is_pinned boolean DEFAULT false,
    is_locked boolean DEFAULT false,
    is_solved boolean DEFAULT false,
    created_at timestamp without time zone DEFAULT now(),
    updated_at timestamp without time zone DEFAULT now(),
    last_post_at timestamp without time zone DEFAULT now(),
    views_count integer DEFAULT 0,
    posts_count integer DEFAULT 0,
    tags jsonb DEFAULT '[]'::jsonb
);


ALTER TABLE public.forum_topics OWNER TO root;

--
-- Name: TABLE forum_topics; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON TABLE public.forum_topics IS 'Темы (топики) форума';


--
-- Name: COLUMN forum_topics.slug; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.forum_topics.slug IS 'URL-friendly название темы';


--
-- Name: COLUMN forum_topics.is_pinned; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.forum_topics.is_pinned IS 'Закреплённая тема (показывается сверху)';


--
-- Name: COLUMN forum_topics.is_solved; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.forum_topics.is_solved IS 'Отмечена как решённая';


--
-- Name: COLUMN forum_topics.tags; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.forum_topics.tags IS 'Массив тегов в формате JSON';


--
-- Name: forum_topics_id_seq; Type: SEQUENCE; Schema: public; Owner: root
--

CREATE SEQUENCE public.forum_topics_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.forum_topics_id_seq OWNER TO root;

--
-- Name: forum_topics_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: root
--

ALTER SEQUENCE public.forum_topics_id_seq OWNED BY public.forum_topics.id;


--
-- Name: forum_votes; Type: TABLE; Schema: public; Owner: root
--

CREATE TABLE public.forum_votes (
    id integer NOT NULL,
    post_id integer NOT NULL,
    user_id integer NOT NULL,
    vote_type character varying(10) NOT NULL,
    created_at timestamp without time zone DEFAULT now(),
    CONSTRAINT forum_votes_vote_type_check CHECK (((vote_type)::text = ANY ((ARRAY['upvote'::character varying, 'downvote'::character varying])::text[])))
);


ALTER TABLE public.forum_votes OWNER TO root;

--
-- Name: TABLE forum_votes; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON TABLE public.forum_votes IS 'Голоса пользователей за посты (upvote/downvote)';


--
-- Name: forum_votes_id_seq; Type: SEQUENCE; Schema: public; Owner: root
--

CREATE SEQUENCE public.forum_votes_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.forum_votes_id_seq OWNER TO root;

--
-- Name: forum_votes_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: root
--

ALTER SEQUENCE public.forum_votes_id_seq OWNED BY public.forum_votes.id;


--
-- Name: game_sessions; Type: TABLE; Schema: public; Owner: owyx
--

CREATE TABLE public.game_sessions (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    user_id integer NOT NULL,
    player_uuid character varying(36) NOT NULL,
    nickname character varying(16) NOT NULL,
    expires_at timestamp without time zone NOT NULL,
    ip_address character varying(45),
    user_agent text,
    is_active boolean DEFAULT true,
    created_at timestamp without time zone DEFAULT now(),
    last_login timestamp without time zone DEFAULT now()
);


ALTER TABLE public.game_sessions OWNER TO owyx;

--
-- Name: TABLE game_sessions; Type: COMMENT; Schema: public; Owner: owyx
--

COMMENT ON TABLE public.game_sessions IS 'Игровые сессии для авторизованных игроков на сервере';


--
-- Name: COLUMN game_sessions.player_uuid; Type: COMMENT; Schema: public; Owner: owyx
--

COMMENT ON COLUMN public.game_sessions.player_uuid IS 'UUID игрока в Minecraft';


--
-- Name: COLUMN game_sessions.nickname; Type: COMMENT; Schema: public; Owner: owyx
--

COMMENT ON COLUMN public.game_sessions.nickname IS 'Никнейм игрока на момент создания сессии';


--
-- Name: COLUMN game_sessions.is_active; Type: COMMENT; Schema: public; Owner: owyx
--

COMMENT ON COLUMN public.game_sessions.is_active IS 'Активна ли сессия';


--
-- Name: COLUMN game_sessions.last_login; Type: COMMENT; Schema: public; Owner: owyx
--

COMMENT ON COLUMN public.game_sessions.last_login IS 'Время последнего входа с этой сессией';


--
-- Name: game_tokens; Type: TABLE; Schema: public; Owner: owyx
--

CREATE TABLE public.game_tokens (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    user_id integer NOT NULL,
    token_hash text NOT NULL,
    expires_at timestamp without time zone NOT NULL,
    ip_address character varying(45),
    user_agent text,
    is_used boolean DEFAULT false,
    used_at timestamp without time zone,
    created_at timestamp without time zone DEFAULT now()
);


ALTER TABLE public.game_tokens OWNER TO owyx;

--
-- Name: TABLE game_tokens; Type: COMMENT; Schema: public; Owner: owyx
--

COMMENT ON TABLE public.game_tokens IS 'Токены для авторизации игроков на игровом сервере';


--
-- Name: COLUMN game_tokens.token_hash; Type: COMMENT; Schema: public; Owner: owyx
--

COMMENT ON COLUMN public.game_tokens.token_hash IS 'Хеш токена для безопасности';


--
-- Name: COLUMN game_tokens.is_used; Type: COMMENT; Schema: public; Owner: owyx
--

COMMENT ON COLUMN public.game_tokens.is_used IS 'Был ли токен использован для входа';


--
-- Name: COLUMN game_tokens.used_at; Type: COMMENT; Schema: public; Owner: owyx
--

COMMENT ON COLUMN public.game_tokens.used_at IS 'Время использования токена';


--
-- Name: login_logs; Type: TABLE; Schema: public; Owner: root
--

CREATE TABLE public.login_logs (
    id integer NOT NULL,
    user_id integer,
    ip_address character varying(45),
    user_agent text,
    success boolean NOT NULL,
    login_time timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    failure_reason character varying(255)
);


ALTER TABLE public.login_logs OWNER TO root;

--
-- Name: login_logs_id_seq; Type: SEQUENCE; Schema: public; Owner: root
--

CREATE SEQUENCE public.login_logs_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.login_logs_id_seq OWNER TO root;

--
-- Name: login_logs_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: root
--

ALTER SEQUENCE public.login_logs_id_seq OWNED BY public.login_logs.id;


--
-- Name: password_reset_tokens; Type: TABLE; Schema: public; Owner: owyx
--

CREATE TABLE public.password_reset_tokens (
    id integer NOT NULL,
    user_id integer NOT NULL,
    token character varying(255) NOT NULL,
    expires_at timestamp without time zone NOT NULL,
    used_at timestamp without time zone,
    created_at timestamp without time zone DEFAULT now()
);


ALTER TABLE public.password_reset_tokens OWNER TO owyx;

--
-- Name: password_reset_tokens_id_seq; Type: SEQUENCE; Schema: public; Owner: owyx
--

CREATE SEQUENCE public.password_reset_tokens_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.password_reset_tokens_id_seq OWNER TO owyx;

--
-- Name: password_reset_tokens_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: owyx
--

ALTER SEQUENCE public.password_reset_tokens_id_seq OWNED BY public.password_reset_tokens.id;


--
-- Name: player_stats; Type: TABLE; Schema: public; Owner: root
--

CREATE TABLE public.player_stats (
    id integer NOT NULL,
    user_id integer NOT NULL,
    is_time_limited boolean DEFAULT true,
    reputation integer DEFAULT 0,
    total_logins integer DEFAULT 0,
    current_level integer DEFAULT 0,
    time_played_minutes integer DEFAULT 0,
    achievements_count integer DEFAULT 0,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    minecraft_stats jsonb DEFAULT '{}'::jsonb,
    session_count integer DEFAULT 0,
    average_session_duration integer DEFAULT 0,
    longest_session_duration integer DEFAULT 0,
    last_seen timestamp without time zone,
    blocks_broken integer DEFAULT 0,
    blocks_placed integer DEFAULT 0,
    distance_walked integer DEFAULT 0,
    deaths_count integer DEFAULT 0,
    mobs_killed integer DEFAULT 0,
    items_crafted integer DEFAULT 0,
    damage_dealt integer DEFAULT 0,
    damage_taken integer DEFAULT 0,
    food_eaten integer DEFAULT 0,
    jumps_count integer DEFAULT 0,
    online_time_today integer DEFAULT 0,
    online_time_week integer DEFAULT 0,
    online_time_month integer DEFAULT 0,
    active_days_count integer DEFAULT 0,
    last_ip_address inet,
    stats_last_updated timestamp without time zone DEFAULT now()
);


ALTER TABLE public.player_stats OWNER TO root;

--
-- Name: COLUMN player_stats.minecraft_stats; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.player_stats.minecraft_stats IS 'JSON с детальной статистикой из Minecraft';


--
-- Name: COLUMN player_stats.session_count; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.player_stats.session_count IS 'Количество игровых сессий';


--
-- Name: COLUMN player_stats.average_session_duration; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.player_stats.average_session_duration IS 'Средняя продолжительность сессии в минутах';


--
-- Name: COLUMN player_stats.longest_session_duration; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.player_stats.longest_session_duration IS 'Самая длинная сессия в минутах';


--
-- Name: COLUMN player_stats.last_seen; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.player_stats.last_seen IS 'Последний раз был в сети';


--
-- Name: COLUMN player_stats.blocks_broken; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.player_stats.blocks_broken IS 'Количество сломанных блоков';


--
-- Name: COLUMN player_stats.blocks_placed; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.player_stats.blocks_placed IS 'Количество поставленных блоков';


--
-- Name: COLUMN player_stats.distance_walked; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.player_stats.distance_walked IS 'Пройденное расстояние в блоках';


--
-- Name: COLUMN player_stats.deaths_count; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.player_stats.deaths_count IS 'Количество смертей';


--
-- Name: COLUMN player_stats.mobs_killed; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.player_stats.mobs_killed IS 'Количество убитых мобов';


--
-- Name: COLUMN player_stats.items_crafted; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.player_stats.items_crafted IS 'Количество скрафченных предметов';


--
-- Name: COLUMN player_stats.damage_dealt; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.player_stats.damage_dealt IS 'Нанесенный урон';


--
-- Name: COLUMN player_stats.damage_taken; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.player_stats.damage_taken IS 'Полученный урон';


--
-- Name: COLUMN player_stats.food_eaten; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.player_stats.food_eaten IS 'Съедено еды';


--
-- Name: COLUMN player_stats.jumps_count; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.player_stats.jumps_count IS 'Количество прыжков';


--
-- Name: COLUMN player_stats.online_time_today; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.player_stats.online_time_today IS 'Время онлайн сегодня в минутах';


--
-- Name: COLUMN player_stats.online_time_week; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.player_stats.online_time_week IS 'Время онлайн за неделю в минутах';


--
-- Name: COLUMN player_stats.online_time_month; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.player_stats.online_time_month IS 'Время онлайн за месяц в минутах';


--
-- Name: COLUMN player_stats.active_days_count; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.player_stats.active_days_count IS 'Количество активных дней';


--
-- Name: COLUMN player_stats.last_ip_address; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.player_stats.last_ip_address IS 'Последний IP адрес';


--
-- Name: COLUMN player_stats.stats_last_updated; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON COLUMN public.player_stats.stats_last_updated IS 'Время последнего обновления статистики';


--
-- Name: player_stats_id_seq; Type: SEQUENCE; Schema: public; Owner: root
--

CREATE SEQUENCE public.player_stats_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.player_stats_id_seq OWNER TO root;

--
-- Name: player_stats_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: root
--

ALTER SEQUENCE public.player_stats_id_seq OWNED BY public.player_stats.id;


--
-- Name: reputation_log; Type: TABLE; Schema: public; Owner: owyx
--

CREATE TABLE public.reputation_log (
    id integer NOT NULL,
    from_user_id integer,
    to_user_id integer NOT NULL,
    reputation_change integer NOT NULL,
    reason character varying(255),
    comment text,
    created_at timestamp without time zone DEFAULT now()
);


ALTER TABLE public.reputation_log OWNER TO owyx;

--
-- Name: reputation_log_id_seq; Type: SEQUENCE; Schema: public; Owner: owyx
--

CREATE SEQUENCE public.reputation_log_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.reputation_log_id_seq OWNER TO owyx;

--
-- Name: reputation_log_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: owyx
--

ALTER SEQUENCE public.reputation_log_id_seq OWNED BY public.reputation_log.id;


--
-- Name: server_settings; Type: TABLE; Schema: public; Owner: root
--

CREATE TABLE public.server_settings (
    id integer NOT NULL,
    setting_key character varying(100) NOT NULL,
    setting_value text,
    setting_type character varying(20) DEFAULT 'string'::character varying,
    category character varying(50) DEFAULT 'general'::character varying,
    description text,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_by integer
);


ALTER TABLE public.server_settings OWNER TO root;

--
-- Name: server_settings_id_seq; Type: SEQUENCE; Schema: public; Owner: root
--

CREATE SEQUENCE public.server_settings_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.server_settings_id_seq OWNER TO root;

--
-- Name: server_settings_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: root
--

ALTER SEQUENCE public.server_settings_id_seq OWNED BY public.server_settings.id;


--
-- Name: server_status; Type: TABLE; Schema: public; Owner: owyx
--

CREATE TABLE public.server_status (
    id integer NOT NULL,
    server_ip character varying(50) DEFAULT 'localhost'::character varying NOT NULL,
    server_port integer DEFAULT 25565 NOT NULL,
    tps numeric(4,2),
    uptime_seconds bigint,
    max_memory bigint,
    used_memory bigint,
    free_memory bigint,
    online_players integer DEFAULT 0,
    max_players integer DEFAULT 20,
    server_version character varying(50),
    plugins_count integer DEFAULT 0,
    loaded_worlds integer DEFAULT 1,
    updated_at timestamp without time zone DEFAULT now(),
    created_at timestamp without time zone DEFAULT now()
);


ALTER TABLE public.server_status OWNER TO owyx;

--
-- Name: server_status_id_seq; Type: SEQUENCE; Schema: public; Owner: owyx
--

CREATE SEQUENCE public.server_status_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.server_status_id_seq OWNER TO owyx;

--
-- Name: server_status_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: owyx
--

ALTER SEQUENCE public.server_status_id_seq OWNED BY public.server_status.id;


--
-- Name: trust_level_applications; Type: TABLE; Schema: public; Owner: owyx
--

CREATE TABLE public.trust_level_applications (
    id integer NOT NULL,
    user_id integer NOT NULL,
    current_level integer NOT NULL,
    requested_level integer NOT NULL,
    reason text NOT NULL,
    status character varying(20) DEFAULT 'pending'::character varying,
    reviewed_by integer,
    reviewed_at timestamp without time zone,
    review_comment text,
    reputation_score integer,
    hours_played integer,
    email_verified boolean,
    submitted_at timestamp without time zone DEFAULT now()
);


ALTER TABLE public.trust_level_applications OWNER TO owyx;

--
-- Name: trust_level_applications_id_seq; Type: SEQUENCE; Schema: public; Owner: owyx
--

CREATE SEQUENCE public.trust_level_applications_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.trust_level_applications_id_seq OWNER TO owyx;

--
-- Name: trust_level_applications_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: owyx
--

ALTER SEQUENCE public.trust_level_applications_id_seq OWNED BY public.trust_level_applications.id;


--
-- Name: user_achievements; Type: TABLE; Schema: public; Owner: owyx
--

CREATE TABLE public.user_achievements (
    id integer NOT NULL,
    user_id integer,
    achievement_type character varying(100) NOT NULL,
    achievement_data jsonb,
    unlocked_at timestamp without time zone DEFAULT now(),
    created_at timestamp without time zone DEFAULT now()
);


ALTER TABLE public.user_achievements OWNER TO owyx;

--
-- Name: user_achievements_id_seq; Type: SEQUENCE; Schema: public; Owner: owyx
--

CREATE SEQUENCE public.user_achievements_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.user_achievements_id_seq OWNER TO owyx;

--
-- Name: user_achievements_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: owyx
--

ALTER SEQUENCE public.user_achievements_id_seq OWNED BY public.user_achievements.id;


--
-- Name: user_activity; Type: TABLE; Schema: public; Owner: root
--

CREATE TABLE public.user_activity (
    id integer NOT NULL,
    user_id integer NOT NULL,
    activity_type character varying(50) NOT NULL,
    metadata jsonb,
    ip_address character varying(45),
    user_agent text,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    description text NOT NULL
);


ALTER TABLE public.user_activity OWNER TO root;

--
-- Name: user_activity_id_seq; Type: SEQUENCE; Schema: public; Owner: root
--

CREATE SEQUENCE public.user_activity_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.user_activity_id_seq OWNER TO root;

--
-- Name: user_activity_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: root
--

ALTER SEQUENCE public.user_activity_id_seq OWNED BY public.user_activity.id;


--
-- Name: user_reputation; Type: TABLE; Schema: public; Owner: root
--

CREATE TABLE public.user_reputation (
    id integer NOT NULL,
    user_id integer NOT NULL,
    reputation_score integer DEFAULT 0,
    positive_votes integer DEFAULT 0,
    negative_votes integer DEFAULT 0,
    forum_posts integer DEFAULT 0,
    helpful_posts integer DEFAULT 0,
    reported_bugs integer DEFAULT 0,
    community_contributions integer DEFAULT 0,
    warnings_received integer DEFAULT 0,
    temporary_bans integer DEFAULT 0,
    reputation_penalties integer DEFAULT 0,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


ALTER TABLE public.user_reputation OWNER TO root;

--
-- Name: user_reputation_id_seq; Type: SEQUENCE; Schema: public; Owner: root
--

CREATE SEQUENCE public.user_reputation_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.user_reputation_id_seq OWNER TO root;

--
-- Name: user_reputation_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: root
--

ALTER SEQUENCE public.user_reputation_id_seq OWNED BY public.user_reputation.id;


--
-- Name: user_sessions; Type: TABLE; Schema: public; Owner: owyx
--

CREATE TABLE public.user_sessions (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    user_id integer NOT NULL,
    token_hash text NOT NULL,
    expires_at timestamp without time zone NOT NULL,
    ip_address character varying(45),
    user_agent text,
    is_active boolean DEFAULT true,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    last_activity timestamp without time zone DEFAULT now()
);


ALTER TABLE public.user_sessions OWNER TO owyx;

--
-- Name: COLUMN user_sessions.last_activity; Type: COMMENT; Schema: public; Owner: owyx
--

COMMENT ON COLUMN public.user_sessions.last_activity IS 'Время последней активности пользователя';


--
-- Name: users; Type: TABLE; Schema: public; Owner: root
--

CREATE TABLE public.users (
    id integer NOT NULL,
    nickname character varying(16) NOT NULL,
    email character varying(255) NOT NULL,
    password_hash character varying(255) NOT NULL,
    first_name character varying(100),
    role character varying(20) DEFAULT 'user'::character varying,
    trust_level integer DEFAULT 0,
    status character varying(20) DEFAULT 'active'::character varying,
    is_active boolean DEFAULT true,
    is_email_verified boolean DEFAULT false,
    is_banned boolean DEFAULT false,
    registered_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    last_login timestamp without time zone,
    age integer,
    bio text,
    avatar_url character varying(255),
    ban_reason text,
    ban_until timestamp without time zone,
    discord_username character varying(100),
    total_minutes integer DEFAULT 0
);


ALTER TABLE public.users OWNER TO root;

--
-- Name: users_id_seq; Type: SEQUENCE; Schema: public; Owner: root
--

CREATE SEQUENCE public.users_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.users_id_seq OWNER TO root;

--
-- Name: users_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: root
--

ALTER SEQUENCE public.users_id_seq OWNED BY public.users.id;


--
-- Name: admin_logs id; Type: DEFAULT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.admin_logs ALTER COLUMN id SET DEFAULT nextval('public.admin_logs_id_seq'::regclass);


--
-- Name: api_tokens id; Type: DEFAULT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.api_tokens ALTER COLUMN id SET DEFAULT nextval('public.api_tokens_id_seq'::regclass);


--
-- Name: applications id; Type: DEFAULT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.applications ALTER COLUMN id SET DEFAULT nextval('public.applications_id_seq'::regclass);


--
-- Name: authplugin id; Type: DEFAULT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.authplugin ALTER COLUMN id SET DEFAULT nextval('public.authplugin_id_seq'::regclass);


--
-- Name: daily_stats id; Type: DEFAULT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.daily_stats ALTER COLUMN id SET DEFAULT nextval('public.daily_stats_id_seq'::regclass);


--
-- Name: discord_oauth id; Type: DEFAULT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.discord_oauth ALTER COLUMN id SET DEFAULT nextval('public.discord_oauth_id_seq'::regclass);


--
-- Name: email_templates id; Type: DEFAULT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.email_templates ALTER COLUMN id SET DEFAULT nextval('public.email_templates_id_seq'::regclass);


--
-- Name: email_verification_tokens id; Type: DEFAULT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.email_verification_tokens ALTER COLUMN id SET DEFAULT nextval('public.email_verification_tokens_id_seq'::regclass);


--
-- Name: forum_attachments id; Type: DEFAULT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_attachments ALTER COLUMN id SET DEFAULT nextval('public.forum_attachments_id_seq'::regclass);


--
-- Name: forum_categories id; Type: DEFAULT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_categories ALTER COLUMN id SET DEFAULT nextval('public.forum_categories_id_seq'::regclass);


--
-- Name: forum_post_edits id; Type: DEFAULT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_post_edits ALTER COLUMN id SET DEFAULT nextval('public.forum_post_edits_id_seq'::regclass);


--
-- Name: forum_posts id; Type: DEFAULT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_posts ALTER COLUMN id SET DEFAULT nextval('public.forum_posts_id_seq'::regclass);


--
-- Name: forum_topics id; Type: DEFAULT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_topics ALTER COLUMN id SET DEFAULT nextval('public.forum_topics_id_seq'::regclass);


--
-- Name: forum_votes id; Type: DEFAULT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_votes ALTER COLUMN id SET DEFAULT nextval('public.forum_votes_id_seq'::regclass);


--
-- Name: login_logs id; Type: DEFAULT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.login_logs ALTER COLUMN id SET DEFAULT nextval('public.login_logs_id_seq'::regclass);


--
-- Name: password_reset_tokens id; Type: DEFAULT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.password_reset_tokens ALTER COLUMN id SET DEFAULT nextval('public.password_reset_tokens_id_seq'::regclass);


--
-- Name: player_stats id; Type: DEFAULT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.player_stats ALTER COLUMN id SET DEFAULT nextval('public.player_stats_id_seq'::regclass);


--
-- Name: reputation_log id; Type: DEFAULT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.reputation_log ALTER COLUMN id SET DEFAULT nextval('public.reputation_log_id_seq'::regclass);


--
-- Name: server_settings id; Type: DEFAULT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.server_settings ALTER COLUMN id SET DEFAULT nextval('public.server_settings_id_seq'::regclass);


--
-- Name: server_status id; Type: DEFAULT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.server_status ALTER COLUMN id SET DEFAULT nextval('public.server_status_id_seq'::regclass);


--
-- Name: trust_level_applications id; Type: DEFAULT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.trust_level_applications ALTER COLUMN id SET DEFAULT nextval('public.trust_level_applications_id_seq'::regclass);


--
-- Name: user_achievements id; Type: DEFAULT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.user_achievements ALTER COLUMN id SET DEFAULT nextval('public.user_achievements_id_seq'::regclass);


--
-- Name: user_activity id; Type: DEFAULT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.user_activity ALTER COLUMN id SET DEFAULT nextval('public.user_activity_id_seq'::regclass);


--
-- Name: user_reputation id; Type: DEFAULT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.user_reputation ALTER COLUMN id SET DEFAULT nextval('public.user_reputation_id_seq'::regclass);


--
-- Name: users id; Type: DEFAULT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.users ALTER COLUMN id SET DEFAULT nextval('public.users_id_seq'::regclass);


--
-- Data for Name: admin_logs; Type: TABLE DATA; Schema: public; Owner: root
--

COPY public.admin_logs (id, admin_id, action, details, target_user_id, created_at) FROM stdin;
1	1	settings_updated	Обновлено настроек: 3. Категории: general	\N	2025-08-01 23:20:12.815529
2	1	settings_updated	Обновлено настроек: 3. Категории: general	\N	2025-08-01 23:20:13.673856
3	1	application_reviewed	Заявка ebluffy одобрена	1	2025-08-02 06:57:12.119991
6	1	profile_update	Обновлен профиль: имя, возраст, биография	1	2025-08-03 03:58:50.734852
7	1	profile_update	Обновлен профиль: имя, возраст, биография	1	2025-08-03 04:02:14.340621
8	1	email_template_saved	Сохранен email шаблон: welcome (Добро пожаловать)	\N	2025-08-05 09:23:30.942401
9	1	settings_updated	Обновлено настроек: 0. Категории: 	\N	2025-08-05 10:07:21.88836
10	1	settings_updated	Обновлено настроек: 36. Категории: general, applications, trust, security, email	\N	2025-08-05 10:07:25.642819
11	1	settings_updated	Обновлено настроек: 40. Категории: general, system, applications, trust, security, email	\N	2025-08-05 10:07:26.831626
12	1	email_template_tested	Тестирование шаблона "welcome" для пользователя ebluffy (dima2_05@mail.ru)	\N	2025-08-05 10:07:38.730757
13	1	email_template_saved	Сохранен email шаблон: welcome (Добро пожаловать)	\N	2025-08-05 10:30:49.849836
14	1	settings_updated	Обновлено настроек: 0. Категории: 	\N	2025-08-05 10:30:55.063655
15	1	settings_updated	Обновлено настроек: 36. Категории: general, applications, trust, security, email	\N	2025-08-05 10:30:58.388759
16	1	settings_updated	Обновлено настроек: 40. Категории: general, system, applications, trust, security, email	\N	2025-08-05 10:30:59.355001
17	1	email_template_tested	Тестирование шаблона "welcome" для пользователя ebluffy (dima2_05@mail.ru)	\N	2025-08-05 10:31:08.851301
18	1	email_template_saved	Сохранен email шаблон: welcome (Добро пожаловать)	\N	2025-08-05 10:32:46.382237
19	1	email_template_tested	Тестирование шаблона "welcome" для пользователя ebluffy (dima2_05@mail.ru)	\N	2025-08-05 10:32:54.863854
20	1	email_template_saved	Сохранен email шаблон: welcome (Добро пожаловать)	\N	2025-08-05 10:33:18.828487
21	1	email_template_saved	Сохранен email шаблон: verification (Подтверждение email)	\N	2025-08-05 10:41:15.273091
22	1	email_template_tested	Тестирование шаблона "verification" для пользователя ebluffy (dima2_05@mail.ru)	\N	2025-08-05 10:41:29.972596
23	1	email_template_saved	Сохранен email шаблон: verification (Подтверждение email)	\N	2025-08-05 10:46:46.550911
24	1	email_template_tested	Тестирование шаблона "verification" для пользователя ebluffy (dima2_05@mail.ru)	\N	2025-08-05 10:50:55.890247
25	1	settings_updated	Обновлено настроек: 0. Категории: 	\N	2025-08-05 10:53:23.695193
26	1	email_template_saved	Сохранен email шаблон: verification (Подтверждение email)	\N	2025-08-05 10:53:24.651937
27	1	settings_updated	Обновлено настроек: 36. Категории: general, applications, trust, security, email	\N	2025-08-05 10:53:27.407419
28	1	settings_updated	Обновлено настроек: 40. Категории: general, system, applications, trust, security, email	\N	2025-08-05 10:53:27.829979
29	1	email_template_tested	Тестирование шаблона "verification" для пользователя ebluffy (dima2_05@mail.ru)	\N	2025-08-05 10:53:34.056359
30	1	plugin_token_created	Создан токен для плагина: Minecraft Plugin Token (owyxPlugin)	\N	2025-08-05 12:45:09.061603
31	1	plugin_token_created	Создан токен для плагина: Minecraft Plugin Token (owyxPlugin)	\N	2025-08-05 13:43:50.548122
32	1	settings_updated	Обновлено настроек: 0. Категории: 	\N	2025-08-05 13:45:13.001442
33	1	settings_updated	Обновлено настроек: 36. Категории: general, applications, trust, security, email	\N	2025-08-05 13:45:17.761021
34	1	settings_updated	Обновлено настроек: 40. Категории: general, system, applications, trust, security, email	\N	2025-08-05 13:45:18.370926
35	1	settings_updated	Обновлено настроек: 0. Категории: 	\N	2025-08-05 13:46:51.821974
36	1	settings_updated	Обновлено настроек: 36. Категории: general, applications, trust, security, email	\N	2025-08-05 13:46:56.516849
37	1	settings_updated	Обновлено настроек: 40. Категории: general, system, applications, trust, security, email	\N	2025-08-05 13:46:57.360601
38	1	settings_updated	Обновлено настроек: 0. Категории: 	\N	2025-08-05 13:47:53.752927
39	1	email_template_tested	Тестирование шаблона "welcome" на адрес: grinih2010@mail.ru	\N	2025-08-05 14:21:14.739209
40	1	settings_updated	Обновлено настроек: 0. Категории: 	\N	2025-08-05 14:23:31.393272
41	1	email_template_saved	Сохранен email шаблон: welcome (Добро пожаловать)	\N	2025-08-05 14:23:32.247247
42	1	settings_updated	Обновлено настроек: 36. Категории: general, applications, trust, security, email	\N	2025-08-05 14:23:35.886883
43	1	email_template_saved	Сохранен email шаблон: welcome (Добро пожаловать)	\N	2025-08-05 14:23:36.537613
44	1	settings_updated	Обновлено настроек: 40. Категории: general, system, applications, trust, security, email	\N	2025-08-05 14:23:37.390338
45	1	plugin_token_created	Создан токен для плагина: Minecraft Plugin Token (owyxPlugin)	\N	2025-08-05 15:09:32.172836
46	1	user_unbanned	Пользователь ebluffy разблокирован	1	2025-08-06 10:42:18.195733
47	1	user_banned	Пользователь ebluffy заблокирован навсегда: test	1	2025-08-06 10:42:51.647646
48	1	user_unbanned	Пользователь ebluffy разблокирован	1	2025-08-06 11:01:29.545273
49	1	user_banned	Пользователь ebluffy заблокирован на 1 days: testing (до 07.08.2025, 14:17:00)	1	2025-08-06 11:17:00.304478
50	1	user_unbanned	Пользователь ebluffy разблокирован	1	2025-08-06 12:03:15.490345
53	1	role_changed	Роль пользователя Jaymand изменена с user на admin	21	2025-08-06 13:41:10.375988
54	1	trust_level_changed	Уровень доверия Jaymand изменен с 0 на 3: Так надо	21	2025-08-06 13:41:19.63355
55	1	email_template_tested	Тестирование шаблона "welcome" для пользователя ebluffy (dima2_05@mail.ru)	\N	2025-08-06 13:41:41.515337
56	1	settings_updated	Обновлено настроек: 0. Категории: 	\N	2025-08-06 13:44:06.589032
57	1	settings_updated	Обновлено настроек: 36. Категории: general, applications, trust, security, email	\N	2025-08-06 13:44:06.713205
58	1	settings_updated	Обновлено настроек: 40. Категории: general, system, applications, trust, security, email	\N	2025-08-06 13:44:06.7288
61	1	application_reviewed	Заявка shadow отклонена: tetstetstetse	20	2025-08-06 14:12:01.282246
4	1	application_reviewed	Заявка shadow одобрена [ПОЛЬЗОВАТЕЛЬ УДАЛЕН]	\N	2025-08-02 08:27:29.977641
63	21	application_reviewed	Заявка Jaymand одобрена	21	2025-08-06 14:41:02.357844
67	21	trust_level_changed	Уровень доверия Jaymand изменен с 3 на 2: 12312	21	2025-08-06 14:48:19.630841
68	21	trust_level_changed	Уровень доверия Jaymand изменен с 2 на 3: Поздравляю	21	2025-08-06 14:49:53.810194
70	1	user_banned	Пользователь Jaymand заблокирован на 1 days: питер (до 07.08.2025, 14:56:12)	21	2025-08-06 14:56:12.695926
71	1	user_unbanned	Пользователь Jaymand разблокирован	21	2025-08-06 14:57:05.240019
72	21	role_changed	Роль пользователя xD_zxc изменена с user на moderator	22	2025-08-06 14:59:53.257416
73	21	trust_level_changed	Уровень доверия xD_zxc изменен с 0 на 3: 1	22	2025-08-06 15:00:07.998085
74	23	profile_update	Обновлен профиль: имя, возраст, биография	23	2025-08-06 15:02:53.475758
75	23	profile_update	Обновлен профиль: имя, возраст, биография	23	2025-08-06 15:02:58.227686
76	23	profile_update	Обновлен профиль: имя, возраст, биография	23	2025-08-06 15:03:16.864906
79	22	profile_update	Обновлен профиль: имя, возраст, биография	22	2025-08-06 15:05:10.514013
80	22	profile_update	Обновлен профиль: имя, возраст, биография	22	2025-08-06 15:05:13.151187
81	22	profile_update	Обновлен профиль: имя, возраст, биография	22	2025-08-06 15:05:27.101669
82	22	application_reviewed	Заявка ikasaXD одобрена	23	2025-08-06 15:05:39.522052
84	21	application_reviewed	Заявка xD_zxc одобрена	22	2025-08-06 15:12:18.313856
86	21	email_template_saved	Сохранен email шаблон: verification (Подтверждение email)	\N	2025-08-06 15:19:01.184001
89	1	settings_updated	Обновлено настроек: 0. Категории: 	\N	2025-08-06 15:21:47.229544
90	1	settings_updated	Обновлено настроек: 36. Категории: general, applications, trust, security, email	\N	2025-08-06 15:21:47.306575
91	1	settings_updated	Обновлено настроек: 40. Категории: general, system, applications, trust, security, email	\N	2025-08-06 15:21:47.345697
92	1	settings_updated	Обновлено настроек: 0. Категории: 	\N	2025-08-06 15:29:33.323889
93	1	settings_updated	Обновлено настроек: 36. Категории: general, applications, trust, security, email	\N	2025-08-06 15:29:33.449592
94	1	settings_updated	Обновлено настроек: 40. Категории: general, system, applications, trust, security, email	\N	2025-08-06 15:29:33.4673
95	22	application_reviewed	Заявка Myatnyy1337 одобрена	29	2025-08-06 17:01:06.149152
96	21	trust_level_changed	Уровень доверия Jaymand изменен с 3 на 2: 12312321	21	2025-08-06 17:07:18.714776
97	21	trust_level_changed	Уровень доверия Jaymand изменен с 2 на 3: 1231232131	21	2025-08-06 17:07:55.001568
98	21	trust_level_changed	Уровень доверия Myatnyy1337 изменен с 0 на 3: Лучший бро	29	2025-08-06 17:08:08.399008
99	1	settings_updated	Обновлено настроек: 0. Категории: 	\N	2025-08-07 02:06:12.843699
100	1	settings_updated	Обновлено настроек: 36. Категории: general, applications, trust, security, email	\N	2025-08-07 02:06:12.994289
101	1	settings_updated	Обновлено настроек: 40. Категории: general, system, applications, trust, security, email	\N	2025-08-07 02:06:13.010812
102	1	settings_updated	Обновлено настроек: 0. Категории: 	\N	2025-08-07 02:06:35.760749
103	1	settings_updated	Обновлено настроек: 36. Категории: general, applications, trust, security, email	\N	2025-08-07 02:06:35.880049
104	1	settings_updated	Обновлено настроек: 40. Категории: general, system, applications, trust, security, email	\N	2025-08-07 02:06:35.898216
105	21	application_reviewed	Заявка Sasasaco одобрена	30	2025-08-08 15:23:22.164618
106	21	trust_level_changed	Уровень доверия Sasasaco изменен с 0 на 2: 1	30	2025-08-08 15:26:15.223856
107	21	application_reviewed	Заявка dritesxs564u одобрена	31	2025-08-08 15:56:44.244299
108	21	application_reviewed	Заявка Bobik одобрена	33	2025-08-09 18:03:34.048212
109	1	plugin_token_created	Создан токен для плагина: Minecraft Plugin Token (owyxPlugin)	\N	2025-08-19 20:30:05.062948
110	1	plugin_token_created	Создан токен для плагина: Minecraft Plugin Token (owyxPlugin)	\N	2025-08-19 20:39:36.903654
111	1	trust_level_changed	Уровень доверия ebluffy изменен с 3 на 3: Изменение через плагин	1	2025-08-19 20:44:55.417
112	1	user_banned	Пользователь Jaymand заблокирован на 1 days: дурашка (до 20.08.2025, 20:45:45)	21	2025-08-19 20:45:45.540363
113	1	user_unbanned	Пользователь Jaymand разблокирован	21	2025-08-19 20:47:14.718486
114	35	profile_update	Обновлен профиль: имя, возраст, биография	35	2025-08-19 20:55:11.358008
115	35	profile_update	Обновлен профиль: имя, возраст, биография	35	2025-08-19 20:55:55.735743
116	21	application_reviewed	Заявка grusha3108 одобрена	36	2025-08-19 21:05:16.382913
117	35	profile_update	Обновлен профиль: имя, возраст, биография	35	2025-08-19 21:08:46.850098
118	36	profile_update	Обновлен профиль: имя, возраст, биография	36	2025-08-19 21:12:15.945116
119	36	profile_update	Обновлен профиль: имя, возраст, биография	36	2025-08-19 21:15:29.796945
120	36	profile_update	Обновлен профиль: имя, возраст, биография	36	2025-08-19 21:15:30.846819
121	36	profile_update	Обновлен профиль: имя, возраст, биография	36	2025-08-19 21:15:36.759932
122	21	application_reviewed	Заявка NeInferno одобрена	37	2025-08-20 12:39:18.822416
123	1	email_template_tested	Тестирование шаблона "verification" для пользователя ikasaXD (ikasaanim@gmail.com)	\N	2025-08-20 16:12:45.054409
124	38	profile_update	Обновлен профиль: имя, возраст, биография	38	2025-08-20 22:41:42.869782
125	21	application_reviewed	Заявка ItzMirk_ одобрена	38	2025-08-21 09:41:01.734914
126	21	trust_level_changed	Уровень доверия ItzMirk_ изменен с 0 на 2: 1	38	2025-08-21 09:41:24.380742
127	21	trust_level_changed	Уровень доверия NeInferno изменен с 0 на 2: 1	37	2025-08-21 09:41:29.790001
128	21	trust_level_changed	Уровень доверия grusha3108 изменен с 0 на 2: 1	36	2025-08-21 09:41:33.655356
129	39	profile_update	Обновлен профиль: имя, возраст, биография	39	2025-08-21 15:33:35.815983
130	21	application_reviewed	Заявка korhjik_ одобрена	39	2025-08-21 15:38:17.446943
131	21	trust_level_changed	Уровень доверия korhjik_ изменен с 0 на 2: 1	39	2025-08-21 15:38:26.538251
132	22	application_reviewed	Заявка ArtemSliva одобрена	40	2025-08-26 13:23:03.501578
133	21	trust_level_changed	Уровень доверия ArtemSliva изменен с 0 на 2: 1	40	2025-08-27 16:17:54.28089
134	1	email_template_tested	Тестирование шаблона "password-reset" на адрес: miwafomenko2005@gmail.com	\N	2025-09-06 06:26:14.493473
136	1	user_banned	Пользователь Asatalop1234 заблокирован на 73050 days: даун (до 09.10.2225, 14:27:12)	41	2025-10-07 14:27:12.279536
137	1	user_banned	Пользователь PidorADM123 заблокирован на 73050 days: даун (до 09.10.2225, 14:28:09)	42	2025-10-07 14:28:09.917657
135	1	application_reviewed	Заявка Asatalop1234 одобрена [ПОЛЬЗОВАТЕЛЬ УДАЛЕН]	\N	2025-10-07 14:23:32.7994
140	1	forum_moderate_delete	Topic ID: 2	\N	2025-10-07 18:45:51.578198
141	1	forum_moderate_delete	Topic ID: 1	\N	2025-10-07 18:51:08.176203
142	1	forum_moderate_pin	Topic ID: 3	\N	2025-10-07 18:57:26.683676
143	1	forum_moderate_pin	Topic ID: 3	\N	2025-10-07 18:57:28.642679
144	1	forum_moderate_pin	Topic ID: 3	\N	2025-10-07 19:00:34.130143
145	1	forum_moderate_pin	Topic ID: 3	\N	2025-10-07 19:00:39.034551
146	1	forum_moderate_pin	Topic ID: 3	\N	2025-10-07 19:00:40.479739
147	1	forum_moderate_pin	Topic ID: 3	\N	2025-10-07 19:00:40.757065
148	1	forum_moderate_pin	Topic ID: 3	\N	2025-10-07 19:00:40.759743
149	1	forum_moderate_pin	Topic ID: 3	\N	2025-10-07 19:00:40.766978
150	1	forum_moderate_pin	Topic ID: 3	\N	2025-10-07 19:00:43.694063
151	1	forum_moderate_pin	Topic ID: 3	\N	2025-10-07 19:00:44.930261
152	1	forum_moderate_lock	Topic ID: 3	\N	2025-10-07 19:26:52.36393
153	1	forum_moderate_delete	Topic ID: 3	\N	2025-10-07 20:04:02.063067
154	1	settings_updated	Обновлено настроек: 0. Категории: 	\N	2025-10-09 19:48:21.12749
155	1	settings_updated	Обновлено настроек: 36. Категории: general, applications, trust, security, email	\N	2025-10-09 19:48:21.299407
156	1	settings_updated	Обновлено настроек: 40. Категории: general, system, applications, trust, security, email	\N	2025-10-09 19:48:21.32175
157	1	settings_updated	Обновлено настроек: 0. Категории: 	\N	2025-10-29 22:24:42.903099
158	1	settings_updated	Обновлено настроек: 36. Категории: general, applications, trust, security, email	\N	2025-10-29 22:24:43.007442
159	1	settings_updated	Обновлено настроек: 40. Категории: general, system, applications, trust, security, email	\N	2025-10-29 22:24:43.016458
\.


--
-- Data for Name: api_tokens; Type: TABLE DATA; Schema: public; Owner: owyx
--

COPY public.api_tokens (id, token_name, token_hash, token_prefix, user_id, permissions, is_active, last_used_at, expires_at, created_at, created_by, description) FROM stdin;
2	Minecraft Plugin Token (owyxPlugin)	a7d990041822fcfba67ba14a16b8ea8ad1d842147ea0c46ec2b9fe6b36f34e23	06e93ee9	1	["server:status", "players:read", "players:update", "sessions:manage", "stats:update"]	t	\N	\N	2025-08-05 13:43:50.423562	1	Бессрочный токен для Minecraft плагина
3	Minecraft Plugin Token (owyxPlugin)	a53ca95060781c5b7bcf044aaeb41c8d66306ffd40f90f38d88a3301f73a8fb8	b834fa18	1	["server:status", "players:read", "players:update", "sessions:manage", "stats:update"]	t	\N	\N	2025-08-05 15:09:32.048461	1	Бессрочный токен для Minecraft плагина
4	Minecraft Plugin Token (owyxPlugin)	10f58417f24f6b21f012b5d32323d1ff0560c86fc0e22c487a3f6f6814ff0183	00515d63	1	["server:status", "players:read", "players:update", "sessions:manage", "stats:update"]	t	\N	\N	2025-08-19 20:30:05.056616	1	Бессрочный токен для Minecraft плагина
5	Minecraft Plugin Token (owyxPlugin)	6af29ec8472cd518b8428d39870505bee53f381cdeae6b3fc1d13e2ab853e502	c11e3555	1	["server:status", "players:read", "players:update", "sessions:manage", "stats:update"]	t	\N	\N	2025-08-19 20:39:36.898675	1	Бессрочный токен для Minecraft плагина
1	Minecraft Plugin Token (owyxPlugin)	a8872aee17f64a035e754072fb8c20a997d956f772df483098a9d5d7aa903ef8	f5b92bb1	1	["server:status", "players:read", "players:update", "sessions:manage", "stats:update"]	t	2025-11-05 02:59:38.262723	\N	2025-08-05 12:45:08.95268	1	Бессрочный токен для Minecraft плагина
\.


--
-- Data for Name: applications; Type: TABLE DATA; Schema: public; Owner: root
--

COPY public.applications (id, user_id, minecraft_nick, age, discord, email, experience, motivation, plans, status, submitted_at, reviewed_at, reviewed_by, review_comment, ip_address, user_agent) FROM stdin;
1	1	ebluffy	20	Не указан	dima2_05@mail.ru	expert	ДАВДВАДВАДВАДВАДВАДВАДВАДВАВДАВДАВАДВАДВАДВАДВАДВАД	Планы: будут указаны в настройках профиля	approved	2025-08-02 06:56:57.458469	2025-08-02 06:57:12.011431	1		::1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36
10	33	Bobik	19	Не указан	gogihfgjhqshktdhn@gmail.com	expert	Bobik - это личность, которая известна своей уникальной индивидуальностью и ярким характером. Он/она отличается глубоким внутренним миром, богатым воображением и сильной внутренней мотивацией. В своих поступках и взглядах он/она часто проявляет независимость и стремление к самовыражению. Хайзебернг ценит искренность и честность, а также умеет вдохновлять окружающих своей энергией и оригинальностью. В целом, это человек с яркой индивидуальностью, который стремится к самореализации и не боится выделяться из толпы.	Планы будут указаны позже	approved	2025-08-09 18:01:21.428856	2025-08-09 18:03:34.03469	21		::ffff:127.0.0.1	Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Mobile Safari/537.36
12	37	NeInferno	15	Не указан	romeohuaweipro20@gmail.com	expert	По блату По блату По блату По блату По блату По блату	Планы будут указаны позже	approved	2025-08-20 12:38:46.48612	2025-08-20 12:39:18.811945	21		::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36 OPR/120.0.0.0 (Edition std-2)
14	39	korhjik_	12	Не указан	agaltuhovs@gmail.com	experienced	раньше играл в майнкрафт на пс 3 сейчас играю на компе (хз зачем вам эта инфа) хорошо строю ну и могу пройти майнкрафт с трудом конечно но могу	Планы будут указаны позже	approved	2025-08-21 15:23:37.947477	2025-08-21 15:38:17.435403	21		::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36
4	21	Jaymand	20	Не указан	gierciek@gmail.com	expert	123123123333333333333333333333333333333333333333333333333333	Планы будут указаны позже	approved	2025-08-06 14:11:22.725423	2025-08-06 14:41:02.350108	21		::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36 Edg/138.0.0.0
5	23	ikasaXD	15	Не указан	ikasaanim@gmail.com	expert	Потому что я хочу. В любом случае я пишу этот текст только потому что проходное количество символов 50 так что я думаю я выполнил свою задачу отлично	Планы будут указаны позже	approved	2025-08-06 15:04:27.600086	2025-08-06 15:05:39.509581	22		::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:141.0) Gecko/20100101 Firefox/141.0
6	22	xD_zxc	19	Не указан	mjhu1@yandex.ru	expert	привет я нашел сервер на просторе интернет я девочка мне 21 год я хочу играть на сервере с другими девочками я много курю и у меня голос как парня , но у меня третий размер груди и внешность как у альтушки примите пжжж мурр 	Планы будут указаны позже	approved	2025-08-06 15:11:48.601943	2025-08-06 15:12:18.301347	21		::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36
7	29	Myatnyy1337	14	Не указан	kumodiche@gmail.com	expert	Ты знаешь, где я вырос, чмо?\nЗнаешь, где я вырос? Где челики, делая кикфлип, а Kickflip - это трюк на скейтборде, катались на обычной площадочке, и тут врываются типы и начинают в них хуярить с пистолетов (пневматов). Просто эти скейтеры ебучие разбегаются и всё. Андерграунд: я жил, блядь, в однушке, где потолок был бетонный нахуй. Где пол был весь искорежен. Я играл на табуретке и сидел за столиком, кухонным. Так что пошел ты нахуй.	Планы будут указаны позже	approved	2025-08-06 17:00:36.809605	2025-08-06 17:01:06.134226	22		::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36
8	30	Sasasaco	100	Не указан	sasasaco64@gmail.com	expert	Почему я хочу играть на сервере Кива?  \n\n1. Комьюнити и атмосфера – Мне нравится, что ваш сервер активный и дружелюбный. Хочу стать частью комьюнити, общаться с игроками и участвовать в ивентах.  \n\n2. Геймплей и механики – У вас есть интересные режимы (например, приватки, мини-игры или экономика), которые делают игру увлекательной. Хочу развиваться, строить и помогать другим.  \n\n3. Честность и правила – Я ценю, что на сервере следят за порядком, нет читеров и гриферов. Готов соблюдать правила и вносить вклад в развитие сервера.  \n\n4. Уникальность – Кива выделяется среди других серверов своими фишками (можно уточнить, что именно тебе нравится: дона система, лут, PvP и т. д.).  \n\n5. Долгая игра – Я не просто хочу зайти на сервер на пару дней, а планирую остаться надолго, строить, зарабатывать и помогать новичкам.	Планы будут указаны позже	approved	2025-08-08 15:22:44.766136	2025-08-08 15:23:22.152721	21		::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36
9	31	dritesxs564u	11	Не указан	digamadhshajaajahahdwd@mail.ru	experienced	я люблю играть майнкрафт хочу играть на этом сервере потому что  задобали эти моды сервера	Планы будут указаны позже	approved	2025-08-08 15:42:21.487956	2025-08-08 15:56:44.232723	21		::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36
11	36	grusha3108	12	Не указан	grigoriyv3108@gmail.com	expert	Я хочу поиграть на сервере потому что, есть ебаный токен который нужно вписывать каждую неделю что бы подтвердить , и отличный орущий стример с партией риса. 	Планы будут указаны позже	approved	2025-08-19 21:02:37.67608	2025-08-19 21:05:16.376702	21		::ffff:127.0.0.1	Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Mobile Safari/537.36
13	38	ItzMirk_	21	Не указан	superreno12@gmail.com	intermediate	Выживать, и возможно стримить на твиче, а так просто повыживать и может поснимать на серваке видео 	Планы будут указаны позже	approved	2025-08-20 22:45:55.037573	2025-08-21 09:41:01.718099	21		::ffff:127.0.0.1	Mozilla/5.0 (Linux; Android 14; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.7258.94 Mobile Safari/537.36
15	40	ArtemSliva	17	Не указан	slivinskijartem295@gmail.com	expert	Люблю много ломать блоков, помогать строить, добывать ресурсы	Планы будут указаны позже	approved	2025-08-26 13:00:38.77156	2025-08-26 13:23:03.495611	22		::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36 Edg/139.0.0.0
\.


--
-- Data for Name: authplugin; Type: TABLE DATA; Schema: public; Owner: root
--

COPY public.authplugin (id, username, password_hash, uuid, ip_address, last_login, registered_at, login_type, email, email_verification_code, email_verification_expires, failed_attempts, last_failed_attempt) FROM stdin;
1	ebluffy	$2a$12$tQhdFibdk3H2adYnHQFkoOeOjYeP0Vo8.2dvcywuSB836L4nxmu4S	324583db-e99c-307c-9b9d-1da976cad0b7	127.0.0.1	\N	2025-11-02 00:09:26.588543	0	\N	\N	\N	0	\N
\.


--
-- Data for Name: daily_stats; Type: TABLE DATA; Schema: public; Owner: owyx
--

COPY public.daily_stats (id, user_id, stat_date, playtime_minutes, logins_count, blocks_broken, blocks_placed, distance_walked, deaths_count, mobs_killed, created_at, updated_at) FROM stdin;
1	1	2025-08-05	70	1	0	0	0	0	0	2025-08-05 16:10:45.155136	2025-08-05 16:44:27.672804
15	1	2025-08-06	75	1	0	0	0	0	0	2025-08-06 02:24:50.849079	2025-08-06 04:18:30.780881
\.


--
-- Data for Name: discord_oauth; Type: TABLE DATA; Schema: public; Owner: owyx
--

COPY public.discord_oauth (id, user_id, discord_id, discord_username, discord_discriminator, discord_avatar, access_token, refresh_token, expires_at, created_at, updated_at) FROM stdin;
\.


--
-- Data for Name: email_templates; Type: TABLE DATA; Schema: public; Owner: root
--

COPY public.email_templates (id, template_name, template_key, template_subject, template_html, template_text, is_active, created_at, updated_at, updated_by, template_variables) FROM stdin;
9	Заявка одобрена	application-approved	Ваша заявка одобрена!	<h1>Поздравляем!</h1><p>Ваша заявка на сервер {{serverName}} была одобрена.</p>	\N	t	2025-08-01 22:39:12.647806	2025-08-01 22:39:12.647806	\N	\N
10	Заявка отклонена	application-rejected	Заявка отклонена	<h1>Заявка отклонена</h1><p>К сожалению, ваша заявка была отклонена. Причина: {{rejectionReason}}</p>	\N	t	2025-08-01 22:39:12.647806	2025-08-01 22:39:12.647806	\N	\N
11	Сброс пароля	password-reset	Сброс пароля на {{serverName}}	<h1>Сброс пароля</h1><p>Для сброса пароля нажмите <a href="{{resetLink}}">здесь</a></p>	\N	t	2025-08-01 22:39:12.647806	2025-08-01 22:39:12.647806	\N	\N
12	Рассылка	newsletter	Новости {{serverName}}	<h1>{{newsletterTitle}}</h1><p>Новости и обновления сервера.</p>	\N	t	2025-08-01 22:39:12.647806	2025-08-01 22:39:12.647806	\N	\N
7	Добро пожаловать	welcome	Спасибо за регистрацию на {{serverName}}!	<h1 class="ql-align-center"><strong class="ql-size-large">Спасибо за регистрацию {{nickname}}!</strong></h1><h3><strong>Спасибо, что присоединились к нашему серверу {{serverName}}!</strong></h3><p><br></p><p>Мы рады видеть Вас в нашем дружном сообществе.</p><p>Впереди вас ждут интересные приключения, новые знакомства и множество возможностей для самореализации. Начните с ознакомления с нашими правилами, чтобы ваше пребывание было максимально комфортным и безопасным.</p><p>Если возникнут вопросы — всегда рады помочь в чатах Discord или Telegram.</p><h3><br></h3><h3>Желаем отличного настроения и ярких впечатлений на {{serverName}}!</h3><p><br></p><p class="ql-align-center">Так же просим вас присоединится к нас в соц. сетях:</p><p class="ql-align-center"><a href="{{discordInvite}}" target="_blank" style="color: rgb(88, 101, 242);"><strong>💬 Discord </strong></a><a href="{{telegramInvite}}" target="_blank" style="color: rgb(0, 136, 204);"><strong>📱 Telegram </strong></a></p><p class="ql-align-center"><br></p><p class="ql-align-center">С уважением, команда <strong>{{serverName}}</strong></p><p class="ql-align-center">Это письмо отправлено автоматически, пожалуйста, не отвечайте на него.</p>	\N	t	2025-08-01 22:39:12.647806	2025-08-05 14:23:36.406669	1	\N
8	Подтверждение email	verification	Подтвердите ваш email на {{serverName}}	<h1 class="ql-align-center"><strong class="ql-size-large">Подтвердите вашу почту!</strong></h1><h3><strong>Здравствуйте, {{nickname}}!</strong></h3><p><br></p><p class="ql-indent-1">Вы отправили запрос на подтверждение вашей почты на {{serverName}}! (Дата: {{currentDate}})</p><p class="ql-indent-1">Пожалуйста, подтвердите свой email,чтобы завершить регистрацию и получить полный доступ ко всем возможностям нашего сообщества.</p><p><br></p><p class="ql-indent-1">Для подтверждения нажмите <a href="{{verificationLink}}" target="_blank">здесь</a>.</p><p>( Если не переходит на страницу подтверждения, то скопируйте следующее и вставьте в адресную строку:</p><p>{{verificationLink}} )</p><p><br></p><blockquote>Если вы не отправляли запрос или вовсе не регистрировались на сайте - просто проигнориуйте это письмо.</blockquote><p><br></p><p class="ql-align-center">С уважением, команда <strong>{{serverName}}</strong></p><p class="ql-align-center">Это письмо отправлено автоматически, пожалуйста, не отвечайте на него.</p>	\N	t	2025-08-01 22:39:12.647806	2025-08-06 15:19:01.170033	21	\N
\.


--
-- Data for Name: email_verification_tokens; Type: TABLE DATA; Schema: public; Owner: owyx
--

COPY public.email_verification_tokens (id, user_id, token, expires_at, used_at, created_at, used) FROM stdin;
23	21	7ce76a77-9682-4e0f-bc2a-e449fafcd62f	2025-08-07 15:22:04.456	\N	2025-08-06 15:22:04.543996	t
25	29	8a491134-31ea-4752-9507-bb9e9a3f0c86	2025-08-07 17:03:55.275	\N	2025-08-06 17:03:55.280783	t
27	30	33eebea9-c282-4027-889b-3497dc337ad2	2025-08-09 15:20:40.124	\N	2025-08-08 15:20:40.131394	t
28	31	3b4aa6f6-e5d5-4fc3-b427-ce4a131bbca9	2025-08-09 15:37:21.155	\N	2025-08-08 15:37:21.155616	f
31	32	7dd1adfa-37bb-48cc-a1be-43bb47b0e023	2025-08-10 17:22:22.721	\N	2025-08-09 17:22:22.727683	f
33	34	8ef23de4-9f43-4b6c-8fb7-61d7fa8cc86a	2025-08-20 18:49:45.56	\N	2025-08-19 18:49:45.56072	f
36	35	09271254-e9cd-4443-8076-89c65815ea1a	2025-08-20 20:56:04.904	\N	2025-08-19 20:56:04.908334	t
38	36	34982cd7-44af-4b54-a019-4688af7614bc	2025-08-20 21:12:29.684	\N	2025-08-19 21:12:29.688269	t
40	37	5786e880-10a7-44a8-9597-369280d989ec	2025-08-21 12:40:38.058	\N	2025-08-20 12:40:38.063638	t
44	23	59bf1146-5107-41fe-8938-2f0c272d3f02	2025-08-21 16:05:17.367	\N	2025-08-20 16:05:17.3721	f
46	38	216901d7-e379-4ff0-94ad-b5db409300ef	2025-08-21 22:42:37.709	\N	2025-08-20 22:42:37.714891	t
52	39	cb6daac1-ce28-4896-bf9f-c1fc7d99c39b	2025-08-22 15:44:52.926	\N	2025-08-21 15:44:52.932881	f
53	33	8af17a69-726f-4786-9b20-cb33bac81a0a	2025-08-23 11:23:27.013	\N	2025-08-22 11:23:27.021146	t
55	40	35056854-af09-414a-ae22-70802d61f478	2025-08-27 12:58:47.089	\N	2025-08-26 12:58:47.094268	t
58	43	018d2031-d218-43f6-8158-cbc671893421	2025-10-29 15:26:22.558	\N	2025-10-28 15:26:22.559041	f
\.


--
-- Data for Name: forum_attachments; Type: TABLE DATA; Schema: public; Owner: root
--

COPY public.forum_attachments (id, post_id, topic_id, uploader_id, filename, original_filename, file_path, file_size, mime_type, created_at) FROM stdin;
\.


--
-- Data for Name: forum_categories; Type: TABLE DATA; Schema: public; Owner: root
--

COPY public.forum_categories (id, title, slug, description, icon, color, parent_id, "position", is_locked, created_at, updated_at, created_by, topics_count, posts_count) FROM stdin;
3	Помощь и поддержка	support	Вопросы по игре, техническая поддержка, решение проблем	❓	#2196F3	\N	3	f	2025-10-07 13:28:25.470028	2025-10-07 13:28:25.470028	\N	0	0
4	Предложения	suggestions	Ваши идеи по улучшению сервера	💡	#FF9800	\N	4	f	2025-10-07 13:28:25.470028	2025-10-07 13:28:25.470028	\N	0	0
5	Жалобы на игроков	reports	Жалобы на нарушителей правил сервера	⚠️	#F44336	\N	5	f	2025-10-07 13:28:25.470028	2025-10-07 13:28:25.470028	\N	0	0
6	Постройки и творчество	builds	Покажите свои постройки и творческие проекты	🏗️	#9C27B0	\N	6	f	2025-10-07 13:28:25.470028	2025-10-07 13:28:25.470028	\N	0	0
7	Гайды и туториалы	guides	Полезные гайды и обучающие материалы	📚	#00BCD4	\N	7	f	2025-10-07 13:28:25.470028	2025-10-07 13:28:25.470028	\N	0	0
8	Торговля	trading	Покупка, продажа и обмен ресурсами	💰	#CDDC39	\N	8	f	2025-10-07 13:28:25.470028	2025-10-07 13:28:25.470028	\N	0	0
9	Оффтоп	offtopic	Обсуждение тем, не связанных с игрой	🎮	#607D8B	\N	9	f	2025-10-07 13:28:25.470028	2025-10-07 13:28:25.470028	\N	0	0
2	Новости	news	Официальные новости, обновления и объявления администрации	📢	#4caf50	\N	1	f	2025-10-07 13:28:25.470028	2025-10-07 17:50:26.87226	\N	0	8
\.


--
-- Data for Name: forum_post_edits; Type: TABLE DATA; Schema: public; Owner: root
--

COPY public.forum_post_edits (id, post_id, edited_by, old_content, new_content, edit_reason, created_at) FROM stdin;
\.


--
-- Data for Name: forum_posts; Type: TABLE DATA; Schema: public; Owner: root
--

COPY public.forum_posts (id, topic_id, author_id, content, parent_post_id, created_at, updated_at, upvotes, downvotes, is_edited, is_deleted, deleted_by, deleted_at, deletion_reason) FROM stdin;
\.


--
-- Data for Name: forum_topics; Type: TABLE DATA; Schema: public; Owner: root
--

COPY public.forum_topics (id, category_id, title, slug, content, author_id, is_pinned, is_locked, is_solved, created_at, updated_at, last_post_at, views_count, posts_count, tags) FROM stdin;
\.


--
-- Data for Name: forum_votes; Type: TABLE DATA; Schema: public; Owner: root
--

COPY public.forum_votes (id, post_id, user_id, vote_type, created_at) FROM stdin;
\.


--
-- Data for Name: game_sessions; Type: TABLE DATA; Schema: public; Owner: owyx
--

COPY public.game_sessions (id, user_id, player_uuid, nickname, expires_at, ip_address, user_agent, is_active, created_at, last_login) FROM stdin;
f177c421-2845-429b-b9e1-8c7d687b8b11	39	71e0c803-d490-318c-98de-1c966d7345fb	korhjik_	2025-09-10 17:54:32.552	95.167.183.205	Minecraft Client	f	2025-09-03 17:54:32.553617	2025-09-03 17:54:32.553617
71564b09-5971-4ca9-91e7-efeec53ba396	39	71e0c803-d490-318c-98de-1c966d7345fb	korhjik_	2025-09-10 18:20:43.404	95.167.182.187	Minecraft Client	f	2025-09-03 18:20:43.404777	2025-09-03 18:20:43.404777
10da14c1-e5e0-42eb-b381-6e92fa8e130e	1	324583db-e99c-307c-9b9d-1da976cad0b7	ebluffy	2025-08-09 11:24:53.645	127.0.0.1	Minecraft Client	f	2025-08-02 08:24:53.15302	2025-08-02 18:53:10.728905
3bd57267-dcd6-40ee-9f7c-824ab1210132	1	324583db-e99c-307c-9b9d-1da976cad0b7	ebluffy	2025-08-09 21:58:37.926	127.0.0.1	Minecraft Client	f	2025-08-02 18:58:37.308049	2025-08-02 18:58:37.308049
97c5cf1c-ba35-4156-9083-a6193fc3f6bf	1	324583db-e99c-307c-9b9d-1da976cad0b7	ebluffy	2025-08-09 22:01:40.67	127.0.0.1	Minecraft Client	f	2025-08-02 19:01:40.052616	2025-08-02 20:57:09.658519
5db5e33e-6435-4635-9961-c82994b5e41f	1	324583db-e99c-307c-9b9d-1da976cad0b7	ebluffy	2025-08-12 18:27:53.432	127.0.0.1	Minecraft Client	f	2025-08-05 15:27:53.522139	2025-08-05 15:27:53.522139
133f5bc4-c566-4fdc-b5be-55db8b2dffe7	39	71e0c803-d490-318c-98de-1c966d7345fb	korhjik_	2025-09-12 11:30:02.725	95.167.183.216	Minecraft Client	t	2025-09-05 11:30:02.726106	2025-09-05 19:26:40.76576
1f4bc303-97ab-4664-9a01-41bb78f4d337	33	70d47449-7e3c-3770-ba0f-547f811d5bb9	Bobik	2025-08-29 11:15:32.756	188.170.87.166	Minecraft Client	t	2025-08-22 11:15:32.757299	2025-08-22 13:15:06.109055
2ffd31d8-bbcd-4ce4-a830-62ca34d72e53	39	71e0c803-d490-318c-98de-1c966d7345fb	korhjik_	2025-08-29 09:17:34.871	95.167.183.208	Minecraft Client	f	2025-08-22 09:17:34.872253	2025-08-22 17:49:35.044808
481a4eba-3201-47ac-a09d-2c6bec791738	39	71e0c803-d490-318c-98de-1c966d7345fb	korhjik_	2025-08-30 06:31:52.084	95.167.183.94	Minecraft Client	f	2025-08-23 06:31:52.084921	2025-08-23 06:31:52.084921
369f9baf-f9ff-4b18-bfe5-bc34100015d7	1	324583db-e99c-307c-9b9d-1da976cad0b7	ebluffy	2025-08-12 18:32:15.376	127.0.0.1	Minecraft Client	f	2025-08-05 15:32:15.465141	2025-08-06 12:03:22.837561
429b57eb-57a4-4da8-92d0-36d814a0221e	1	324583db-e99c-307c-9b9d-1da976cad0b7	ebluffy	2025-08-26 20:42:02.799	84.17.46.67	Minecraft Client	t	2025-08-19 20:42:02.800284	2025-08-19 20:42:02.800284
f012aa87-6e08-4b05-9b2c-8a951a7d512a	37	71300b7b-c6e3-33ac-958f-2f9b8bb1d883	NeInferno	2025-08-27 12:49:45.756	31.43.223.242	Minecraft Client	t	2025-08-20 12:49:45.75757	2025-08-20 12:49:45.75757
8b619bb7-f125-40cf-9985-89c0437ca301	36	ce41dd15-3da6-3247-8107-8c60b5841591	grusha3108	2025-08-27 13:07:26.742	5.77.6.107	Minecraft Client	t	2025-08-20 13:07:26.743226	2025-08-20 13:07:26.743226
0ba78085-19b3-477f-bc8c-2247836018fe	39	71e0c803-d490-318c-98de-1c966d7345fb	korhjik_	2025-08-28 15:45:42.255	95.167.183.208	Minecraft Client	f	2025-08-21 15:45:42.256078	2025-08-21 15:45:42.256078
9864aa75-6c2a-45bf-902f-ec3e4af6792e	38	ff2fe742-44da-3ee4-accd-5c98b3052364	ItzMirk_	2025-08-28 11:19:12.969	77.222.100.129	Minecraft Client	t	2025-08-21 11:19:12.969964	2025-08-22 09:59:45.46684
fd818b61-4b38-4804-93e9-bccf697b1eb2	39	71e0c803-d490-318c-98de-1c966d7345fb	korhjik_	2025-08-30 08:54:08.217	95.167.182.99	Minecraft Client	f	2025-08-23 08:54:08.21765	2025-08-23 14:23:10.292935
8d9e1893-f526-4464-a3e3-77951cff3691	39	71e0c803-d490-318c-98de-1c966d7345fb	korhjik_	2025-08-30 15:10:53.303	95.167.183.43	Minecraft Client	f	2025-08-23 15:10:53.303767	2025-08-23 15:10:53.303767
5afd4dec-e2e0-47ba-afef-90fe41827bd0	39	71e0c803-d490-318c-98de-1c966d7345fb	korhjik_	2025-08-30 19:30:08.06	95.167.182.128	Minecraft Client	f	2025-08-23 19:30:08.060972	2025-08-23 19:30:08.060972
ada34c89-d445-4d32-ab64-8776d3232b26	21	bb6aeea7-d811-3493-9b55-0a8d5b1b301c	Jaymand	2025-08-26 20:43:11.271	94.50.181.125	Minecraft Client	f	2025-08-19 20:43:11.271995	2025-08-19 21:02:44.842406
e6a5305a-fd67-49e5-9cb3-90ae3764c9ea	21	bb6aeea7-d811-3493-9b55-0a8d5b1b301c	Jaymand	2025-09-03 14:16:49.122	31.163.252.31	Minecraft Client	t	2025-08-27 14:16:49.123379	2025-08-27 14:16:49.123379
713c5b67-1c55-43af-811e-51dabe003690	39	71e0c803-d490-318c-98de-1c966d7345fb	korhjik_	2025-08-31 06:02:39.618	95.167.182.99	Minecraft Client	f	2025-08-24 06:02:39.61943	2025-08-24 06:29:36.201291
\.


--
-- Data for Name: game_tokens; Type: TABLE DATA; Schema: public; Owner: owyx
--

COPY public.game_tokens (id, user_id, token_hash, expires_at, ip_address, user_agent, is_used, used_at, created_at) FROM stdin;
45089669-b377-47ca-86d3-ca7318ddfe2f	1	be34528805c9c60a68ba2d6497e05ad698d30533493d723510d86dd3a0f3aac1	2025-08-02 10:53:00.126	::1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-02 07:38:30.326427	2025-08-02 07:37:59.746636
bd4c19a4-d832-4847-a0ca-54b3b12ae092	1	46debc5cf5194f50cd55865b20cab6d43689aecba2f6eb83a2b11b348c9316db	2025-08-02 11:39:23.182	::1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-02 08:24:51.92212	2025-08-02 08:24:22.791823
4b4be0b2-8c53-42cf-ac43-5eab08586956	1	8d18832570fb2ed65f45c2f76f55cc0cbdec39e401c642d50460a61a51587f3b	2025-08-02 22:13:27.975	::1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-02 18:58:37.232315	2025-08-02 18:58:27.376169
0c8dbcc4-f958-4498-bd51-03b1dd81ba79	1	4b6375d107d75fe36d3ae3ee8b761b7614df1d78821e0a97ae7f05d5575bfbb8	2025-08-02 22:16:33.872	::1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-02 19:01:39.974699	2025-08-02 19:01:33.272051
6de40712-f238-4aca-b2c6-d5e183c47457	1	08a4fca2e5d23cb3794d5034d9d4b4d9c3cf7282ac9d9fa22f066df157643aad	2025-08-03 06:44:24.253	::1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	f	\N	2025-08-03 03:29:23.761747
eead5d9d-ec3f-468a-8bbf-964ed45a09b7	1	2461c20c23183ab9e54dae3d52f1d153888cfb0b17f9aa32e9f6be07b36d6574	2025-08-05 18:10:59.767	::1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-05 15:06:04.465273	2025-08-05 14:55:57.343301
12c521cf-6393-4501-844b-feea4caac0cb	1	cab87a5be0b18e36f9260e080d3ade17a06e145ef4f8fbc6b976886a8c3cb795	2025-08-05 18:21:07.009	::1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-05 15:15:15.488768	2025-08-05 15:06:04.58358
c374c87a-51f4-4aa8-8761-36e53ed311c1	1	cb7cbd4c690ea0d6297bb86b81a4438f2aa8d555fa9bc703895542bbe341a615	2025-08-05 18:30:18.031	::1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-05 15:20:36.574972	2025-08-05 15:15:15.607387
45beded7-f938-4828-aa49-a7dcb3c13b42	1	eb696feff95a7db50c9ca5b864b108ef5949fe073b9e9c93c872bd3d1877c3a7	2025-08-05 18:35:39.118	::1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-05 15:27:52.355255	2025-08-05 15:20:36.687648
8ed864f0-9533-441a-87d7-3bca8027200c	1	e3b1c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855	2025-08-05 15:42:58.057592	127.0.0.1	test	t	2025-08-05 15:32:06.540209	2025-08-05 15:27:58.057592
0115cfbc-4e16-49b4-8df4-5d70a24998d0	1	ada22c9c18f7ee91503efbf87f39cf0d7b8036550aac9f42e103b2d93d6536a4	2025-08-05 18:47:06.451	::1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-05 15:32:14.841238	2025-08-05 15:32:06.682002
31a74124-c8aa-454b-ae57-c11d044b13d1	1	283e1c9413b9483eeb2f81c7848c256242e1a3114c138fda68584b4dac7a9e04	2025-08-06 14:33:52.208	::1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-06 11:33:53.849896	2025-08-06 11:18:52.387671
e762b412-380e-4356-88db-bf68da2de2f6	1	d322b7cd5a4ba54f78af34d99b1ae1d8467a3b4610e41683a7d56ba836ba3ca6	2025-08-06 14:48:53.786	::1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	f	\N	2025-08-06 11:33:53.962105
6b30c03e-b064-4d31-9fcf-dc67cb639df3	21	35e37a850d6d0459816f8f0bf63104b2285522b6fce6e8d05ccc25881823d424	2025-08-06 14:56:23.235	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36 Edg/138.0.0.0	f	\N	2025-08-06 14:41:23.237323
ef164224-ac34-4804-a206-5625157fea3b	23	e95390636289e1e28d5dd6e8a7e4031ecb408a8021adde34fe917ccd1dd96f4b	2025-08-06 15:24:31.449	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:141.0) Gecko/20100101 Firefox/141.0	f	\N	2025-08-06 15:09:31.451249
713a361b-3615-4842-86cb-380a3a3dea25	30	88f7e5ac94939d92ebc8198be93b5c44f5003c242ef0f74e9a3ff01db32f7ce7	2025-08-08 15:38:35.569	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	f	\N	2025-08-08 15:23:35.57235
15fccc57-b214-4fbb-b284-6d9cfb712819	1	14386bd2612d621330248676099d1cd7544a238bf8adc532574bcb483d10dde2	2025-08-19 20:56:54.64	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-19 20:42:02.73269	2025-08-19 20:41:54.642451
b16005b2-706c-4d4d-9564-322e88517cab	21	a11c224f0e7bfb3e54936ef787088cd46da257481f638346384202d99d41f62e	2025-08-19 20:58:01.478	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36 Edg/139.0.0.0	t	2025-08-19 20:43:11.202301	2025-08-19 20:43:01.482045
51205015-fe4b-45b7-a0ae-7b308fd763aa	37	914c557e6e63ca44d03b32acbcf933963179485b0a4b54c808f82fdb8ebc3663	2025-08-20 13:00:53.357	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36 OPR/120.0.0.0 (Edition std-2)	t	2025-08-20 12:49:38.021277	2025-08-20 12:45:53.360152
856f683d-f628-4017-b741-46c41acd016f	37	132d75e85848fb633902d5654e3ea97d1cb453e336f5f7522ca8787df3590006	2025-08-20 13:04:38.02	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36 OPR/120.0.0.0 (Edition std-2)	t	2025-08-20 12:49:45.672079	2025-08-20 12:49:38.025392
1e19e1ae-90e9-4eef-8505-066e37c1d2d8	36	2856b382906bbaa3c561c98409378c4f56800b3f6284d7c3db74e78e5aa6a212	2025-08-20 13:22:20.511	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36 OPR/120.0.0.0 (Edition Yx 03)	t	2025-08-20 13:07:26.667398	2025-08-20 13:07:20.514194
7f27045d-b1fd-499f-82d8-a8914de2a280	38	558887d44d623214cc77a667f9315c969a8a7f8ea4ab23fd35d90986e23ebef3	2025-08-21 11:32:36.06	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36	t	2025-08-21 11:19:12.842366	2025-08-21 11:17:36.063129
69b4ae6d-c4bb-43e5-8a13-fb44a69bc442	39	51465c05cd16c7ff0bf3d98c0412aad4b95602eb31076c2e75f5e596ba21fbf3	2025-08-21 16:00:22.152	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-08-21 15:45:42.183719	2025-08-21 15:45:22.154644
547e3f97-8b8f-4d65-923b-05ff23b2e60a	39	755eb30e78110545afbbe787d1f35a4ea1849874e2f3f2711961a50f73327cac	2025-08-22 09:30:59.606	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-08-22 09:17:28.225452	2025-08-22 09:15:59.609149
875b9d54-d55d-4e3c-9aec-1ab3e881b001	39	6885d459d08ef3f7cd0e2ba97a779fa728ea319c4f506a1970e066aed20e3cf4	2025-08-22 09:32:28.224	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-08-22 09:17:34.793681	2025-08-22 09:17:28.228786
cea1aff1-b730-4785-b332-13656a1f728b	33	6dd42e048c76a480e519ed7cd69d21e04e8b5f0433f93792ce3de1cf78c664a6	2025-08-22 11:29:55.499	::ffff:127.0.0.1	Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Mobile Safari/537.36	t	2025-08-22 11:15:32.68049	2025-08-22 11:14:55.500816
12c05359-188e-438f-a503-0714b52c0997	39	14032a06dd1d788df6bf91312266850ee37707714e36d21dddf762471203afd0	2025-08-23 06:46:39.063	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-08-23 06:31:52.046434	2025-08-23 06:31:39.0658
a662c0ec-3658-48c6-88c9-50d4627067c0	39	19e94db970ae5eea67846cd5cd4d433822d97523c04b677d69f85119228b0cd7	2025-08-23 09:08:55.976	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-08-23 08:54:08.138423	2025-08-23 08:53:55.978838
4e716c53-a4c3-4bfd-9c42-b1d55bc4bdbd	39	a6fa753156da08584c627f050680929bba6e376b263d95af45a7167592529b25	2025-08-23 15:25:38.347	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-08-23 15:10:53.224758	2025-08-23 15:10:38.349529
219fedb6-a956-4876-95ff-13603301df8f	39	b0a9bcf3fac82caf3ad17c9db0718dd8062ce9416af3d2280e381f03f2faba23	2025-08-23 19:44:59.198	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-08-23 19:30:07.981133	2025-08-23 19:29:59.200426
bac31e3a-88c3-4b38-9a84-07be725f8e35	39	853cab73b0b04cbd1387613909b4c8195df846770a86066d188c14e545d8c7f7	2025-08-24 06:17:19.825	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-08-24 06:02:39.540829	2025-08-24 06:02:19.828484
9849e0da-8634-4478-8b97-7cfe4e76f450	21	38dbd3c0f96c6bba627b78a8cccce048fcc8b1aad3530267771be792e25f910e	2025-08-27 14:31:33.941	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36 Edg/139.0.0.0	t	2025-08-27 14:16:49.021818	2025-08-27 14:16:33.943746
9d6f60a6-fa4f-492e-9ad0-e359e67ec9b4	40	20c1839ac8821a3bade9a92af9ff70cd121ef27656d5df3880543429ce3d3715	2025-08-29 18:51:28.11	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36 Edg/139.0.0.0	f	\N	2025-08-29 18:36:28.112572
ed95d0bb-abd5-4ec2-882b-73b8fadb678b	39	9fcc1aad5e2fcbf399269ca9c766beac355dea5d8eeeadf08dda9f8d8df23748	2025-09-03 18:09:18.207	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-09-03 17:54:32.476692	2025-09-03 17:54:18.209304
0c7bee9f-6d12-48f8-bd3e-0cbd179dd77c	39	33a15ed1f554a125c74fe7288c12ec275d8df0dd56480e6565f97fd2f750448e	2025-09-03 18:35:32.745	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-09-03 18:20:43.274334	2025-09-03 18:20:32.747675
d1a1c059-0a85-4232-af9a-225d22018623	39	6c722fa5c900b595bf2765caaa2fef47d1084a850b56575db6acee9a7bca5fee	2025-09-05 11:44:46.258	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-09-05 11:30:02.651601	2025-09-05 11:29:46.261268
d5605e2f-be03-4a87-b79f-1ad20c86a976	39	1c1f974bbdee7d0413910a219a40cda91792ed31f8a6e5a2ba5c751ad4124953	2025-09-05 15:57:00.15	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	f	\N	2025-09-05 15:42:00.152481
0ab9ee7f-e527-404e-830b-ab3a1cfee5c7	21	222a0beab84ba0b5c010bcec5c5eda133eb94cbb8dc2c1785c2ae8eacc0e9802	2025-09-20 18:26:10.732	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36 Edg/140.0.0.0	f	\N	2025-09-20 18:11:10.735507
a1554f8b-73a5-46c9-8b23-ba03581f132e	39	886c61c768552b62750afef6edc3d077855cded674eb6332b100da5c991682a6	2025-10-01 17:35:38.078	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	f	\N	2025-10-01 17:20:38.080968
0173aeb4-0446-4850-b865-f5dc413172ee	1	4b5856ade6906391bcf914b475d0c9a193d0cd374fa45df44c005da236bb028d	2025-10-07 14:39:02.408	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	f	\N	2025-10-07 14:24:02.411109
\.


--
-- Data for Name: login_logs; Type: TABLE DATA; Schema: public; Owner: root
--

COPY public.login_logs (id, user_id, ip_address, user_agent, success, login_time, failure_reason) FROM stdin;
1	1	::1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-01 22:46:11.396297	\N
2	1	::1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-01 22:46:33.285399	\N
3	1	::1	Test-Client/1.0	f	2025-08-01 22:47:51.680583	\N
4	1	::1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-01 22:47:57.986039	\N
5	1	::1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-01 22:48:37.693499	\N
6	1	::1	Test-Client/1.0	f	2025-08-01 22:51:21.056526	\N
7	1	::1	Test-Client/1.0	f	2025-08-01 22:51:51.447852	\N
8	1	::1	Test-Client/1.0	t	2025-08-01 22:53:19.983075	\N
9	1	::1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-01 22:53:58.467914	\N
10	1	::1	Test-Client/1.0	t	2025-08-01 23:00:28.998998	\N
11	1	::1	axios/1.11.0	t	2025-08-01 23:01:35.716866	\N
12	1	::1	axios/1.11.0	t	2025-08-01 23:03:15.070582	\N
13	1	::1	axios/1.11.0	t	2025-08-01 23:03:40.647845	\N
14	1	::1	axios/1.11.0	t	2025-08-01 23:04:52.511608	\N
15	\N	::1	axios/1.11.0	f	2025-08-01 23:19:39.664754	\N
16	1	::1	axios/1.11.0	t	2025-08-01 23:20:11.845193	\N
18	\N	::1	axios/1.11.0	f	2025-08-02 01:56:03.288426	\N
19	1	::1	axios/1.11.0	t	2025-08-02 01:56:28.598076	\N
20	1	::1	axios/1.11.0	t	2025-08-02 05:42:08.171218	\N
21	1	::1	axios/1.11.0	t	2025-08-03 03:38:03.252803	\N
22	\N	::1	axios/1.11.0	f	2025-08-03 03:58:12.286086	\N
23	1	::1	axios/1.11.0	t	2025-08-03 03:58:49.761531	\N
24	\N	::1	node	f	2025-08-03 07:45:21.760981	\N
25	1	::1	node	t	2025-08-03 07:46:08.049172	\N
26	1	::1	node	t	2025-08-03 07:46:36.613487	\N
27	1	::1	axios/1.11.0	t	2025-08-05 12:39:30.597897	\N
28	1	::1	axios/1.11.0	t	2025-08-05 12:45:08.604033	\N
29	1	::1	axios/1.11.0	t	2025-08-05 13:43:50.058939	\N
30	1	::1	axios/1.11.0	t	2025-08-05 13:47:53.255432	\N
31	1	::1	axios/1.11.0	t	2025-08-05 15:09:31.677732	\N
32	\N	::1	Mozilla/5.0 (Windows NT; Windows NT 10.0; ru-RU) WindowsPowerShell/5.1.26100.4768	f	2025-08-05 15:26:10.090757	\N
33	1	::1	Mozilla/5.0 (Windows NT; Windows NT 10.0; ru-RU) WindowsPowerShell/5.1.26100.4768	t	2025-08-05 15:26:54.826963	\N
34	1	::1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-06 11:14:54.489801	\N
35	1	::1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-06 11:17:12.926893	\N
36	1	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-06 13:25:17.011267	\N
37	21	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36 Edg/138.0.0.0	t	2025-08-06 13:32:31.68	\N
40	1	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-06 14:01:23.355232	\N
42	1	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-06 14:11:36.181317	\N
43	1	::ffff:127.0.0.1	curl/7.81.0	t	2025-08-06 14:40:57.486098	\N
44	1	::ffff:127.0.0.1	curl/7.81.0	t	2025-08-06 14:41:47.015317	\N
45	1	::ffff:127.0.0.1	curl/7.81.0	t	2025-08-06 14:52:21.883301	\N
46	21	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36 Edg/138.0.0.0	t	2025-08-06 14:57:03.561411	\N
47	23	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:141.0) Gecko/20100101 Firefox/141.0	t	2025-08-06 14:59:11.107069	\N
48	22	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36	t	2025-08-06 14:59:37.115817	\N
49	\N	::ffff:127.0.0.1	curl/7.81.0	t	2025-08-06 15:14:17.040456	\N
50	\N	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-06 15:15:09.413587	\N
52	1	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-06 15:17:47.87516	\N
54	1	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-06 15:20:31.95951	\N
55	\N	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36	t	2025-08-06 16:59:13.982765	\N
56	29	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36	t	2025-08-06 16:59:25.153687	\N
57	\N	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-08 15:13:50.021607	\N
58	30	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-08 15:13:57.899308	\N
59	\N	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-08 15:37:21.151979	\N
60	31	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-08 15:37:27.974929	\N
61	\N	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-09 17:17:33.923846	\N
62	32	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-09 17:17:40.381408	\N
63	\N	::ffff:127.0.0.1	Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Mobile Safari/537.36	t	2025-08-09 17:57:23.293217	\N
64	33	::ffff:127.0.0.1	Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Mobile Safari/537.36	t	2025-08-09 17:57:33.600203	\N
65	23	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:141.0) Gecko/20100101 Firefox/141.0	t	2025-08-13 11:40:15.319388	\N
66	23	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:141.0) Gecko/20100101 Firefox/141.0	t	2025-08-13 11:40:17.592382	\N
67	21	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36 Edg/139.0.0.0	t	2025-08-17 00:36:19.20872	\N
68	1	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-19 13:47:43.875854	\N
69	\N	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36 Edg/139.0.0.0	t	2025-08-19 18:49:45.554545	\N
70	34	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36 Edg/139.0.0.0	t	2025-08-19 18:49:51.829964	\N
71	1	::ffff:127.0.0.1	axios/1.11.0	t	2025-08-19 20:28:32.230591	\N
72	1	::ffff:127.0.0.1	axios/1.11.0	t	2025-08-19 20:30:04.991535	\N
73	1	::ffff:127.0.0.1	axios/1.11.0	t	2025-08-19 20:39:36.738082	\N
74	21	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36 Edg/139.0.0.0	t	2025-08-19 20:46:45.593159	\N
75	\N	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 YaBrowser/25.2.0.0 Safari/537.36	f	2025-08-19 20:50:47.734527	\N
76	\N	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 YaBrowser/25.2.0.0 Safari/537.36	t	2025-08-19 20:53:30.928115	\N
77	35	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 YaBrowser/25.2.0.0 Safari/537.36	t	2025-08-19 20:53:35.736042	\N
78	\N	::ffff:127.0.0.1	Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Mobile Safari/537.36	t	2025-08-19 20:55:05.29396	\N
79	36	::ffff:127.0.0.1	Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Mobile Safari/537.36	t	2025-08-19 20:55:25.483938	\N
80	\N	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36 Edg/139.0.0.0	t	2025-08-20 12:29:34.824527	\N
81	37	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36 Edg/139.0.0.0	t	2025-08-20 12:29:45.236396	\N
82	37	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36 OPR/120.0.0.0 (Edition std-2)	t	2025-08-20 12:32:03.391005	\N
83	36	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36 OPR/120.0.0.0 (Edition Yx 03)	t	2025-08-20 12:56:17.731756	\N
84	36	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36 OPR/120.0.0.0 (Edition Yx 03)	t	2025-08-20 12:57:04.152204	\N
85	30	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 YaBrowser/25.6.0.0 Safari/537.36	t	2025-08-20 12:58:31.809676	\N
86	23	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:142.0) Gecko/20100101 Firefox/142.0	t	2025-08-20 15:57:41.564937	\N
87	\N	::ffff:127.0.0.1	Mozilla/5.0 (Linux; Android 14; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.7258.94 Mobile Safari/537.36	t	2025-08-20 22:40:23.992871	\N
88	38	::ffff:127.0.0.1	Mozilla/5.0 (Linux; Android 14; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.7258.94 Mobile Safari/537.36	t	2025-08-20 22:40:48.411879	\N
89	38	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36	t	2025-08-21 11:16:41.077325	\N
90	\N	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-08-21 15:19:57.119794	\N
91	39	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-08-21 15:20:21.054709	\N
92	33	::ffff:127.0.0.1	Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Mobile Safari/537.36	t	2025-08-22 11:14:41.834121	\N
93	\N	::ffff:127.0.0.1	Mozilla/5.0 (iPhone; CPU iPhone OS 18_3_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/15E148 THDConsumer/7.45.0.1 (iPhone;iOS 18.3.1)DID:	f	2025-08-25 20:26:13.806962	\N
94	\N	::ffff:127.0.0.1	Mozilla/5.0 (iPhone; CPU iPhone OS 18_3_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/15E148 THDConsumer/7.45.0.1 (iPhone;iOS 18.3.1)DID:	f	2025-08-25 20:26:25.515727	\N
95	\N	::ffff:127.0.0.1	Mozilla/5.0 (iPhone; CPU iPhone OS 18_3_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/15E148 THDConsumer/7.45.0.1 (iPhone;iOS 18.3.1)DID:	f	2025-08-25 20:26:36.074017	\N
96	\N	::ffff:127.0.0.1	Mozilla/5.0 (iPhone; CPU iPhone OS 18_3_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/15E148 THDConsumer/7.45.0.1 (iPhone;iOS 18.3.1)DID:	f	2025-08-25 20:26:45.631217	\N
97	\N	::ffff:127.0.0.1	Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Mobile Safari/537.36	f	2025-08-25 21:09:52.412515	\N
98	\N	::ffff:127.0.0.1	Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Mobile Safari/537.36	f	2025-08-25 21:10:04.063871	\N
99	\N	::ffff:127.0.0.1	Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Mobile Safari/537.36	f	2025-08-25 21:10:14.856977	\N
100	\N	::ffff:127.0.0.1	Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Mobile Safari/537.36	f	2025-08-25 21:10:25.681844	\N
101	\N	::ffff:127.0.0.1	Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Mobile Safari/537.36	f	2025-08-26 06:29:12.155147	\N
102	\N	::ffff:127.0.0.1	Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Mobile Safari/537.36	f	2025-08-26 06:29:22.978185	\N
103	\N	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36 Edg/139.0.0.0	t	2025-08-26 12:58:03.839929	\N
104	40	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36 Edg/139.0.0.0	t	2025-08-26 12:58:10.890464	\N
105	40	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36 Edg/139.0.0.0	t	2025-08-26 12:58:40.553799	\N
106	22	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36	t	2025-08-26 13:22:24.637799	\N
107	1	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-08-28 22:41:54.051611	\N
108	40	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36 Edg/139.0.0.0	t	2025-08-29 18:36:13.742348	\N
109	21	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36 Edg/139.0.0.0	t	2025-09-01 18:45:47.373172	\N
110	39	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-09-03 17:54:08.576354	\N
111	1	::ffff:127.0.0.1	Mozilla/5.0 (Linux; arm_64; Android 15; 2311DRK48G) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.7204.107 YaBrowser/25.8.3.107.00 SA/3 Mobile Safari/537.36	t	2025-09-06 05:42:32.060367	\N
112	1	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-09-14 19:49:08.30225	\N
113	1	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-09-14 19:50:25.572283	\N
114	21	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36 Edg/140.0.0.0	t	2025-09-15 04:51:45.285608	\N
115	30	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-09-23 18:57:37.594826	\N
116	1	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-01 10:09:21.234423	\N
117	39	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-01 17:20:18.211743	\N
118	1	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-04 07:07:20.156092	\N
119	\N	::ffff:127.0.0.1	Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Mobile Safari/537.36	f	2025-10-06 03:27:41.447456	\N
120	\N	::ffff:127.0.0.1	Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Mobile Safari/537.36	f	2025-10-06 03:27:50.795964	\N
121	\N	::ffff:127.0.0.1	Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Mobile Safari/537.36	f	2025-10-06 03:28:00.074265	\N
122	\N	::ffff:127.0.0.1	Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Mobile Safari/537.36	f	2025-10-06 03:28:09.684047	\N
123	\N	::ffff:127.0.0.1	Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Mobile Safari/537.36	f	2025-10-06 03:41:41.543337	\N
124	\N	::ffff:127.0.0.1	Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Mobile Safari/537.36	f	2025-10-06 03:41:50.037605	\N
125	\N	::ffff:127.0.0.1	Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Mobile Safari/537.36	f	2025-10-06 03:58:23.742244	\N
126	1	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-07 12:35:43.678499	\N
127	1	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-07 13:02:32.658754	\N
128	\N	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36	t	2025-10-07 14:13:14.114627	\N
130	\N	::ffff:127.0.0.1	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36	t	2025-10-07 14:27:53.484687	\N
132	1	84.17.46.67	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-07 14:46:52.949493	\N
133	\N	84.17.46.67	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	f	2025-10-07 15:41:30.106513	\N
134	1	84.17.46.67	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-07 15:41:33.531609	\N
135	1	84.17.46.67	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-07 15:50:17.716188	\N
136	1	84.17.46.67	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-07 15:51:22.76463	\N
137	1	84.17.46.67	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-07 16:05:11.40608	\N
138	1	84.17.46.67	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-07 16:18:52.820533	\N
139	1	84.17.46.67	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-07 16:19:49.602568	\N
140	1	84.17.46.67	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-07 16:21:45.778381	\N
141	1	84.17.46.67	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-07 16:29:16.75427	\N
142	1	84.17.46.67	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-07 16:49:25.193027	\N
143	1	84.17.46.67	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-07 16:50:53.242965	\N
144	1	84.17.46.67	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-07 16:53:26.821613	\N
145	1	84.17.46.67	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-07 16:54:45.901344	\N
146	1	84.17.46.67	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-07 16:59:44.020441	\N
147	1	84.17.46.67	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-07 16:59:47.000134	\N
148	1	84.17.46.67	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-07 17:01:48.647526	\N
149	1	84.17.46.67	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-07 17:02:19.417836	\N
150	1	84.17.46.67	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-07 17:03:15.679018	\N
151	1	84.17.46.67	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-07 17:04:49.543295	\N
152	1	195.98.85.126	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-09 19:48:39.854789	\N
153	\N	176.116.165.202	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36 OPR/122.0.0.0 (Edition MSI)	t	2025-10-28 15:26:22.551031	\N
154	43	176.116.165.202	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36 OPR/122.0.0.0 (Edition MSI)	t	2025-10-28 15:26:45.484208	\N
155	43	176.116.165.202	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36 OPR/122.0.0.0 (Edition MSI)	t	2025-10-28 15:30:02.870168	\N
156	1	195.98.85.126	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-29 18:13:44.464141	\N
157	\N	195.98.85.126	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	f	2025-10-29 22:55:59.415509	\N
158	1	195.98.85.126	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-29 22:56:29.037497	\N
159	1	195.98.85.126	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-29 23:00:06.876663	\N
160	1	195.98.85.126	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-29 23:05:24.803872	\N
161	1	195.98.85.126	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-29 23:07:52.031769	\N
162	1	195.98.85.126	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-29 23:11:21.481232	\N
163	1	195.98.85.126	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-29 23:11:58.576296	\N
164	1	195.98.85.126	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-29 23:15:34.142643	\N
165	1	195.98.85.126	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 YaBrowser/25.8.0.0 Safari/537.36	t	2025-10-29 23:15:47.616045	\N
166	1	109.195.53.120	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 YaBrowser/25.10.0.0 Safari/537.36	t	2025-12-04 03:25:10.103076	\N
\.


--
-- Data for Name: password_reset_tokens; Type: TABLE DATA; Schema: public; Owner: owyx
--

COPY public.password_reset_tokens (id, user_id, token, expires_at, used_at, created_at) FROM stdin;
\.


--
-- Data for Name: player_stats; Type: TABLE DATA; Schema: public; Owner: root
--

COPY public.player_stats (id, user_id, is_time_limited, reputation, total_logins, current_level, time_played_minutes, achievements_count, created_at, updated_at, minecraft_stats, session_count, average_session_duration, longest_session_duration, last_seen, blocks_broken, blocks_placed, distance_walked, deaths_count, mobs_killed, items_crafted, damage_dealt, damage_taken, food_eaten, jumps_count, online_time_today, online_time_week, online_time_month, active_days_count, last_ip_address, stats_last_updated) FROM stdin;
12	33	t	0	19	0	5649	0	2025-08-09 17:57:23.286178	2025-08-22 13:27:51.483658	{}	0	0	0	2025-08-22 16:27:51.357874	0	0	0	30	6457	0	0	0	0	0	0	0	0	0	\N	2025-08-22 13:27:51.483658
16	37	t	0	0	0	48	0	2025-08-20 12:29:34.815276	2025-08-20 13:40:40.622631	{}	0	0	0	2025-08-20 16:40:40.563789	0	0	0	0	36	0	0	0	0	0	0	0	0	0	\N	2025-08-20 13:40:40.622631
4	23	f	0	0	0	0	0	2025-08-06 15:03:40.165397	2025-08-06 15:03:40.165397	{}	0	0	0	\N	0	0	0	0	0	0	0	0	0	0	0	0	0	0	\N	2025-08-06 15:03:40.165397
18	39	t	0	11	0	1170	0	2025-08-21 15:19:57.11481	2025-09-05 19:30:07.93353	{}	0	0	0	2025-09-05 22:30:03.662604	0	0	0	19	3341	0	0	0	0	0	0	0	0	0	\N	2025-09-05 19:30:07.93353
22	43	t	0	0	0	0	0	2025-10-28 15:26:22.532086	2025-10-28 15:26:22.532086	{}	0	0	0	\N	0	0	0	0	0	0	0	0	0	0	0	0	0	0	\N	2025-10-28 15:26:22.532086
7	22	f	0	0	0	0	0	2025-08-06 15:26:04.964986	2025-08-06 15:26:04.964986	{}	0	0	0	\N	0	0	0	0	0	0	0	0	0	0	0	0	0	0	\N	2025-08-06 15:26:04.964986
8	29	t	0	0	0	0	0	2025-08-06 16:59:13.976112	2025-08-06 16:59:13.976112	{}	0	0	0	\N	0	0	0	0	0	0	0	0	0	0	0	0	0	0	\N	2025-08-06 16:59:13.976112
9	30	t	0	0	0	0	0	2025-08-08 15:13:50.004323	2025-08-08 15:13:50.004323	{}	0	0	0	\N	0	0	0	0	0	0	0	0	0	0	0	0	0	0	\N	2025-08-08 15:13:50.004323
10	31	t	0	0	0	0	0	2025-08-08 15:37:21.147858	2025-08-08 15:37:21.147858	{}	0	0	0	\N	0	0	0	0	0	0	0	0	0	0	0	0	0	0	\N	2025-08-08 15:37:21.147858
11	32	t	0	0	0	0	0	2025-08-09 17:17:33.914139	2025-08-09 17:17:33.914139	{}	0	0	0	\N	0	0	0	0	0	0	0	0	0	0	0	0	0	0	\N	2025-08-09 17:17:33.914139
2	1	f	0	6	10	555	0	2025-08-01 22:59:26.196421	2025-11-05 02:59:28.251779	{"fish_caught": 0, "jumps_count": 3392, "mobs_killed": 22, "times_slept": 3, "animals_bred": 0, "damage_dealt": 530, "damage_taken": 423, "deaths_count": 8, "blocks_broken": 0, "blocks_placed": 0, "items_crafted": 0, "items_dropped": 0, "distance_flying": 8809, "distance_walked": 16562, "items_picked_up": 0, "distance_by_boat": 839, "distance_by_horse": 0, "distance_swimming": 98}	1	0	0	2025-11-05 05:59:27.769075	0	0	524492	16	14	0	5302	4236	0	3392	0	0	0	0	127.0.0.1	2025-11-05 02:59:28.251779
13	34	t	0	0	0	0	0	2025-08-19 18:49:45.546597	2025-08-19 18:49:45.546597	{}	0	0	0	\N	0	0	0	0	0	0	0	0	0	0	0	0	0	0	\N	2025-08-19 18:49:45.546597
17	38	t	0	1	0	741	0	2025-08-20 22:40:23.974383	2025-08-22 10:02:31.624898	{}	0	0	0	2025-08-22 13:02:31.569825	0	0	0	24	21	0	0	0	0	0	0	0	0	0	\N	2025-08-22 10:02:31.624898
15	36	t	0	0	0	2154	0	2025-08-19 20:55:05.287891	2025-08-20 13:20:50.217773	{}	0	0	0	2025-08-20 16:20:50.163029	0	0	0	58	515	0	0	0	0	0	0	0	0	0	\N	2025-08-20 13:20:50.217773
19	40	t	0	0	0	0	0	2025-08-26 12:58:03.831811	2025-08-26 12:58:03.831811	{}	0	0	0	\N	0	0	0	0	0	0	0	0	0	0	0	0	0	0	\N	2025-08-26 12:58:03.831811
14	35	t	0	0	0	0	0	2025-08-19 20:53:30.923552	2025-08-19 20:53:30.923552	{}	0	0	0	\N	0	0	0	0	0	0	0	0	0	0	0	0	0	0	\N	2025-08-19 20:53:30.923552
3	21	f	0	1	0	2962	0	2025-08-06 13:41:22.792296	2025-08-27 14:29:01.094441	{}	0	0	0	2025-08-27 17:29:00.932761	0	0	0	21	2962	0	0	0	0	0	0	0	0	0	\N	2025-08-27 14:29:01.094441
\.


--
-- Data for Name: reputation_log; Type: TABLE DATA; Schema: public; Owner: owyx
--

COPY public.reputation_log (id, from_user_id, to_user_id, reputation_change, reason, comment, created_at) FROM stdin;
\.


--
-- Data for Name: server_settings; Type: TABLE DATA; Schema: public; Owner: root
--

COPY public.server_settings (id, setting_key, setting_value, setting_type, category, description, updated_at, updated_by) FROM stdin;
1	applications-enabled	true	boolean	applications	Прием заявок	2025-10-29 22:24:42.932877	1
24	login-lockout-duration	15	integer	security	Время блокировки (мин)	2025-10-29 22:24:42.971575	1
4	min-motivation-length	50	integer	applications	Мин. символов в мотивации	2025-10-29 22:24:42.937511	1
20	server-name	owyxMine	string	general	Название сервера	2025-10-29 22:24:42.900105	1
23	jwt-expires-days	30	integer	security	Время жизни JWT (дни)	2025-10-29 22:24:42.975657	1
18	server-description	Minecraft сервер	string	general	Описание сервера	2025-10-29 22:24:42.906078	1
5	min-plans-length	30	integer	applications	Мин. символов в планах	2025-10-29 22:24:42.93962	1
19	server-ip	play.owyx.site	string	general	IP адрес сервера	2025-10-29 22:24:42.908512	1
27	require-email-verification	true	boolean	security	Требовать подтверждение email	2025-10-29 22:24:42.978792	1
3	max-applications-per-day	3	integer	applications	Лимит заявок в день	2025-10-29 22:24:42.941477	1
28	two-factor-enabled	false	boolean	security	2FA включен	2025-10-29 22:24:42.981345	1
21	server-port	25164	integer	general	Порт сервера	2025-10-29 22:24:42.911533	1
26	rate-limit-requests	100	integer	security	Rate limit (запросов/мин)	2025-10-29 22:24:42.983819	1
2	auto-approve-trust-level	1	integer	applications	Автоодобрение по Trust Level	2025-10-29 22:24:42.943383	1
8	smtp-host	smtp.yandex.ru	string	email	SMTP сервер	2025-10-29 22:24:42.986797	1
38	trust-points-email	10	integer	trust	Очки за подтверждение email	2025-10-29 22:24:42.945083	1
37	trust-points-discord	20	integer	trust	Очки за Discord	2025-10-29 22:24:42.947116	1
39	trust-points-hour	1	integer	trust	Очки за час игры	2025-10-29 22:24:42.949282	1
40	trust-points-reputation	2	integer	trust	Очки за единицу репутации	2025-10-29 22:24:42.951319	1
17	max-players	100	integer	general	Максимум игроков	2025-10-29 22:24:42.916734	1
9	smtp-password	hoiigazkhichljfz	string	email	SMTP пароль	2025-10-29 22:24:43.002298	1
32	trust-level-1-required	25	integer	trust	Очки для Trust Level 1	2025-10-29 22:24:42.960848	1
16	discord-invite	https://discord.gg/dyy4xYyUe5	string	general	Discord приглашение	2025-10-29 22:24:42.919504	1
22	telegram-invite	https://discord.gg/dyy4xYyUe5	string	general	Telegram канал	2025-10-29 22:24:42.9227	1
14	smtp-tls	true	boolean	email	Использовать TLS	2025-10-29 22:24:43.005468	1
12	smtp-sender-name	owyxMine	string	email	Имя отправителя	2025-10-29 22:24:43.008994	1
11	smtp-reply-to	owyx.helper@yandex.ru	string	email	Reply-To адрес	2025-10-29 22:24:43.010685	1
33	trust-level-2-required	100	integer	trust	Очки для Trust Level 2	2025-10-29 22:24:42.963826	1
30	maintenance-mode	true	boolean	system	Режим обслуживания	2025-10-29 22:24:42.925964	1
6	email-notifications-enabled	true	boolean	email	Email уведомления	2025-10-29 22:24:43.013397	1
13	smtp-timeout	30	integer	email	Тайм-аут SMTP (сек)	2025-10-29 22:24:43.015004	1
31	registration-enabled	true	boolean	system	Регистрация разрешена	2025-10-29 22:24:42.928063	1
34	trust-level-3-required	500	integer	trust	Очки для Trust Level 3	2025-10-29 22:24:42.966485	1
29	auto-backup-enabled	true	boolean	system	Автобэкапы	2025-10-29 22:24:42.930379	1
10	smtp-port	465	integer	email	SMTP порт	2025-10-29 22:24:42.989772	1
7	smtp-from	owyx.helper@yandex.ru	string	email	Email отправителя	2025-10-29 22:24:42.99292	1
15	smtp-user	owyx.helper@yandex.ru	string	email	SMTP пользователь	2025-10-29 22:24:42.997153	1
35	trust-minimum-hours	10	integer	trust	Минимум часов игры для повышения	2025-10-29 22:24:42.954627	1
36	trust-minimum-reputation	1	integer	trust	Минимум репутации для повышения	2025-10-29 22:24:42.957607	1
25	max-login-attempts	5	integer	security	Максимум попыток входа	2025-10-29 22:24:42.96922	1
\.


--
-- Data for Name: server_status; Type: TABLE DATA; Schema: public; Owner: owyx
--

COPY public.server_status (id, server_ip, server_port, tps, uptime_seconds, max_memory, used_memory, free_memory, online_players, max_players, server_version, plugins_count, loaded_worlds, updated_at, created_at) FROM stdin;
1	localhost	25565	19.80	3661	8192	4096	4096	1	20	1.21.1	15	3	2025-08-05 15:37:21.098109	2025-08-05 15:37:21.098109
2	localhost	25164	19.80	3661	8192	4096	4096	1	20	1.21.1	15	3	2025-08-05 15:38:16.321982	2025-08-05 15:38:16.321982
3	play.owyx.site	25164	20.00	161	4096	2010	578	1	100	1.21.4-222-9b1798d (MC: 1.21.4)	63	3	2025-11-05 02:59:38.266455	2025-11-01 21:06:41.848283
\.


--
-- Data for Name: trust_level_applications; Type: TABLE DATA; Schema: public; Owner: owyx
--

COPY public.trust_level_applications (id, user_id, current_level, requested_level, reason, status, reviewed_by, reviewed_at, review_comment, reputation_score, hours_played, email_verified, submitted_at) FROM stdin;
\.


--
-- Data for Name: user_achievements; Type: TABLE DATA; Schema: public; Owner: owyx
--

COPY public.user_achievements (id, user_id, achievement_type, achievement_data, unlocked_at, created_at) FROM stdin;
\.


--
-- Data for Name: user_activity; Type: TABLE DATA; Schema: public; Owner: root
--

COPY public.user_activity (id, user_id, activity_type, metadata, ip_address, user_agent, created_at, description) FROM stdin;
2	1	login	\N	::1	\N	2025-08-01 22:53:20.08606	Вход в систему
3	1	login	\N	::1	\N	2025-08-01 22:53:58.571722	Вход в систему
4	1	login	\N	::1	\N	2025-08-01 23:00:29.103177	Вход в систему
5	1	login	\N	::1	\N	2025-08-01 23:01:35.82177	Вход в систему
6	1	login	\N	::1	\N	2025-08-01 23:03:15.175107	Вход в систему
7	1	login	\N	::1	\N	2025-08-01 23:03:40.752609	Вход в систему
8	1	login	\N	::1	\N	2025-08-01 23:04:52.615353	Вход в систему
9	1	login	\N	::1	\N	2025-08-01 23:20:11.951244	Вход в систему
11	1	login	\N	::1	\N	2025-08-02 01:56:28.7008	Вход в систему
12	1	login	\N	::1	\N	2025-08-02 05:42:08.310761	Вход в систему
19	1	join_server	\N	\N	\N	2025-08-02 07:04:29.088918	Игрок зашел на сервер
20	1	leave_server	{"session_minutes": 0}	\N	\N	2025-08-02 07:05:07.022318	Игрок вышел с сервера
21	1	join_server	\N	\N	\N	2025-08-02 07:05:48.440278	Игрок зашел на сервер
22	1	leave_server	{"session_minutes": 0}	\N	\N	2025-08-02 07:39:34.917929	Игрок вышел с сервера
23	1	leave_server	{"session_minutes": 0}	\N	\N	2025-08-02 08:25:16.093282	Игрок вышел с сервера
24	1	leave_server	{"session_minutes": 0}	\N	\N	2025-08-02 08:26:35.357463	Игрок вышел с сервера
25	1	leave_server	{"session_minutes": 0}	\N	\N	2025-08-02 18:24:40.57089	Игрок вышел с сервера
26	1	playtime_update	{"total_minutes": 70, "session_minutes": 3}	\N	\N	2025-08-02 18:57:56.217104	Обновлено время игры
27	1	leave_server	{"session_minutes": 0}	\N	\N	2025-08-02 18:57:56.341452	Игрок вышел с сервера
28	1	leave_server	{"session_minutes": 0}	\N	\N	2025-08-02 19:01:02.771658	Игрок вышел с сервера
29	1	leave_server	{"session_minutes": 0}	\N	\N	2025-08-02 19:28:41.335486	Игрок вышел с сервера
30	1	leave_server	{"session_minutes": 0}	\N	\N	2025-08-02 19:29:00.49041	Игрок вышел с сервера
31	1	avatar_update	\N	\N	\N	2025-08-03 03:26:56.748038	Обновлен аватар профиля
32	1	login	\N	::1	\N	2025-08-03 03:38:03.358192	Вход в систему
33	1	login	\N	::1	\N	2025-08-03 03:58:49.867327	Вход в систему
34	1	login	\N	::1	\N	2025-08-03 07:46:08.155606	Вход в систему
35	1	login	\N	::1	\N	2025-08-03 07:46:36.719106	Вход в систему
36	1	avatar_update	\N	\N	\N	2025-08-03 07:47:12.665645	Обновлен аватар профиля
37	1	avatar_update	\N	\N	\N	2025-08-03 20:20:39.943909	Обновлен аватар профиля
38	1	avatar_update	\N	\N	\N	2025-08-03 20:21:05.176566	Обновлен аватар профиля
39	1	avatar_update	\N	\N	\N	2025-08-03 20:22:01.641655	Обновлен аватар профиля
40	1	avatar_update	\N	\N	\N	2025-08-03 20:29:48.647704	Обновлен аватар профиля
41	1	login	\N	::1	\N	2025-08-05 12:39:30.730463	Вход в систему
42	1	login	\N	::1	\N	2025-08-05 12:45:08.728817	Вход в систему
43	1	login	\N	::1	\N	2025-08-05 13:43:50.177019	Вход в систему
44	1	login	\N	::1	\N	2025-08-05 13:47:53.37977	Вход в систему
45	1	login	\N	::1	\N	2025-08-05 15:09:31.796576	Вход в систему
46	1	login	\N	::1	\N	2025-08-05 15:26:54.96198	Вход в систему
47	1	playtime_update	{"total_minutes": 676, "session_minutes": 8}	\N	\N	2025-08-05 16:18:45.383377	Обновлено время игры
48	1	leave_server	{"session_minutes": 0}	\N	\N	2025-08-05 16:20:29.043421	Игрок вышел с сервера
49	1	playtime_update	{"total_minutes": 688, "session_minutes": 9}	\N	\N	2025-08-05 16:42:24.177493	Обновлено время игры
50	1	leave_server	{"session_minutes": 0}	\N	\N	2025-08-05 16:44:27.784021	Игрок вышел с сервера
51	1	playtime_update	{"total_minutes": 696, "session_minutes": 5}	\N	\N	2025-08-06 02:29:51.480253	Обновлено время игры
52	1	playtime_update	{"total_minutes": 701, "session_minutes": 5}	\N	\N	2025-08-06 02:34:50.960094	Обновлено время игры
53	1	playtime_update	{"total_minutes": 706, "session_minutes": 5}	\N	\N	2025-08-06 02:39:50.951597	Обновлено время игры
54	1	playtime_update	{"total_minutes": 711, "session_minutes": 5}	\N	\N	2025-08-06 02:44:50.951947	Обновлено время игры
55	1	playtime_update	{"total_minutes": 720, "session_minutes": 9}	\N	\N	2025-08-06 02:54:51.02457	Обновлено время игры
56	1	playtime_update	{"total_minutes": 725, "session_minutes": 5}	\N	\N	2025-08-06 02:59:50.939329	Обновлено время игры
57	1	playtime_update	{"total_minutes": 730, "session_minutes": 5}	\N	\N	2025-08-06 03:04:50.941036	Обновлено время игры
58	1	playtime_update	{"total_minutes": 735, "session_minutes": 5}	\N	\N	2025-08-06 03:09:50.940873	Обновлено время игры
59	1	playtime_update	{"total_minutes": 740, "session_minutes": 5}	\N	\N	2025-08-06 03:14:51.009105	Обновлено время игры
60	1	playtime_update	{"total_minutes": 745, "session_minutes": 5}	\N	\N	2025-08-06 03:19:50.94455	Обновлено время игры
61	1	playtime_update	{"total_minutes": 754, "session_minutes": 9}	\N	\N	2025-08-06 03:29:50.999256	Обновлено время игры
62	1	playtime_update	{"total_minutes": 759, "session_minutes": 5}	\N	\N	2025-08-06 03:34:50.996849	Обновлено время игры
63	1	playtime_update	{"total_minutes": 764, "session_minutes": 5}	\N	\N	2025-08-06 03:39:51.001531	Обновлено время игры
64	1	leave_server	{"session_minutes": 0}	\N	\N	2025-08-06 03:41:37.850819	Игрок вышел с сервера
65	1	playtime_update	{"total_minutes": 775, "session_minutes": 7}	\N	\N	2025-08-06 03:49:51.440609	Обновлено время игры
66	1	playtime_update	{"total_minutes": 780, "session_minutes": 5}	\N	\N	2025-08-06 03:54:50.946243	Обновлено время игры
67	1	playtime_update	{"total_minutes": 785, "session_minutes": 5}	\N	\N	2025-08-06 03:59:50.955064	Обновлено время игры
68	1	playtime_update	{"total_minutes": 790, "session_minutes": 5}	\N	\N	2025-08-06 04:04:50.965524	Обновлено время игры
69	1	leave_server	{"session_minutes": 0}	\N	\N	2025-08-06 04:10:19.277732	Игрок вышел с сервера
70	1	playtime_update	{"total_minutes": 799, "session_minutes": 9}	\N	\N	2025-08-06 04:19:30.876017	Обновлено время игры
71	1	playtime_update	{"total_minutes": 804, "session_minutes": 5}	\N	\N	2025-08-06 04:24:30.879108	Обновлено время игры
72	1	playtime_update	{"total_minutes": 809, "session_minutes": 5}	\N	\N	2025-08-06 04:29:30.921918	Обновлено время игры
73	1	playtime_update	{"total_minutes": 819, "session_minutes": 8}	\N	\N	2025-08-06 09:16:01.961186	Обновлено время игры
74	1	playtime_update	{"total_minutes": 824, "session_minutes": 5}	\N	\N	2025-08-06 09:21:01.986693	Обновлено время игры
75	1	playtime_update	{"total_minutes": 829, "session_minutes": 5}	\N	\N	2025-08-06 09:26:01.979079	Обновлено время игры
76	1	playtime_update	{"total_minutes": 834, "session_minutes": 5}	\N	\N	2025-08-06 09:31:05.225225	Обновлено время игры
77	1	leave_server	{"session_minutes": 0}	\N	\N	2025-08-06 09:31:24.068525	Игрок вышел с сервера
78	1	playtime_update	{"total_minutes": 842, "session_minutes": 8}	\N	\N	2025-08-06 09:41:01.966492	Обновлено время игры
79	1	playtime_update	{"total_minutes": 847, "session_minutes": 5}	\N	\N	2025-08-06 09:46:01.954078	Обновлено время игры
80	1	playtime_update	{"total_minutes": 852, "session_minutes": 5}	\N	\N	2025-08-06 09:51:02.011571	Обновлено время игры
81	1	leave_server	{"session_minutes": 0}	\N	\N	2025-08-06 09:55:44.082857	Игрок вышел с сервера
82	1	reputation_modified	{"reason": "test", "target_id": 1, "target_nickname": "ebluffy", "reputation_change": 10}	\N	\N	2025-08-06 09:56:04.203833	Изменил репутацию игрока ebluffy на 10
83	1	reputation_modified	{"reason": "test", "target_id": 1, "target_nickname": "ebluffy", "reputation_change": 10}	\N	\N	2025-08-06 09:56:59.101471	Изменил репутацию игрока ebluffy на 10
84	1	reputation_modified	{"reason": "test", "target_id": 1, "target_nickname": "ebluffy", "reputation_change": 10}	\N	\N	2025-08-06 09:57:49.845318	Изменил репутацию игрока ebluffy на 10
85	1	playtime_update	{"total_minutes": 863, "session_minutes": 5}	\N	\N	2025-08-06 10:01:02.032868	Обновлено время игры
86	1	leave_server	{"session_minutes": 0}	\N	\N	2025-08-06 10:03:07.319147	Игрок вышел с сервера
87	1	playtime_update	{"total_minutes": 873, "session_minutes": 7}	\N	\N	2025-08-06 10:11:01.959557	Обновлено время игры
88	1	playtime_update	{"total_minutes": 878, "session_minutes": 5}	\N	\N	2025-08-06 10:16:01.952669	Обновлено время игры
89	1	playtime_update	{"total_minutes": 887, "session_minutes": 9}	\N	\N	2025-08-06 10:26:01.957236	Обновлено время игры
90	1	playtime_update	{"total_minutes": 892, "session_minutes": 5}	\N	\N	2025-08-06 10:31:02.338591	Обновлено время игры
91	1	playtime_update	{"total_minutes": 897, "session_minutes": 5}	\N	\N	2025-08-06 10:36:01.947742	Обновлено время игры
92	1	playtime_update	{"total_minutes": 902, "session_minutes": 5}	\N	\N	2025-08-06 10:41:01.963374	Обновлено время игры
93	1	leave_server	{"session_minutes": 0}	\N	\N	2025-08-06 10:43:01.649118	Игрок вышел с сервера
94	1	login	\N	::1	\N	2025-08-06 11:14:54.591492	Вход в систему
95	1	login	\N	::1	\N	2025-08-06 11:17:13.031264	Вход в систему
96	1	login	\N	::ffff:127.0.0.1	\N	2025-08-06 13:25:17.021063	Вход в систему
97	1	avatar_update	\N	\N	\N	2025-08-06 13:25:39.330289	Обновлен аватар профиля
98	1	avatar_update	\N	\N	\N	2025-08-06 13:25:41.278997	Обновлен аватар профиля
99	21	login	\N	::ffff:127.0.0.1	\N	2025-08-06 13:32:31.683222	Вход в систему
100	21	avatar_update	\N	\N	\N	2025-08-06 13:33:54.493248	Обновлен аватар профиля
101	21	avatar_update	\N	\N	\N	2025-08-06 13:40:57.410822	Обновлен аватар профиля
102	21	avatar_update	\N	\N	\N	2025-08-06 13:41:07.615549	Обновлен аватар профиля
104	1	login	\N	::ffff:127.0.0.1	\N	2025-08-06 14:01:23.360291	Вход в систему
106	1	login	\N	::ffff:127.0.0.1	\N	2025-08-06 14:11:36.184505	Вход в систему
107	1	login	\N	::ffff:127.0.0.1	\N	2025-08-06 14:40:57.489679	Вход в систему
108	1	login	\N	::ffff:127.0.0.1	\N	2025-08-06 14:41:47.019143	Вход в систему
109	1	login	\N	::ffff:127.0.0.1	\N	2025-08-06 14:52:21.887353	Вход в систему
110	21	login	\N	::ffff:127.0.0.1	\N	2025-08-06 14:57:03.564457	Вход в систему
111	23	login	\N	::ffff:127.0.0.1	\N	2025-08-06 14:59:11.112085	Вход в систему
112	22	login	\N	::ffff:127.0.0.1	\N	2025-08-06 14:59:37.118859	Вход в систему
113	23	avatar_update	\N	\N	\N	2025-08-06 15:00:01.334627	Обновлен аватар профиля
114	22	avatar_update	\N	\N	\N	2025-08-06 15:02:24.600158	Обновлен аватар профиля
116	1	login	\N	::ffff:127.0.0.1	\N	2025-08-06 15:17:47.878864	Вход в систему
118	1	login	\N	::ffff:127.0.0.1	\N	2025-08-06 15:20:31.962343	Вход в систему
119	29	login	\N	::ffff:127.0.0.1	\N	2025-08-06 16:59:25.157696	Вход в систему
120	29	avatar_delete	\N	\N	\N	2025-08-06 17:02:44.560383	Удален аватар профиля
121	29	avatar_update	\N	\N	\N	2025-08-06 17:03:06.285152	Обновлен аватар профиля
122	29	avatar_delete	\N	\N	\N	2025-08-06 17:03:09.554459	Удален аватар профиля
123	29	avatar_update	\N	\N	\N	2025-08-06 17:03:34.673473	Обновлен аватар профиля
124	30	login	\N	::ffff:127.0.0.1	\N	2025-08-08 15:13:57.9044	Вход в систему
125	30	avatar_update	\N	\N	\N	2025-08-08 15:20:29.470574	Обновлен аватар профиля
126	30	avatar_update	\N	\N	\N	2025-08-08 15:20:29.506459	Обновлен аватар профиля
127	31	login	\N	::ffff:127.0.0.1	\N	2025-08-08 15:37:27.977257	Вход в систему
128	32	login	\N	::ffff:127.0.0.1	\N	2025-08-09 17:17:40.38341	Вход в систему
129	33	login	\N	::ffff:127.0.0.1	\N	2025-08-09 17:57:33.603111	Вход в систему
130	23	login	\N	::ffff:127.0.0.1	\N	2025-08-13 11:40:15.32231	Вход в систему
131	23	login	\N	::ffff:127.0.0.1	\N	2025-08-13 11:40:17.594512	Вход в систему
132	21	login	\N	::ffff:127.0.0.1	\N	2025-08-17 00:36:19.214291	Вход в систему
133	1	login	\N	::ffff:127.0.0.1	\N	2025-08-19 13:47:43.880306	Вход в систему
134	34	login	\N	::ffff:127.0.0.1	\N	2025-08-19 18:49:51.834258	Вход в систему
135	1	login	\N	::ffff:127.0.0.1	\N	2025-08-19 20:28:32.233956	Вход в систему
136	1	login	\N	::ffff:127.0.0.1	\N	2025-08-19 20:30:04.994549	Вход в систему
137	1	login	\N	::ffff:127.0.0.1	\N	2025-08-19 20:39:36.741385	Вход в систему
138	21	login	\N	::ffff:127.0.0.1	\N	2025-08-19 20:46:45.59587	Вход в систему
139	35	login	\N	::ffff:127.0.0.1	\N	2025-08-19 20:53:35.739293	Вход в систему
140	35	avatar_update	\N	\N	\N	2025-08-19 20:54:31.293274	Обновлен аватар профиля
141	35	avatar_delete	\N	\N	\N	2025-08-19 20:54:38.948811	Удален аватар профиля
142	35	avatar_update	\N	\N	\N	2025-08-19 20:54:55.439026	Обновлен аватар профиля
143	35	avatar_delete	\N	\N	\N	2025-08-19 20:54:58.166853	Удален аватар профиля
144	35	avatar_update	\N	\N	\N	2025-08-19 20:55:05.221667	Обновлен аватар профиля
145	36	login	\N	::ffff:127.0.0.1	\N	2025-08-19 20:55:25.486603	Вход в систему
146	21	leave_server	{"session_minutes": 0}	\N	\N	2025-08-19 21:10:16.552574	Игрок вышел с сервера
147	21	playtime_update	{"total_minutes": 2945, "session_minutes": 3}	\N	\N	2025-08-19 21:10:16.553735	Обновлено время игры
148	37	login	\N	::ffff:127.0.0.1	\N	2025-08-20 12:29:45.24001	Вход в систему
149	37	login	\N	::ffff:127.0.0.1	\N	2025-08-20 12:32:03.394633	Вход в систему
150	36	login	\N	::ffff:127.0.0.1	\N	2025-08-20 12:56:17.736046	Вход в систему
151	36	login	\N	::ffff:127.0.0.1	\N	2025-08-20 12:57:04.155597	Вход в систему
152	30	login	\N	::ffff:127.0.0.1	\N	2025-08-20 12:58:31.812911	Вход в систему
153	36	leave_server	{"session_minutes": 0}	\N	\N	2025-08-20 13:20:50.215445	Игрок вышел с сервера
154	37	leave_server	{"session_minutes": 0}	\N	\N	2025-08-20 13:40:40.688208	Игрок вышел с сервера
155	23	login	\N	::ffff:127.0.0.1	\N	2025-08-20 15:57:41.568551	Вход в систему
156	38	login	\N	::ffff:127.0.0.1	\N	2025-08-20 22:40:48.414054	Вход в систему
157	38	login	\N	::ffff:127.0.0.1	\N	2025-08-21 11:16:41.080683	Вход в систему
158	38	leave_server	{"session_minutes": 0}	\N	\N	2025-08-21 11:26:39.688372	Игрок вышел с сервера
159	39	login	\N	::ffff:127.0.0.1	\N	2025-08-21 15:20:21.057492	Вход в систему
160	39	leave_server	{"session_minutes": 0}	\N	\N	2025-08-21 16:50:47.582151	Игрок вышел с сервера
161	38	leave_server	{"session_minutes": 0}	\N	\N	2025-08-22 10:02:31.676361	Игрок вышел с сервера
162	33	login	\N	::ffff:127.0.0.1	\N	2025-08-22 11:14:41.838682	Вход в систему
163	33	leave_server	{"session_minutes": 0}	\N	\N	2025-08-22 11:20:29.932813	Игрок вышел с сервера
164	33	time_limit_exceeded	{"limit_minutes": 600, "total_minutes": 5542}	\N	\N	2025-08-22 11:20:29.95583	Игрок исключен за превышение лимита времени
165	33	leave_server	{"session_minutes": 0}	\N	\N	2025-08-22 11:22:08.930712	Игрок вышел с сервера
166	33	playtime_update	{"total_minutes": 5558, "session_minutes": 6}	\N	\N	2025-08-22 11:30:29.893961	Обновлено время игры
167	33	leave_server	{"session_minutes": 0}	\N	\N	2025-08-22 11:33:32.575448	Игрок вышел с сервера
168	33	leave_server	{"session_minutes": 0}	\N	\N	2025-08-22 11:47:20.162293	Игрок вышел с сервера
169	33	playtime_update	{"total_minutes": 5563, "session_minutes": 1}	\N	\N	2025-08-22 11:47:20.172709	Обновлено время игры
170	33	playtime_update	{"total_minutes": 5570, "session_minutes": 5}	\N	\N	2025-08-22 11:55:29.906433	Обновлено время игры
171	33	playtime_update	{"total_minutes": 5575, "session_minutes": 5}	\N	\N	2025-08-22 12:00:29.885011	Обновлено время игры
172	33	leave_server	{"session_minutes": 0}	\N	\N	2025-08-22 12:02:04.856513	Игрок вышел с сервера
173	33	leave_server	{"session_minutes": 0}	\N	\N	2025-08-22 12:03:09.224012	Игрок вышел с сервера
174	33	leave_server	{"session_minutes": 0}	\N	\N	2025-08-22 12:04:27.270069	Игрок вышел с сервера
175	33	playtime_update	{"total_minutes": 5584, "session_minutes": 5}	\N	\N	2025-08-22 12:15:29.930678	Обновлено время игры
176	33	leave_server	{"session_minutes": 0}	\N	\N	2025-08-22 12:16:40.207223	Игрок вышел с сервера
177	33	leave_server	{"session_minutes": 0}	\N	\N	2025-08-22 12:19:12.587697	Игрок вышел с сервера
178	33	leave_server	{"session_minutes": 0}	\N	\N	2025-08-22 12:21:44.95431	Игрок вышел с сервера
179	33	leave_server	{"session_minutes": 0}	\N	\N	2025-08-22 12:23:24.814862	Игрок вышел с сервера
180	33	playtime_update	{"total_minutes": 5598, "session_minutes": 7}	\N	\N	2025-08-22 12:30:29.931427	Обновлено время игры
181	33	playtime_update	{"total_minutes": 5603, "session_minutes": 5}	\N	\N	2025-08-22 12:35:30.048286	Обновлено время игры
182	33	leave_server	{"session_minutes": 0}	\N	\N	2025-08-22 12:38:49.400258	Игрок вышел с сервера
183	33	playtime_update	{"total_minutes": 5612, "session_minutes": 5}	\N	\N	2025-08-22 12:45:30.036318	Обновлено время игры
184	33	playtime_update	{"total_minutes": 5617, "session_minutes": 5}	\N	\N	2025-08-22 12:50:29.981795	Обновлено время игры
185	33	leave_server	{"session_minutes": 0}	\N	\N	2025-08-22 12:54:37.812251	Игрок вышел с сервера
186	33	leave_server	{"session_minutes": 0}	\N	\N	2025-08-22 12:55:39.227176	Игрок вышел с сервера
187	33	leave_server	{"session_minutes": 0}	\N	\N	2025-08-22 12:57:33.884777	Игрок вышел с сервера
188	33	leave_server	{"session_minutes": 0}	\N	\N	2025-08-22 12:58:36.014868	Игрок вышел с сервера
189	33	playtime_update	{"total_minutes": 5629, "session_minutes": 5}	\N	\N	2025-08-22 13:05:29.97775	Обновлено время игры
190	33	leave_server	{"session_minutes": 0}	\N	\N	2025-08-22 13:10:09.706619	Игрок вышел с сервера
191	33	leave_server	{"session_minutes": 0}	\N	\N	2025-08-22 13:11:45.949835	Игрок вышел с сервера
192	33	leave_server	{"session_minutes": 0}	\N	\N	2025-08-22 13:14:27.087572	Игрок вышел с сервера
302	1	logout	\N	84.17.46.67	\N	2025-10-07 15:41:26.895951	Выход из системы
193	33	playtime_update	{"total_minutes": 5641, "session_minutes": 5}	\N	\N	2025-08-22 13:20:29.99558	Обновлено время игры
194	33	playtime_update	{"total_minutes": 5646, "session_minutes": 5}	\N	\N	2025-08-22 13:25:30.066951	Обновлено время игры
195	33	leave_server	{"session_minutes": 0}	\N	\N	2025-08-22 13:27:51.478638	Игрок вышел с сервера
196	39	leave_server	{"session_minutes": 0}	\N	\N	2025-08-22 13:28:04.20875	Игрок вышел с сервера
197	39	playtime_update	{"total_minutes": 319, "session_minutes": 5}	\N	\N	2025-08-22 17:55:29.960423	Обновлено время игры
198	39	playtime_update	{"total_minutes": 324, "session_minutes": 5}	\N	\N	2025-08-22 18:00:29.965204	Обновлено время игры
199	39	playtime_update	{"total_minutes": 329, "session_minutes": 5}	\N	\N	2025-08-22 18:05:30.051514	Обновлено время игры
200	39	playtime_update	{"total_minutes": 334, "session_minutes": 5}	\N	\N	2025-08-22 18:10:29.957414	Обновлено время игры
201	39	playtime_update	{"total_minutes": 339, "session_minutes": 5}	\N	\N	2025-08-22 18:15:29.959464	Обновлено время игры
202	39	playtime_update	{"total_minutes": 348, "session_minutes": 9}	\N	\N	2025-08-22 18:25:29.999303	Обновлено время игры
203	39	playtime_update	{"total_minutes": 353, "session_minutes": 5}	\N	\N	2025-08-22 18:30:29.957104	Обновлено время игры
204	39	playtime_update	{"total_minutes": 358, "session_minutes": 5}	\N	\N	2025-08-22 18:35:30.059182	Обновлено время игры
205	39	playtime_update	{"total_minutes": 363, "session_minutes": 5}	\N	\N	2025-08-22 18:40:29.898382	Обновлено время игры
206	39	playtime_update	{"total_minutes": 368, "session_minutes": 5}	\N	\N	2025-08-22 18:45:30.044588	Обновлено время игры
207	39	playtime_update	{"total_minutes": 373, "session_minutes": 5}	\N	\N	2025-08-22 18:50:29.886883	Обновлено время игры
208	39	playtime_update	{"total_minutes": 378, "session_minutes": 5}	\N	\N	2025-08-22 18:55:30.021209	Обновлено время игры
209	39	playtime_update	{"total_minutes": 383, "session_minutes": 5}	\N	\N	2025-08-22 19:00:29.953373	Обновлено время игры
210	39	playtime_update	{"total_minutes": 388, "session_minutes": 5}	\N	\N	2025-08-22 19:05:30.003984	Обновлено время игры
211	39	playtime_update	{"total_minutes": 393, "session_minutes": 5}	\N	\N	2025-08-22 19:10:29.965345	Обновлено время игры
212	39	playtime_update	{"total_minutes": 398, "session_minutes": 5}	\N	\N	2025-08-22 19:15:30.038098	Обновлено время игры
213	39	leave_server	{"session_minutes": 0}	\N	\N	2025-08-22 19:16:15.729278	Игрок вышел с сервера
214	39	leave_server	{"session_minutes": 0}	\N	\N	2025-08-23 08:53:06.074392	Игрок вышел с сервера
215	39	leave_server	{"session_minutes": 0}	\N	\N	2025-08-23 08:59:39.420609	Игрок вышел с сервера
216	39	playtime_update	{"total_minutes": 556, "session_minutes": 9}	\N	\N	2025-08-23 09:10:29.605856	Обновлено время игры
217	39	playtime_update	{"total_minutes": 561, "session_minutes": 5}	\N	\N	2025-08-23 09:15:29.612702	Обновлено время игры
218	39	playtime_update	{"total_minutes": 566, "session_minutes": 5}	\N	\N	2025-08-23 09:20:29.509727	Обновлено время игры
219	39	playtime_update	{"total_minutes": 571, "session_minutes": 5}	\N	\N	2025-08-23 09:25:29.505204	Обновлено время игры
220	39	leave_server	{"session_minutes": 0}	\N	\N	2025-08-23 09:29:37.411574	Игрок вышел с сервера
221	39	playtime_update	{"total_minutes": 585, "session_minutes": 6}	\N	\N	2025-08-23 09:40:29.565668	Обновлено время игры
222	39	playtime_update	{"total_minutes": 590, "session_minutes": 5}	\N	\N	2025-08-23 09:45:29.42326	Обновлено время игры
223	39	playtime_update	{"total_minutes": 595, "session_minutes": 5}	\N	\N	2025-08-23 09:50:29.62088	Обновлено время игры
224	39	playtime_update	{"total_minutes": 600, "session_minutes": 5}	\N	\N	2025-08-23 09:55:29.484511	Обновлено время игры
225	39	playtime_update	{"total_minutes": 605, "session_minutes": 5}	\N	\N	2025-08-23 10:00:29.557598	Обновлено время игры
226	39	playtime_update	{"total_minutes": 610, "session_minutes": 5}	\N	\N	2025-08-23 10:05:29.46873	Обновлено время игры
227	39	playtime_update	{"total_minutes": 615, "session_minutes": 5}	\N	\N	2025-08-23 10:10:29.550167	Обновлено время игры
228	39	playtime_update	{"total_minutes": 620, "session_minutes": 5}	\N	\N	2025-08-23 10:15:29.588763	Обновлено время игры
229	39	playtime_update	{"total_minutes": 625, "session_minutes": 5}	\N	\N	2025-08-23 10:20:29.515602	Обновлено время игры
230	39	playtime_update	{"total_minutes": 630, "session_minutes": 5}	\N	\N	2025-08-23 10:25:29.532733	Обновлено время игры
231	39	playtime_update	{"total_minutes": 635, "session_minutes": 5}	\N	\N	2025-08-23 10:30:29.509684	Обновлено время игры
232	39	playtime_update	{"total_minutes": 640, "session_minutes": 5}	\N	\N	2025-08-23 10:35:29.529525	Обновлено время игры
233	39	playtime_update	{"total_minutes": 645, "session_minutes": 5}	\N	\N	2025-08-23 10:40:29.457484	Обновлено время игры
234	39	playtime_update	{"total_minutes": 650, "session_minutes": 5}	\N	\N	2025-08-23 10:45:29.478942	Обновлено время игры
235	39	leave_server	{"session_minutes": 0}	\N	\N	2025-08-23 10:47:15.117916	Игрок вышел с сервера
236	39	playtime_update	{"total_minutes": 661, "session_minutes": 8}	\N	\N	2025-08-23 11:15:29.498908	Обновлено время игры
237	39	playtime_update	{"total_minutes": 666, "session_minutes": 5}	\N	\N	2025-08-23 11:20:29.541061	Обновлено время игры
238	39	playtime_update	{"total_minutes": 671, "session_minutes": 5}	\N	\N	2025-08-23 11:25:29.659747	Обновлено время игры
239	39	playtime_update	{"total_minutes": 676, "session_minutes": 5}	\N	\N	2025-08-23 11:30:29.465653	Обновлено время игры
240	39	playtime_update	{"total_minutes": 681, "session_minutes": 5}	\N	\N	2025-08-23 11:35:29.584353	Обновлено время игры
241	39	playtime_update	{"total_minutes": 686, "session_minutes": 5}	\N	\N	2025-08-23 11:40:29.604601	Обновлено время игры
242	39	playtime_update	{"total_minutes": 691, "session_minutes": 5}	\N	\N	2025-08-23 11:45:29.452108	Обновлено время игры
243	39	playtime_update	{"total_minutes": 696, "session_minutes": 5}	\N	\N	2025-08-23 11:50:29.483941	Обновлено время игры
244	39	playtime_update	{"total_minutes": 701, "session_minutes": 5}	\N	\N	2025-08-23 11:55:29.64323	Обновлено время игры
245	39	leave_server	{"session_minutes": 0}	\N	\N	2025-08-23 11:57:56.437513	Игрок вышел с сервера
246	39	playtime_update	{"total_minutes": 711, "session_minutes": 7}	\N	\N	2025-08-23 14:30:29.601149	Обновлено время игры
247	39	playtime_update	{"total_minutes": 716, "session_minutes": 5}	\N	\N	2025-08-23 14:35:29.473527	Обновлено время игры
248	39	playtime_update	{"total_minutes": 721, "session_minutes": 5}	\N	\N	2025-08-23 14:40:29.493949	Обновлено время игры
249	39	playtime_update	{"total_minutes": 726, "session_minutes": 5}	\N	\N	2025-08-23 14:45:29.478593	Обновлено время игры
250	39	playtime_update	{"total_minutes": 731, "session_minutes": 5}	\N	\N	2025-08-23 14:50:29.510934	Обновлено время игры
251	39	playtime_update	{"total_minutes": 736, "session_minutes": 5}	\N	\N	2025-08-23 14:55:29.471462	Обновлено время игры
252	39	playtime_update	{"total_minutes": 741, "session_minutes": 5}	\N	\N	2025-08-23 15:00:29.573703	Обновлено время игры
253	39	playtime_update	{"total_minutes": 746, "session_minutes": 5}	\N	\N	2025-08-23 15:05:29.615945	Обновлено время игры
254	39	leave_server	{"session_minutes": 0}	\N	\N	2025-08-23 15:07:22.254892	Игрок вышел с сервера
255	39	leave_server	{"session_minutes": 0}	\N	\N	2025-08-23 15:14:57.295086	Игрок вышел с сервера
256	39	leave_server	{"session_minutes": 0}	\N	\N	2025-08-23 19:31:07.070341	Игрок вышел с сервера
257	39	leave_server	{"session_minutes": 0}	\N	\N	2025-08-24 06:29:05.524682	Игрок вышел с сервера
258	39	playtime_update	{"total_minutes": 784, "session_minutes": 5}	\N	\N	2025-08-24 06:35:33.712022	Обновлено время игры
259	39	playtime_update	{"total_minutes": 789, "session_minutes": 5}	\N	\N	2025-08-24 06:40:33.540238	Обновлено время игры
260	39	playtime_update	{"total_minutes": 794, "session_minutes": 5}	\N	\N	2025-08-24 06:45:33.567547	Обновлено время игры
261	39	leave_server	{"session_minutes": 0}	\N	\N	2025-08-24 06:46:16.597366	Игрок вышел с сервера
262	40	login	\N	::ffff:127.0.0.1	\N	2025-08-26 12:58:10.892308	Вход в систему
263	40	login	\N	::ffff:127.0.0.1	\N	2025-08-26 12:58:40.557044	Вход в систему
264	22	login	\N	::ffff:127.0.0.1	\N	2025-08-26 13:22:24.643213	Вход в систему
265	21	leave_server	{"session_minutes": 0}	\N	\N	2025-08-27 14:29:01.093024	Игрок вышел с сервера
266	1	login	\N	::ffff:127.0.0.1	\N	2025-08-28 22:41:54.054911	Вход в систему
267	40	login	\N	::ffff:127.0.0.1	\N	2025-08-29 18:36:13.746773	Вход в систему
268	21	login	\N	::ffff:127.0.0.1	\N	2025-09-01 18:45:47.376034	Вход в систему
269	39	login	\N	::ffff:127.0.0.1	\N	2025-09-03 17:54:08.579416	Вход в систему
270	39	leave_server	{"session_minutes": 0}	\N	\N	2025-09-03 18:17:17.228539	Игрок вышел с сервера
271	39	leave_server	{"session_minutes": 0}	\N	\N	2025-09-03 20:17:26.400958	Игрок вышел с сервера
272	39	leave_server	{"session_minutes": 0}	\N	\N	2025-09-05 14:22:27.888181	Игрок вышел с сервера
273	39	playtime_update	{"total_minutes": 1114, "session_minutes": 3}	\N	\N	2025-09-05 15:48:34.859722	Обновлено время игры
274	39	leave_server	{"session_minutes": 0}	\N	\N	2025-09-05 15:48:34.863035	Игрок вышел с сервера
275	39	leave_server	{"session_minutes": 0}	\N	\N	2025-09-05 18:36:55.453792	Игрок вышел с сервера
276	39	playtime_update	{"total_minutes": 1122, "session_minutes": 4}	\N	\N	2025-09-05 18:36:55.455785	Обновлено время игры
277	39	leave_server	{"session_minutes": 0}	\N	\N	2025-09-05 18:44:36.664159	Игрок вышел с сервера
278	39	playtime_update	{"total_minutes": 1135, "session_minutes": 9}	\N	\N	2025-09-05 18:55:31.524785	Обновлено время игры
279	39	playtime_update	{"total_minutes": 1140, "session_minutes": 5}	\N	\N	2025-09-05 19:00:31.653318	Обновлено время игры
280	39	playtime_update	{"total_minutes": 1145, "session_minutes": 5}	\N	\N	2025-09-05 19:05:31.67909	Обновлено время игры
281	39	playtime_update	{"total_minutes": 1150, "session_minutes": 5}	\N	\N	2025-09-05 19:10:31.525263	Обновлено время игры
282	39	playtime_update	{"total_minutes": 1155, "session_minutes": 5}	\N	\N	2025-09-05 19:15:31.58903	Обновлено время игры
283	39	playtime_update	{"total_minutes": 1160, "session_minutes": 5}	\N	\N	2025-09-05 19:20:31.614632	Обновлено время игры
284	39	playtime_update	{"total_minutes": 1165, "session_minutes": 5}	\N	\N	2025-09-05 19:25:31.562091	Обновлено время игры
285	39	leave_server	{"session_minutes": 0}	\N	\N	2025-09-05 19:26:38.050329	Игрок вышел с сервера
286	39	leave_server	{"session_minutes": 0}	\N	\N	2025-09-05 19:30:07.856566	Игрок вышел с сервера
287	1	login	\N	::ffff:127.0.0.1	\N	2025-09-06 05:42:32.064332	Вход в систему
288	1	login	\N	::ffff:127.0.0.1	\N	2025-09-14 19:49:08.306529	Вход в систему
289	1	login	\N	::ffff:127.0.0.1	\N	2025-09-14 19:50:25.577728	Вход в систему
290	21	login	\N	::ffff:127.0.0.1	\N	2025-09-15 04:51:45.287808	Вход в систему
291	30	login	\N	::ffff:127.0.0.1	\N	2025-09-23 18:57:37.597575	Вход в систему
292	1	login	\N	::ffff:127.0.0.1	\N	2025-10-01 10:09:21.238184	Вход в систему
293	39	login	\N	::ffff:127.0.0.1	\N	2025-10-01 17:20:18.217428	Вход в систему
294	1	login	\N	::ffff:127.0.0.1	\N	2025-10-04 07:07:20.160048	Вход в систему
295	1	logout	\N	::ffff:127.0.0.1	\N	2025-10-07 12:12:24.949839	Выход из системы
296	1	logout	\N	::ffff:127.0.0.1	\N	2025-10-07 12:12:24.952425	Выход из системы
297	1	login	\N	::ffff:127.0.0.1	\N	2025-10-07 12:35:43.682986	Вход в систему
298	1	login	\N	::ffff:127.0.0.1	\N	2025-10-07 13:02:32.662547	Вход в систему
301	1	login	\N	84.17.46.67	\N	2025-10-07 14:46:52.953512	Вход в систему
303	1	login	\N	84.17.46.67	\N	2025-10-07 15:41:33.534375	Вход в систему
304	1	login	\N	84.17.46.67	\N	2025-10-07 15:50:17.721629	Вход в систему
305	1	login	\N	84.17.46.67	\N	2025-10-07 15:51:22.768439	Вход в систему
306	1	login	\N	84.17.46.67	\N	2025-10-07 16:05:11.410911	Вход в систему
307	1	login	\N	84.17.46.67	\N	2025-10-07 16:18:52.824275	Вход в систему
308	1	login	\N	84.17.46.67	\N	2025-10-07 16:19:49.605711	Вход в систему
309	1	login	\N	84.17.46.67	\N	2025-10-07 16:21:45.782499	Вход в систему
310	1	login	\N	84.17.46.67	\N	2025-10-07 16:29:16.758418	Вход в систему
311	1	login	\N	84.17.46.67	\N	2025-10-07 16:49:25.197949	Вход в систему
312	1	login	\N	84.17.46.67	\N	2025-10-07 16:50:53.246246	Вход в систему
313	1	login	\N	84.17.46.67	\N	2025-10-07 16:53:26.826106	Вход в систему
314	1	login	\N	84.17.46.67	\N	2025-10-07 16:54:45.905038	Вход в систему
315	1	login	\N	84.17.46.67	\N	2025-10-07 16:59:44.024328	Вход в систему
316	1	login	\N	84.17.46.67	\N	2025-10-07 16:59:47.003352	Вход в систему
317	1	login	\N	84.17.46.67	\N	2025-10-07 17:01:48.650748	Вход в систему
318	1	login	\N	84.17.46.67	\N	2025-10-07 17:02:19.420418	Вход в систему
319	1	login	\N	84.17.46.67	\N	2025-10-07 17:03:15.682147	Вход в систему
320	1	login	\N	84.17.46.67	\N	2025-10-07 17:04:49.547228	Вход в систему
321	1	logout	\N	195.98.85.126	\N	2025-10-09 19:48:26.673472	Выход из системы
322	1	logout	\N	195.98.85.126	\N	2025-10-09 19:48:26.676354	Выход из системы
323	1	login	\N	195.98.85.126	\N	2025-10-09 19:48:39.857871	Вход в систему
324	43	login	\N	176.116.165.202	\N	2025-10-28 15:26:45.487661	Вход в систему
325	43	login	\N	176.116.165.202	\N	2025-10-28 15:30:02.875251	Вход в систему
326	1	login	\N	195.98.85.126	\N	2025-10-29 18:13:44.467023	Вход в систему
327	1	login	\N	195.98.85.126	\N	2025-10-29 22:56:29.041145	Вход в систему
328	1	logout	\N	195.98.85.126	\N	2025-10-29 22:57:01.266015	Выход из системы
329	1	logout	\N	195.98.85.126	\N	2025-10-29 22:57:01.319209	Выход из системы
330	1	login	\N	195.98.85.126	\N	2025-10-29 23:00:06.879931	Вход в систему
331	1	login	\N	195.98.85.126	\N	2025-10-29 23:05:24.808365	Вход в систему
332	1	login	\N	195.98.85.126	\N	2025-10-29 23:07:52.035751	Вход в систему
333	1	login	\N	195.98.85.126	\N	2025-10-29 23:11:21.485628	Вход в систему
334	1	login	\N	195.98.85.126	\N	2025-10-29 23:11:58.580367	Вход в систему
335	1	login	\N	195.98.85.126	\N	2025-10-29 23:15:34.146435	Вход в систему
336	1	login	\N	195.98.85.126	\N	2025-10-29 23:15:47.620793	Вход в систему
337	1	login	\N	109.195.53.120	\N	2025-12-04 03:25:10.110052	Вход в систему
338	1	avatar_delete	\N	\N	\N	2025-12-04 03:25:57.971464	Удален аватар профиля
\.


--
-- Data for Name: user_reputation; Type: TABLE DATA; Schema: public; Owner: root
--

COPY public.user_reputation (id, user_id, reputation_score, positive_votes, negative_votes, forum_posts, helpful_posts, reported_bugs, community_contributions, warnings_received, temporary_bans, reputation_penalties, created_at, updated_at) FROM stdin;
2	1	30	0	0	0	0	0	0	0	0	0	2025-08-01 22:59:26.196421	2025-08-06 09:57:49.518427
\.


--
-- Data for Name: user_sessions; Type: TABLE DATA; Schema: public; Owner: owyx
--

COPY public.user_sessions (id, user_id, token_hash, expires_at, ip_address, user_agent, is_active, created_at, last_activity) FROM stdin;
6a7d2300-4dff-4450-b726-5fd12789bd88	1	ZXlKaGJHY2lPaUpJVXpJMU5pSXNJblI1Y0NJNklrcFhWQ0o5LmV5SjFjMlZ5U1dRaU9qRXNJbVZ0WVdsc0lqb2laR2x0WVRKZk1EVkFiV0ZwYkM1eWRTSXNJbkp2YkdVaU9pSmhaRzFwYmlJc0ltbGhkQ0k2TVRjMk5EZ3hPRGN4TUN3aVpYaHdJam94TnpZM05ERXdOekV3ZlEudmE4ZjZfT2N3cG5fdU5jMzdkNTF0VU9fSmhRbkhBM0dqaHFRczJhYzVJUQ==	2026-01-03 03:25:10.077	109.195.53.120	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 YaBrowser/25.10.0.0 Safari/537.36	f	2025-12-04 03:25:10.07821	2025-12-04 03:29:35.870275
\.


--
-- Data for Name: users; Type: TABLE DATA; Schema: public; Owner: root
--

COPY public.users (id, nickname, email, password_hash, first_name, role, trust_level, status, is_active, is_email_verified, is_banned, registered_at, last_login, age, bio, avatar_url, ban_reason, ban_until, discord_username, total_minutes) FROM stdin;
21	Jaymand	gierciek@gmail.com	$2a$12$JBVk9ADYNkBqmtZfu00uL.ocmIpt6maeHASkBV.e5l5rcXkmtOOjm	Максимильяно	admin	3	active	t	t	f	2025-08-06 13:26:04.587145	2025-09-15 04:51:45.280223	20	\N	/uploads/avatars/avatar-21-1754487667213-final.png	\N	\N	\N	2962
35	Maksimkooo	maksrikov2001@gmail.com	$2a$12$l1fsITphu3BMhxUhMSyZ7uGL9QJDQ6TUKPFAEXeBAW8nW0P0W3DyW	Максим	user	0	active	t	t	f	2025-08-19 20:53:30.904929	2025-08-19 20:53:35.727588	20	Я почти олд данного сервера\n	/uploads/avatars/avatar-35-1755636904126-final.png	\N	\N	\N	0
30	Sasasaco	sasasaco64@gmail.com	$2a$12$TACuMzSPZqkyfYhk8wz0s..iJ5AjbV2OxcvxDh0yViTEO8jQvwvT6	\N	user	2	active	t	t	f	2025-08-08 15:13:49.998767	2025-09-23 18:57:37.587486	100	\N	/uploads/avatars/avatar-30-1754666428585-final.png	\N	\N	\N	0
39	korhjik_	agaltuhovs@gmail.com	$2a$12$hXXp4JPoMlo3w3Hcikusg.Q1oaBWnHs45YlQJQYQIJBhSD/umNpfW	Матвей	user	2	active	t	f	f	2025-08-21 15:19:57.10767	2025-10-01 17:20:18.201382	12		\N	\N	\N	\N	1170
37	NeInferno	romeohuaweipro20@gmail.com	$2a$12$7MpBhDrhJF8MhPsrPd6Xy.yMjb/Bk3sam1ytINnmWLdnAQQCsTlhe	Инферно	user	2	active	t	t	f	2025-08-20 12:29:34.796983	2025-08-20 12:32:03.382056	15	\N	\N	\N	\N	\N	48
31	dritesxs564u	digamadhshajaajahahdwd@mail.ru	$2a$12$vaMUunYTAZYSFhe4/MBai.TCyORbex0hkplW99FPa1mSHdB/3kYN2	дима	user	0	active	t	f	f	2025-08-08 15:37:21.142101	2025-08-08 15:37:27.968395	11	\N	\N	\N	\N	\N	0
36	grusha3108	grigoriyv3108@gmail.com	$2a$12$PykYjoUQPgO8nZoLHAE2quaEmswfj5O3gXND3s2lMQWkA0HX9WUda	Grushinonicsrandello	user	2	active	t	t	f	2025-08-19 20:55:05.280231	2025-08-20 12:57:04.146539	100	Первая ихривая флюфка этохо проефта. Буту рата ефли фделаесе меня фембоем😘	\N	\N	\N	\N	2154
32	divay	dudu8245@gmail.com	$2a$12$qghZs1DW/0u1m3/lgKfkceWYLD2K9bx6zqn6JqeQ8PcINfieS4O3i	divay	user	0	active	t	f	f	2025-08-09 17:17:33.906902	2025-08-09 17:17:40.375768	\N	\N	\N	\N	\N	\N	0
33	Bobik	gogihfgjhqshktdhn@gmail.com	$2a$12$TGtv7muXcXE6pDNJjH5TzufZausXVoCdAE9CXW.mxdCSln4dEwSDq	Артур	user	0	active	t	t	f	2025-08-09 17:57:23.267859	2025-08-22 11:14:41.831274	19	\N	\N	\N	\N	\N	5649
29	Myatnyy1337	kumodiche@gmail.com	$2a$12$cXUfyUTLtdhWGtdFg..oNOPtjLSF4X9WXYrkyLbUJ683buHzGOHgu	Мега_Дрочила666	user	3	active	t	t	f	2025-08-06 16:59:13.968136	2025-08-06 16:59:25.141604	14	\N	/uploads/avatars/avatar-29-1754499814035-final.png	\N	\N	\N	0
34	DannybayYT	dannybay48@gmail.com	$2a$12$ce.3YkKejCnMSIFr1cLdQeY.YlIrXLJ1n0U29ZuhhJvAH9yQIlTjS	Dannybay	user	0	active	t	f	f	2025-08-19 18:49:45.539555	2025-08-19 18:49:51.816851	\N	\N	\N	\N	\N	\N	0
40	ArtemSliva	slivinskijartem295@gmail.com	$2a$12$hTGBCG7wl4fMZAbOYsGdBu9/vbPOlQJnT29nmDRevxoXOdC0G7BPy	Artem	user	2	active	t	t	f	2025-08-26 12:58:03.824142	2025-08-29 18:36:13.733149	17	\N	\N	\N	\N	\N	0
1	ebluffy	dima2_05@mail.ru	$2a$12$Wp8pSrr9R1tPyaT7BUW4RuIT2Kdt1YdEWdsrL.J3vvSs6p/am39o2	Дмитрий	admin	3	active	t	t	f	2025-07-26 02:12:51.073	2025-12-04 03:25:10.088865	20	Технический администратор, но я занимаюсь вайбкодингом, а не истинный программист...	\N	\N	\N	\N	555
38	ItzMirk_	superreno12@gmail.com	$2a$12$ESOM97UHXhOMJrXrk828k.dyQdXVvCJz12aQm.P1Gz7g4Bx74ICta		user	2	active	t	t	f	2025-08-20 22:40:23.96726	2025-08-21 11:16:41.065835	21		\N	\N	\N	\N	741
23	ikasaXD	ikasaanim@gmail.com	$2a$12$GouTxoVHW1rZ64Pg.gTiGe3J5o4u0cpy7n55b9VtOCeEBvVJaNckK	Ikasa	user	0	active	t	f	f	2025-08-06 14:58:54.088664	2025-08-20 15:57:41.553555	15	Oh hey there! ✨ So like... deep breath I'm actually a femboy IRL! Took me a while to fully embrace it, but here I am—just being my cute, authentic self! 💖 Life's too short to hide who you are, y'know? Anywayyy, just wanted to say hi and spread some positivity! Stay fabulous! 🌈 blows kiss"	/uploads/avatars/avatar-23-1754492401032-final.png	\N	\N	\N	0
22	xD_zxc	mjhu1@yandex.ru	$2a$12$xpj3KS0iU0exQu.jIDx.e.UrFKCvhI/mLDYytK41u6BQEGE98XdLG	легенда	moderator	3	active	t	f	f	2025-08-06 14:58:40.521672	2025-08-26 13:22:24.628228	19	вы педики xD	/uploads/avatars/avatar-22-1754492543330-final.png	\N	\N	\N	0
43	talaganul24	sboyarka2016@yandex.ru	$2a$12$l46Ye3itSnMm5ObsRuX/NOwCFgvftieim38fjfBz3TXn1wk/vE.0a	\N	user	0	active	t	f	f	2025-10-28 15:26:22.525467	2025-10-28 15:30:02.860622	\N	\N	\N	\N	\N	\N	0
\.


--
-- Name: admin_logs_id_seq; Type: SEQUENCE SET; Schema: public; Owner: root
--

SELECT pg_catalog.setval('public.admin_logs_id_seq', 159, true);


--
-- Name: api_tokens_id_seq; Type: SEQUENCE SET; Schema: public; Owner: owyx
--

SELECT pg_catalog.setval('public.api_tokens_id_seq', 5, true);


--
-- Name: applications_id_seq; Type: SEQUENCE SET; Schema: public; Owner: root
--

SELECT pg_catalog.setval('public.applications_id_seq', 16, true);


--
-- Name: authplugin_id_seq; Type: SEQUENCE SET; Schema: public; Owner: root
--

SELECT pg_catalog.setval('public.authplugin_id_seq', 1, true);


--
-- Name: daily_stats_id_seq; Type: SEQUENCE SET; Schema: public; Owner: owyx
--

SELECT pg_catalog.setval('public.daily_stats_id_seq', 29, true);


--
-- Name: discord_oauth_id_seq; Type: SEQUENCE SET; Schema: public; Owner: owyx
--

SELECT pg_catalog.setval('public.discord_oauth_id_seq', 1, false);


--
-- Name: email_templates_id_seq; Type: SEQUENCE SET; Schema: public; Owner: root
--

SELECT pg_catalog.setval('public.email_templates_id_seq', 12, true);


--
-- Name: email_verification_tokens_id_seq; Type: SEQUENCE SET; Schema: public; Owner: owyx
--

SELECT pg_catalog.setval('public.email_verification_tokens_id_seq', 58, true);


--
-- Name: forum_attachments_id_seq; Type: SEQUENCE SET; Schema: public; Owner: root
--

SELECT pg_catalog.setval('public.forum_attachments_id_seq', 1, false);


--
-- Name: forum_categories_id_seq; Type: SEQUENCE SET; Schema: public; Owner: root
--

SELECT pg_catalog.setval('public.forum_categories_id_seq', 9, true);


--
-- Name: forum_post_edits_id_seq; Type: SEQUENCE SET; Schema: public; Owner: root
--

SELECT pg_catalog.setval('public.forum_post_edits_id_seq', 1, false);


--
-- Name: forum_posts_id_seq; Type: SEQUENCE SET; Schema: public; Owner: root
--

SELECT pg_catalog.setval('public.forum_posts_id_seq', 8, true);


--
-- Name: forum_topics_id_seq; Type: SEQUENCE SET; Schema: public; Owner: root
--

SELECT pg_catalog.setval('public.forum_topics_id_seq', 3, true);


--
-- Name: forum_votes_id_seq; Type: SEQUENCE SET; Schema: public; Owner: root
--

SELECT pg_catalog.setval('public.forum_votes_id_seq', 1, false);


--
-- Name: login_logs_id_seq; Type: SEQUENCE SET; Schema: public; Owner: root
--

SELECT pg_catalog.setval('public.login_logs_id_seq', 166, true);


--
-- Name: password_reset_tokens_id_seq; Type: SEQUENCE SET; Schema: public; Owner: owyx
--

SELECT pg_catalog.setval('public.password_reset_tokens_id_seq', 1, false);


--
-- Name: player_stats_id_seq; Type: SEQUENCE SET; Schema: public; Owner: root
--

SELECT pg_catalog.setval('public.player_stats_id_seq', 22, true);


--
-- Name: reputation_log_id_seq; Type: SEQUENCE SET; Schema: public; Owner: owyx
--

SELECT pg_catalog.setval('public.reputation_log_id_seq', 1, false);


--
-- Name: server_settings_id_seq; Type: SEQUENCE SET; Schema: public; Owner: root
--

SELECT pg_catalog.setval('public.server_settings_id_seq', 1034, true);


--
-- Name: server_status_id_seq; Type: SEQUENCE SET; Schema: public; Owner: owyx
--

SELECT pg_catalog.setval('public.server_status_id_seq', 51, true);


--
-- Name: trust_level_applications_id_seq; Type: SEQUENCE SET; Schema: public; Owner: owyx
--

SELECT pg_catalog.setval('public.trust_level_applications_id_seq', 1, true);


--
-- Name: user_achievements_id_seq; Type: SEQUENCE SET; Schema: public; Owner: owyx
--

SELECT pg_catalog.setval('public.user_achievements_id_seq', 1, false);


--
-- Name: user_activity_id_seq; Type: SEQUENCE SET; Schema: public; Owner: root
--

SELECT pg_catalog.setval('public.user_activity_id_seq', 338, true);


--
-- Name: user_reputation_id_seq; Type: SEQUENCE SET; Schema: public; Owner: root
--

SELECT pg_catalog.setval('public.user_reputation_id_seq', 5, true);


--
-- Name: users_id_seq; Type: SEQUENCE SET; Schema: public; Owner: root
--

SELECT pg_catalog.setval('public.users_id_seq', 43, true);


--
-- Name: admin_logs admin_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.admin_logs
    ADD CONSTRAINT admin_logs_pkey PRIMARY KEY (id);


--
-- Name: api_tokens api_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.api_tokens
    ADD CONSTRAINT api_tokens_pkey PRIMARY KEY (id);


--
-- Name: api_tokens api_tokens_token_hash_key; Type: CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.api_tokens
    ADD CONSTRAINT api_tokens_token_hash_key UNIQUE (token_hash);


--
-- Name: applications applications_pkey; Type: CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.applications
    ADD CONSTRAINT applications_pkey PRIMARY KEY (id);


--
-- Name: authplugin authplugin_pkey; Type: CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.authplugin
    ADD CONSTRAINT authplugin_pkey PRIMARY KEY (id);


--
-- Name: authplugin authplugin_username_key; Type: CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.authplugin
    ADD CONSTRAINT authplugin_username_key UNIQUE (username);


--
-- Name: daily_stats daily_stats_pkey; Type: CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.daily_stats
    ADD CONSTRAINT daily_stats_pkey PRIMARY KEY (id);


--
-- Name: daily_stats daily_stats_user_id_stat_date_key; Type: CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.daily_stats
    ADD CONSTRAINT daily_stats_user_id_stat_date_key UNIQUE (user_id, stat_date);


--
-- Name: discord_oauth discord_oauth_discord_id_key; Type: CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.discord_oauth
    ADD CONSTRAINT discord_oauth_discord_id_key UNIQUE (discord_id);


--
-- Name: discord_oauth discord_oauth_pkey; Type: CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.discord_oauth
    ADD CONSTRAINT discord_oauth_pkey PRIMARY KEY (id);


--
-- Name: email_templates email_templates_pkey; Type: CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.email_templates
    ADD CONSTRAINT email_templates_pkey PRIMARY KEY (id);


--
-- Name: email_templates email_templates_template_key_key; Type: CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.email_templates
    ADD CONSTRAINT email_templates_template_key_key UNIQUE (template_key);


--
-- Name: email_verification_tokens email_verification_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.email_verification_tokens
    ADD CONSTRAINT email_verification_tokens_pkey PRIMARY KEY (id);


--
-- Name: email_verification_tokens email_verification_tokens_token_key; Type: CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.email_verification_tokens
    ADD CONSTRAINT email_verification_tokens_token_key UNIQUE (token);


--
-- Name: forum_attachments forum_attachments_pkey; Type: CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_attachments
    ADD CONSTRAINT forum_attachments_pkey PRIMARY KEY (id);


--
-- Name: forum_categories forum_categories_pkey; Type: CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_categories
    ADD CONSTRAINT forum_categories_pkey PRIMARY KEY (id);


--
-- Name: forum_categories forum_categories_slug_key; Type: CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_categories
    ADD CONSTRAINT forum_categories_slug_key UNIQUE (slug);


--
-- Name: forum_post_edits forum_post_edits_pkey; Type: CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_post_edits
    ADD CONSTRAINT forum_post_edits_pkey PRIMARY KEY (id);


--
-- Name: forum_posts forum_posts_pkey; Type: CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_posts
    ADD CONSTRAINT forum_posts_pkey PRIMARY KEY (id);


--
-- Name: forum_topics forum_topics_pkey; Type: CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_topics
    ADD CONSTRAINT forum_topics_pkey PRIMARY KEY (id);


--
-- Name: forum_votes forum_votes_pkey; Type: CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_votes
    ADD CONSTRAINT forum_votes_pkey PRIMARY KEY (id);


--
-- Name: game_sessions game_sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.game_sessions
    ADD CONSTRAINT game_sessions_pkey PRIMARY KEY (id);


--
-- Name: game_tokens game_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.game_tokens
    ADD CONSTRAINT game_tokens_pkey PRIMARY KEY (id);


--
-- Name: game_tokens game_tokens_token_hash_key; Type: CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.game_tokens
    ADD CONSTRAINT game_tokens_token_hash_key UNIQUE (token_hash);


--
-- Name: login_logs login_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.login_logs
    ADD CONSTRAINT login_logs_pkey PRIMARY KEY (id);


--
-- Name: password_reset_tokens password_reset_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.password_reset_tokens
    ADD CONSTRAINT password_reset_tokens_pkey PRIMARY KEY (id);


--
-- Name: password_reset_tokens password_reset_tokens_token_key; Type: CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.password_reset_tokens
    ADD CONSTRAINT password_reset_tokens_token_key UNIQUE (token);


--
-- Name: player_stats player_stats_pkey; Type: CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.player_stats
    ADD CONSTRAINT player_stats_pkey PRIMARY KEY (id);


--
-- Name: player_stats player_stats_user_id_key; Type: CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.player_stats
    ADD CONSTRAINT player_stats_user_id_key UNIQUE (user_id);


--
-- Name: reputation_log reputation_log_pkey; Type: CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.reputation_log
    ADD CONSTRAINT reputation_log_pkey PRIMARY KEY (id);


--
-- Name: server_settings server_settings_pkey; Type: CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.server_settings
    ADD CONSTRAINT server_settings_pkey PRIMARY KEY (id);


--
-- Name: server_settings server_settings_setting_key_key; Type: CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.server_settings
    ADD CONSTRAINT server_settings_setting_key_key UNIQUE (setting_key);


--
-- Name: server_status server_status_pkey; Type: CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.server_status
    ADD CONSTRAINT server_status_pkey PRIMARY KEY (id);


--
-- Name: server_status server_status_server_ip_server_port_key; Type: CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.server_status
    ADD CONSTRAINT server_status_server_ip_server_port_key UNIQUE (server_ip, server_port);


--
-- Name: trust_level_applications trust_level_applications_pkey; Type: CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.trust_level_applications
    ADD CONSTRAINT trust_level_applications_pkey PRIMARY KEY (id);


--
-- Name: forum_topics unique_topic_slug_per_category; Type: CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_topics
    ADD CONSTRAINT unique_topic_slug_per_category UNIQUE (category_id, slug);


--
-- Name: forum_votes unique_vote_per_user; Type: CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_votes
    ADD CONSTRAINT unique_vote_per_user UNIQUE (post_id, user_id);


--
-- Name: CONSTRAINT unique_vote_per_user ON forum_votes; Type: COMMENT; Schema: public; Owner: root
--

COMMENT ON CONSTRAINT unique_vote_per_user ON public.forum_votes IS 'Один пользователь может проголосовать за пост только один раз';


--
-- Name: user_achievements user_achievements_pkey; Type: CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.user_achievements
    ADD CONSTRAINT user_achievements_pkey PRIMARY KEY (id);


--
-- Name: user_activity user_activity_pkey; Type: CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.user_activity
    ADD CONSTRAINT user_activity_pkey PRIMARY KEY (id);


--
-- Name: user_reputation user_reputation_pkey; Type: CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.user_reputation
    ADD CONSTRAINT user_reputation_pkey PRIMARY KEY (id);


--
-- Name: user_reputation user_reputation_user_id_key; Type: CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.user_reputation
    ADD CONSTRAINT user_reputation_user_id_key UNIQUE (user_id);


--
-- Name: user_sessions user_sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.user_sessions
    ADD CONSTRAINT user_sessions_pkey PRIMARY KEY (id);


--
-- Name: users users_email_key; Type: CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_email_key UNIQUE (email);


--
-- Name: users users_nickname_key; Type: CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_nickname_key UNIQUE (nickname);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: idx_admin_logs_admin_id; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_admin_logs_admin_id ON public.admin_logs USING btree (admin_id);


--
-- Name: idx_admin_logs_created_at; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_admin_logs_created_at ON public.admin_logs USING btree (created_at);


--
-- Name: idx_admin_logs_target_user_id; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_admin_logs_target_user_id ON public.admin_logs USING btree (target_user_id);


--
-- Name: idx_api_tokens_expires_at; Type: INDEX; Schema: public; Owner: owyx
--

CREATE INDEX idx_api_tokens_expires_at ON public.api_tokens USING btree (expires_at);


--
-- Name: idx_api_tokens_is_active; Type: INDEX; Schema: public; Owner: owyx
--

CREATE INDEX idx_api_tokens_is_active ON public.api_tokens USING btree (is_active);


--
-- Name: idx_api_tokens_token_hash; Type: INDEX; Schema: public; Owner: owyx
--

CREATE INDEX idx_api_tokens_token_hash ON public.api_tokens USING btree (token_hash);


--
-- Name: idx_api_tokens_user_id; Type: INDEX; Schema: public; Owner: owyx
--

CREATE INDEX idx_api_tokens_user_id ON public.api_tokens USING btree (user_id);


--
-- Name: idx_applications_status; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_applications_status ON public.applications USING btree (status);


--
-- Name: idx_applications_submitted_at; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_applications_submitted_at ON public.applications USING btree (submitted_at);


--
-- Name: idx_authplugin_email; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_authplugin_email ON public.authplugin USING btree (email);


--
-- Name: idx_authplugin_login_type; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_authplugin_login_type ON public.authplugin USING btree (login_type);


--
-- Name: idx_authplugin_username; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_authplugin_username ON public.authplugin USING btree (username);


--
-- Name: idx_authplugin_uuid; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_authplugin_uuid ON public.authplugin USING btree (uuid);


--
-- Name: idx_forum_attachments_post; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_forum_attachments_post ON public.forum_attachments USING btree (post_id);


--
-- Name: idx_forum_attachments_topic; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_forum_attachments_topic ON public.forum_attachments USING btree (topic_id);


--
-- Name: idx_forum_attachments_uploader; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_forum_attachments_uploader ON public.forum_attachments USING btree (uploader_id);


--
-- Name: idx_forum_categories_parent; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_forum_categories_parent ON public.forum_categories USING btree (parent_id);


--
-- Name: idx_forum_categories_position; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_forum_categories_position ON public.forum_categories USING btree ("position");


--
-- Name: idx_forum_categories_slug; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_forum_categories_slug ON public.forum_categories USING btree (slug);


--
-- Name: idx_forum_post_edits_editor; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_forum_post_edits_editor ON public.forum_post_edits USING btree (edited_by);


--
-- Name: idx_forum_post_edits_post; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_forum_post_edits_post ON public.forum_post_edits USING btree (post_id, created_at DESC);


--
-- Name: idx_forum_posts_author; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_forum_posts_author ON public.forum_posts USING btree (author_id);


--
-- Name: idx_forum_posts_created; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_forum_posts_created ON public.forum_posts USING btree (created_at DESC);


--
-- Name: idx_forum_posts_parent; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_forum_posts_parent ON public.forum_posts USING btree (parent_post_id);


--
-- Name: idx_forum_posts_topic; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_forum_posts_topic ON public.forum_posts USING btree (topic_id, created_at);


--
-- Name: idx_forum_topics_author; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_forum_topics_author ON public.forum_topics USING btree (author_id);


--
-- Name: idx_forum_topics_category; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_forum_topics_category ON public.forum_topics USING btree (category_id);


--
-- Name: idx_forum_topics_created; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_forum_topics_created ON public.forum_topics USING btree (created_at DESC);


--
-- Name: idx_forum_topics_last_post; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_forum_topics_last_post ON public.forum_topics USING btree (last_post_at DESC);


--
-- Name: idx_forum_topics_pinned; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_forum_topics_pinned ON public.forum_topics USING btree (is_pinned, last_post_at DESC);


--
-- Name: idx_forum_topics_tags; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_forum_topics_tags ON public.forum_topics USING gin (tags);


--
-- Name: idx_forum_votes_post; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_forum_votes_post ON public.forum_votes USING btree (post_id);


--
-- Name: idx_forum_votes_user; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_forum_votes_user ON public.forum_votes USING btree (user_id);


--
-- Name: idx_game_sessions_active; Type: INDEX; Schema: public; Owner: owyx
--

CREATE INDEX idx_game_sessions_active ON public.game_sessions USING btree (is_active);


--
-- Name: idx_game_sessions_expires_at; Type: INDEX; Schema: public; Owner: owyx
--

CREATE INDEX idx_game_sessions_expires_at ON public.game_sessions USING btree (expires_at);


--
-- Name: idx_game_sessions_player_uuid; Type: INDEX; Schema: public; Owner: owyx
--

CREATE INDEX idx_game_sessions_player_uuid ON public.game_sessions USING btree (player_uuid);


--
-- Name: idx_game_sessions_unique_active; Type: INDEX; Schema: public; Owner: owyx
--

CREATE UNIQUE INDEX idx_game_sessions_unique_active ON public.game_sessions USING btree (user_id, player_uuid, is_active) WHERE (is_active = true);


--
-- Name: idx_game_sessions_user_id; Type: INDEX; Schema: public; Owner: owyx
--

CREATE INDEX idx_game_sessions_user_id ON public.game_sessions USING btree (user_id);


--
-- Name: idx_game_tokens_expires_at; Type: INDEX; Schema: public; Owner: owyx
--

CREATE INDEX idx_game_tokens_expires_at ON public.game_tokens USING btree (expires_at);


--
-- Name: idx_game_tokens_token_hash; Type: INDEX; Schema: public; Owner: owyx
--

CREATE INDEX idx_game_tokens_token_hash ON public.game_tokens USING btree (token_hash);


--
-- Name: idx_game_tokens_user_id; Type: INDEX; Schema: public; Owner: owyx
--

CREATE INDEX idx_game_tokens_user_id ON public.game_tokens USING btree (user_id);


--
-- Name: idx_login_logs_login_time; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_login_logs_login_time ON public.login_logs USING btree (login_time);


--
-- Name: idx_login_logs_user_id; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_login_logs_user_id ON public.login_logs USING btree (user_id);


--
-- Name: idx_player_stats_last_seen; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_player_stats_last_seen ON public.player_stats USING btree (last_seen);


--
-- Name: idx_player_stats_updated; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_player_stats_updated ON public.player_stats USING btree (stats_last_updated);


--
-- Name: idx_player_stats_user_id; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_player_stats_user_id ON public.player_stats USING btree (user_id);


--
-- Name: idx_server_settings_category; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_server_settings_category ON public.server_settings USING btree (category);


--
-- Name: idx_server_settings_setting_key; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_server_settings_setting_key ON public.server_settings USING btree (setting_key);


--
-- Name: idx_user_sessions_expires_at; Type: INDEX; Schema: public; Owner: owyx
--

CREATE INDEX idx_user_sessions_expires_at ON public.user_sessions USING btree (expires_at);


--
-- Name: idx_user_sessions_is_active; Type: INDEX; Schema: public; Owner: owyx
--

CREATE INDEX idx_user_sessions_is_active ON public.user_sessions USING btree (is_active);


--
-- Name: idx_user_sessions_last_activity; Type: INDEX; Schema: public; Owner: owyx
--

CREATE INDEX idx_user_sessions_last_activity ON public.user_sessions USING btree (last_activity) WHERE (is_active = true);


--
-- Name: idx_user_sessions_token_hash; Type: INDEX; Schema: public; Owner: owyx
--

CREATE INDEX idx_user_sessions_token_hash ON public.user_sessions USING btree (token_hash);


--
-- Name: idx_user_sessions_user_id; Type: INDEX; Schema: public; Owner: owyx
--

CREATE INDEX idx_user_sessions_user_id ON public.user_sessions USING btree (user_id);


--
-- Name: idx_users_email; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_users_email ON public.users USING btree (email);


--
-- Name: idx_users_nickname; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_users_nickname ON public.users USING btree (nickname);


--
-- Name: idx_users_role; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_users_role ON public.users USING btree (role);


--
-- Name: idx_users_trust_level; Type: INDEX; Schema: public; Owner: root
--

CREATE INDEX idx_users_trust_level ON public.users USING btree (trust_level);


--
-- Name: users trigger_check_ban_expiry; Type: TRIGGER; Schema: public; Owner: root
--

CREATE TRIGGER trigger_check_ban_expiry BEFORE UPDATE ON public.users FOR EACH ROW EXECUTE FUNCTION public.check_ban_expiry();


--
-- Name: player_stats trigger_sync_user_total_minutes; Type: TRIGGER; Schema: public; Owner: root
--

CREATE TRIGGER trigger_sync_user_total_minutes AFTER UPDATE OF time_played_minutes ON public.player_stats FOR EACH ROW EXECUTE FUNCTION public.sync_user_total_minutes();


--
-- Name: forum_topics trigger_update_category_counts; Type: TRIGGER; Schema: public; Owner: root
--

CREATE TRIGGER trigger_update_category_counts AFTER INSERT OR DELETE ON public.forum_topics FOR EACH ROW EXECUTE FUNCTION public.update_forum_counts();


--
-- Name: forum_posts trigger_update_topic_counts; Type: TRIGGER; Schema: public; Owner: root
--

CREATE TRIGGER trigger_update_topic_counts AFTER INSERT OR DELETE ON public.forum_posts FOR EACH ROW EXECUTE FUNCTION public.update_forum_counts();


--
-- Name: applications trigger_update_user_age; Type: TRIGGER; Schema: public; Owner: root
--

CREATE TRIGGER trigger_update_user_age AFTER UPDATE ON public.applications FOR EACH ROW EXECUTE FUNCTION public.update_user_age_from_application();


--
-- Name: forum_votes trigger_update_votes; Type: TRIGGER; Schema: public; Owner: root
--

CREATE TRIGGER trigger_update_votes AFTER INSERT OR DELETE OR UPDATE ON public.forum_votes FOR EACH ROW EXECUTE FUNCTION public.update_post_votes();


--
-- Name: admin_logs admin_logs_admin_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.admin_logs
    ADD CONSTRAINT admin_logs_admin_id_fkey FOREIGN KEY (admin_id) REFERENCES public.users(id);


--
-- Name: api_tokens api_tokens_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.api_tokens
    ADD CONSTRAINT api_tokens_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: api_tokens api_tokens_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.api_tokens
    ADD CONSTRAINT api_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: applications applications_reviewed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.applications
    ADD CONSTRAINT applications_reviewed_by_fkey FOREIGN KEY (reviewed_by) REFERENCES public.users(id);


--
-- Name: applications applications_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.applications
    ADD CONSTRAINT applications_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: daily_stats daily_stats_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.daily_stats
    ADD CONSTRAINT daily_stats_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: discord_oauth discord_oauth_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.discord_oauth
    ADD CONSTRAINT discord_oauth_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: email_templates email_templates_updated_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.email_templates
    ADD CONSTRAINT email_templates_updated_by_fkey FOREIGN KEY (updated_by) REFERENCES public.users(id);


--
-- Name: email_verification_tokens email_verification_tokens_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.email_verification_tokens
    ADD CONSTRAINT email_verification_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: authplugin fk_user_email; Type: FK CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.authplugin
    ADD CONSTRAINT fk_user_email FOREIGN KEY (email) REFERENCES public.users(email) ON DELETE SET NULL;


--
-- Name: forum_attachments forum_attachments_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_attachments
    ADD CONSTRAINT forum_attachments_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.forum_posts(id) ON DELETE CASCADE;


--
-- Name: forum_attachments forum_attachments_topic_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_attachments
    ADD CONSTRAINT forum_attachments_topic_id_fkey FOREIGN KEY (topic_id) REFERENCES public.forum_topics(id) ON DELETE CASCADE;


--
-- Name: forum_attachments forum_attachments_uploader_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_attachments
    ADD CONSTRAINT forum_attachments_uploader_id_fkey FOREIGN KEY (uploader_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: forum_categories forum_categories_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_categories
    ADD CONSTRAINT forum_categories_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: forum_categories forum_categories_parent_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_categories
    ADD CONSTRAINT forum_categories_parent_id_fkey FOREIGN KEY (parent_id) REFERENCES public.forum_categories(id) ON DELETE CASCADE;


--
-- Name: forum_post_edits forum_post_edits_edited_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_post_edits
    ADD CONSTRAINT forum_post_edits_edited_by_fkey FOREIGN KEY (edited_by) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: forum_post_edits forum_post_edits_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_post_edits
    ADD CONSTRAINT forum_post_edits_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.forum_posts(id) ON DELETE CASCADE;


--
-- Name: forum_posts forum_posts_author_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_posts
    ADD CONSTRAINT forum_posts_author_id_fkey FOREIGN KEY (author_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: forum_posts forum_posts_deleted_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_posts
    ADD CONSTRAINT forum_posts_deleted_by_fkey FOREIGN KEY (deleted_by) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: forum_posts forum_posts_parent_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_posts
    ADD CONSTRAINT forum_posts_parent_post_id_fkey FOREIGN KEY (parent_post_id) REFERENCES public.forum_posts(id) ON DELETE CASCADE;


--
-- Name: forum_posts forum_posts_topic_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_posts
    ADD CONSTRAINT forum_posts_topic_id_fkey FOREIGN KEY (topic_id) REFERENCES public.forum_topics(id) ON DELETE CASCADE;


--
-- Name: forum_topics forum_topics_author_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_topics
    ADD CONSTRAINT forum_topics_author_id_fkey FOREIGN KEY (author_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: forum_topics forum_topics_category_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_topics
    ADD CONSTRAINT forum_topics_category_id_fkey FOREIGN KEY (category_id) REFERENCES public.forum_categories(id) ON DELETE CASCADE;


--
-- Name: forum_votes forum_votes_post_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_votes
    ADD CONSTRAINT forum_votes_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.forum_posts(id) ON DELETE CASCADE;


--
-- Name: forum_votes forum_votes_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.forum_votes
    ADD CONSTRAINT forum_votes_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: game_sessions game_sessions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.game_sessions
    ADD CONSTRAINT game_sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: game_tokens game_tokens_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.game_tokens
    ADD CONSTRAINT game_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: login_logs login_logs_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.login_logs
    ADD CONSTRAINT login_logs_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: password_reset_tokens password_reset_tokens_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.password_reset_tokens
    ADD CONSTRAINT password_reset_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: player_stats player_stats_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.player_stats
    ADD CONSTRAINT player_stats_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: reputation_log reputation_log_from_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.reputation_log
    ADD CONSTRAINT reputation_log_from_user_id_fkey FOREIGN KEY (from_user_id) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: reputation_log reputation_log_to_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.reputation_log
    ADD CONSTRAINT reputation_log_to_user_id_fkey FOREIGN KEY (to_user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: server_settings server_settings_updated_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.server_settings
    ADD CONSTRAINT server_settings_updated_by_fkey FOREIGN KEY (updated_by) REFERENCES public.users(id);


--
-- Name: trust_level_applications trust_level_applications_reviewed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.trust_level_applications
    ADD CONSTRAINT trust_level_applications_reviewed_by_fkey FOREIGN KEY (reviewed_by) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: trust_level_applications trust_level_applications_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.trust_level_applications
    ADD CONSTRAINT trust_level_applications_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_achievements user_achievements_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.user_achievements
    ADD CONSTRAINT user_achievements_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_activity user_activity_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.user_activity
    ADD CONSTRAINT user_activity_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_reputation user_reputation_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: root
--

ALTER TABLE ONLY public.user_reputation
    ADD CONSTRAINT user_reputation_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_sessions user_sessions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: owyx
--

ALTER TABLE ONLY public.user_sessions
    ADD CONSTRAINT user_sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--

\unrestrict sM3QiBjqwUNDlfPEt5IU9ijbKAsxY9oDXmqmVvGwhGtVYa9bbVR9GJxDIFcgAKX

