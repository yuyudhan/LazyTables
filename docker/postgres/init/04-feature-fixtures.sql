-- FilePath: docker/postgres/init/04-feature-fixtures.sql

-- Feature-test fixtures layered on top of 01-03:
--   * extra schemas sales, inventory, "CamelCase Schema" (needs quoting), restricted
--   * same table name in several schemas (products in public, sales, inventory)
--   * cross-schema foreign keys, views, a materialized view, enums, arrays, json/jsonb
--   * public.all_types (one column per common type), public.big_events (100k rows)
--   * unicode, NULLs and long text values
--   * login role lt_reader (password lt_reader) that cannot use schema restricted

\echo 'Loading LazyTables feature fixtures...'

CREATE SCHEMA IF NOT EXISTS sales;
CREATE SCHEMA IF NOT EXISTS inventory;
CREATE SCHEMA IF NOT EXISTS "CamelCase Schema";
CREATE SCHEMA IF NOT EXISTS restricted;

-- Enums ---------------------------------------------------------------
CREATE TYPE sales.order_status AS ENUM ('draft', 'placed', 'shipped', 'delivered', 'cancelled');
CREATE TYPE public.mood AS ENUM ('happy', 'neutral', 'sad');

-- Same table name "products" in sales and inventory (public.products exists) --
CREATE TABLE sales.products (
    id SERIAL PRIMARY KEY,
    public_product_id INTEGER REFERENCES public.products(id),
    list_price NUMERIC(12, 2) NOT NULL,
    tags TEXT[] NOT NULL DEFAULT '{}',
    attributes JSONB
);

