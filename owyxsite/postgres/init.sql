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


-- ============================================================
-- TABLE DATA omitted from git (PII / tokens / password hashes).
-- Local full dump (gitignored): postgres/init.full.sql
-- ============================================================

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

