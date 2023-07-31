CREATE USER mangaka WITH
NOINHERIT
PASSWORD 'mangakapwd';

CREATE TABLE public.manga
(
    id smallserial NOT NULL,
    title character varying(40) NOT NULL,
    chapter smallint NOT NULL,
    chapter_title character varying(40) NOT NULL,
    urls text[] NOT NULL,
    PRIMARY KEY (id)
);


ALTER TABLE IF EXISTS public.manga
    OWNER to mangaka;