CREATE TABLE inventory.products (
    id SERIAL PRIMARY KEY,
    sales_product_id INTEGER NOT NULL REFERENCES sales.products(id),
    warehouse TEXT NOT NULL,
    bin_codes VARCHAR(10)[],
    quantity INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE sales.orders (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES public.users(id),
    status sales.order_status NOT NULL DEFAULT 'draft',
    lines JSON,
    note TEXT,
    placed_at TIMESTAMPTZ
);

CREATE TABLE "CamelCase Schema"."OrderLines" (
    "Id" SERIAL PRIMARY KEY,
    "OrderId" INTEGER REFERENCES sales.orders(id),
    "Product Name" TEXT NOT NULL,
    "Qty" INTEGER NOT NULL
);

CREATE TABLE restricted.secrets (
    id SERIAL PRIMARY KEY,
    label TEXT NOT NULL,
    secret_value TEXT NOT NULL
);

INSERT INTO sales.products (public_product_id, list_price, tags, attributes)
SELECT p.id,
       p.price * 1.2,
       ARRAY['tag-' || (p.id % 3), 'promo'],
       jsonb_build_object('color', CASE WHEN p.id % 2 = 0 THEN 'red' ELSE 'blue' END, 'size', p.id)
FROM public.products p;

INSERT INTO sales.products (public_product_id, list_price, tags, attributes) VALUES
    (NULL, 9.99, '{}', NULL),
    (NULL, 1234.50, ARRAY['ünïcödé', '日本語', 'emoji 🚀'], '{"name": "Ωmega", "nested": {"k": [1, 2, 3]}}');

INSERT INTO inventory.products (sales_product_id, warehouse, bin_codes, quantity)
SELECT sp.id, w.name, ARRAY['A' || sp.id, 'B' || sp.id], (sp.id * 7) % 50
FROM sales.products sp
CROSS JOIN (VALUES ('north'), ('south')) AS w(name);

INSERT INTO sales.orders (user_id, status, lines, note, placed_at)
SELECT u.id,
       (ARRAY['draft', 'placed', 'shipped', 'delivered', 'cancelled']::sales.order_status[])[1 + u.id % 5],
       json_build_array(json_build_object('sku', 'SKU-' || u.id, 'qty', u.id % 4 + 1)),
       CASE WHEN u.id % 3 = 0 THEN NULL ELSE 'order for ' || u.username END,
       CASE WHEN u.id % 4 = 0 THEN NULL ELSE now() - (u.id || ' days')::interval END
FROM public.users u;

INSERT INTO "CamelCase Schema"."OrderLines" ("OrderId", "Product Name", "Qty")
SELECT o.id, 'Line for order ' || o.id, o.id % 5 + 1 FROM sales.orders o;

INSERT INTO restricted.secrets (label, secret_value) VALUES
    ('api', 'not-for-lt_reader'),
    ('db', 'still-not-for-lt_reader');

-- Views ------------------------------------------------------------------
CREATE VIEW sales.v_open_orders AS
SELECT o.id, u.username, o.status, o.placed_at
FROM sales.orders o
LEFT JOIN public.users u ON u.id = o.user_id
WHERE o.status IN ('draft', 'placed', 'shipped');

CREATE MATERIALIZED VIEW inventory.mv_stock_by_warehouse AS
SELECT warehouse, count(*) AS product_count, sum(quantity) AS total_quantity
FROM inventory.products
GROUP BY warehouse;

-- Every common type in one table -----------------------------------------
CREATE TABLE public.all_types (
    id BIGSERIAL PRIMARY KEY,
    c_smallint SMALLINT,
    c_integer INTEGER,
    c_bigint BIGINT,
    c_numeric NUMERIC(20, 6),
    c_real REAL,
    c_double DOUBLE PRECISION,
    c_money MONEY,
    c_boolean BOOLEAN,
    c_char CHAR(5),
    c_varchar VARCHAR(255),
    c_text TEXT,
    c_bytea BYTEA,
    c_date DATE,
    c_time TIME,
    c_timetz TIMETZ,
    c_timestamp TIMESTAMP,
    c_timestamptz TIMESTAMPTZ,
    c_interval INTERVAL,
    c_uuid UUID,
    c_json JSON,
    c_jsonb JSONB,
    c_int_array INTEGER[],
    c_text_array TEXT[],
    c_inet INET,
    c_cidr CIDR,
    c_macaddr MACADDR,
    c_point POINT,
    c_tsvector TSVECTOR,
    c_xml XML,
    c_bit BIT(8),
    c_varbit VARBIT(16),
    c_int4range INT4RANGE,
    c_mood public.mood
);

INSERT INTO public.all_types (
    c_smallint, c_integer, c_bigint, c_numeric, c_real, c_double, c_money, c_boolean,
    c_char, c_varchar, c_text, c_bytea, c_date, c_time, c_timetz, c_timestamp,
    c_timestamptz, c_interval, c_uuid, c_json, c_jsonb, c_int_array, c_text_array,
    c_inet, c_cidr, c_macaddr, c_point, c_tsvector, c_xml, c_bit, c_varbit,
    c_int4range, c_mood
) VALUES
    (1, 42, 9007199254740993, 12345.678901, 1.5, 2.718281828, 19.99, true,
     'abc', 'plain ascii', 'short text', '\xdeadbeef', '2024-02-29', '13:37:00', '13:37:00+05:30',
     '2024-01-01 00:00:00', '2024-01-01 00:00:00+00', '1 day 2 hours', 'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11',
     '{"a": 1}', '{"b": [true, null]}', '{1,2,3}', '{"x","y"}',
     '192.168.0.1', '10.0.0.0/8', '08:00:2b:01:02:03', '(1,2)', to_tsvector('english', 'lazy tables rock'),
     '<root><v>1</v></root>', B'10101010', B'101', '[1,10)', 'happy'),
    (-32768, -2147483648, -9223372036854775808, -0.000001, 'NaN', 'Infinity', -1.00, false,
     'ü', 'ünïcödé 日本語 العربية emoji 🚀', repeat('long text line. ', 4000), '\x00', '0001-01-01', '00:00:00', '00:00:00+00',
     '1970-01-01 00:00:00', '9999-12-31 23:59:59+00', '-3 years', '00000000-0000-0000-0000-000000000000',
     '[]', '{}', '{}', '{NULL,"with,comma","with \"quote\""}',
     '::1', '2001:db8::/32', 'ff:ff:ff:ff:ff:ff', '(-1.5,0)', ''::tsvector,
     '<empty/>', B'00000000', B'', 'empty', 'sad'),
    (NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,
     NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,
     NULL, NULL, NULL, NULL, NULL, NULL, NULL,
     NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,
     NULL, NULL);

-- Large table for paging (100k rows) ---------------------------------------
CREATE TABLE public.big_events (
    id BIGINT PRIMARY KEY,
    user_id INTEGER REFERENCES public.users(id),
    kind TEXT NOT NULL,
    payload JSONB,
    note TEXT,
    created_at TIMESTAMPTZ NOT NULL
);

INSERT INTO public.big_events (id, user_id, kind, payload, note, created_at)
SELECT g,
       CASE WHEN g % 10 = 0 THEN NULL ELSE (SELECT min(id) FROM public.users) END,
       (ARRAY['click', 'view', 'purchase', 'signup'])[1 + g % 4],
       jsonb_build_object('n', g, 'even', g % 2 = 0),
       CASE WHEN g % 7 = 0 THEN NULL WHEN g % 1000 = 0 THEN repeat('ü', 5000) ELSE 'event ' || g END,
       timestamptz '2024-01-01' + (g || ' seconds')::interval
FROM generate_series(1, 100000) AS g;

CREATE INDEX CONCURRENTLY big_events_kind_idx ON public.big_events (kind);

-- Limited login role --------------------------------------------------------
CREATE ROLE lt_reader LOGIN PASSWORD 'lt_reader';
GRANT CONNECT ON DATABASE lazytables TO lt_reader;
GRANT USAGE ON SCHEMA public, analytics, admin, sales, inventory, "CamelCase Schema" TO lt_reader;
GRANT SELECT ON ALL TABLES IN SCHEMA public, analytics, admin, sales, inventory, "CamelCase Schema" TO lt_reader;
REVOKE ALL ON SCHEMA restricted FROM PUBLIC;

ANALYZE;

\echo 'LazyTables feature fixtures loaded.'
