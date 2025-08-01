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
-- Name: purchases; Type: TABLE; Schema: public; Owner: klave
--

CREATE TABLE public.purchases (
    id integer NOT NULL,
    user_id integer,
    product_id integer,
    purchase_date character varying(255),
    purchased_quantity integer,
    payment_method character varying(255),
    total_price integer
);


ALTER TABLE public.purchases OWNER TO klave;

--
-- Name: purchases purchases_pkey; Type: CONSTRAINT; Schema: public; Owner: klave
--

ALTER TABLE ONLY public.purchases
    ADD CONSTRAINT purchases_pkey PRIMARY KEY (id);


--
-- Name: purchases product_id; Type: FK CONSTRAINT; Schema: public; Owner: klave
--

ALTER TABLE ONLY public.purchases
    ADD CONSTRAINT product_id FOREIGN KEY (product_id) REFERENCES public.products(id);


--
-- Name: purchases user_id; Type: FK CONSTRAINT; Schema: public; Owner: klave
--

ALTER TABLE ONLY public.purchases
    ADD CONSTRAINT user_id FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- PostgreSQL database dump complete
--

