--
-- PostgreSQL database dump
--

-- Dumped from database version 16.9 (Ubuntu 16.9-0ubuntu0.24.04.1)
-- Dumped by pg_dump version 17.5

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: products; Type: TABLE; Schema: public; Owner: klave
--

CREATE TABLE public.products (
    id integer NOT NULL,
    product_name character varying(255),
    category character varying(255),
    brand character varying(255),
    description character varying(255),
    price numeric,
    quantity integer,
    size numeric,
    color character varying(255),
    material character varying(255),
    sku character varying(255),
    weight integer,
    is_active boolean
);


ALTER TABLE public.products OWNER TO klave;

--
-- Name: products products_pkey; Type: CONSTRAINT; Schema: public; Owner: klave
--

ALTER TABLE ONLY public.products
    ADD CONSTRAINT products_pkey PRIMARY KEY (id);


--
-- PostgreSQL database dump complete
--

