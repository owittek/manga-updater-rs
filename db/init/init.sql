CREATE USER mangaka WITH
NOINHERIT
PASSWORD 'mangakapwd';

CREATE TABLE public.manga
(
    id serial NOT NULL,
    title character varying(40) NOT NULL,
    chapter integer NOT NULL DEFAULT 0,
    urls text[] NOT NULL,
    PRIMARY KEY (id)
);


ALTER TABLE IF EXISTS public.manga
    OWNER to mangaka;
