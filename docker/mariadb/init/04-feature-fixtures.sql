-- FilePath: docker/mariadb/init/04-feature-fixtures.sql
-- (identical copy in docker/mysql/init; keep both in sync)

-- Feature-test fixtures layered on top of 01-03 (test_db):
--   * extra databases lt_sales, lt_inventory, lt_empty, lt_restricted
--   * same table name "products" in test_db, lt_sales and lt_inventory, cross-database FK
--   * test_db.all_types (one column per common type), test_db.big_events (100k rows)
--   * unicode, NULLs and long text values
--   * user lt_reader (password lt_reader) without access to lt_restricted

SET NAMES utf8mb4;

CREATE DATABASE IF NOT EXISTS lt_sales CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
CREATE DATABASE IF NOT EXISTS lt_inventory CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
CREATE DATABASE IF NOT EXISTS lt_empty;
CREATE DATABASE IF NOT EXISTS lt_restricted;

CREATE TABLE lt_sales.products (
    id INT AUTO_INCREMENT PRIMARY KEY,
    test_db_product_id INT NULL,
    list_price DECIMAL(12, 2) NOT NULL,
    status ENUM('draft', 'active', 'retired') NOT NULL DEFAULT 'active',
    attributes JSON NULL,
    CONSTRAINT fk_sales_products_test_db FOREIGN KEY (test_db_product_id) REFERENCES test_db.products(id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE lt_inventory.products (
    id INT AUTO_INCREMENT PRIMARY KEY,
    sales_product_id INT NOT NULL,
    warehouse VARCHAR(50) NOT NULL,
    quantity INT NOT NULL DEFAULT 0,
    CONSTRAINT fk_inventory_products_sales FOREIGN KEY (sales_product_id) REFERENCES lt_sales.products(id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE lt_restricted.secrets (
    id INT AUTO_INCREMENT PRIMARY KEY,
    label VARCHAR(50) NOT NULL,
    secret_value VARCHAR(100) NOT NULL
) ENGINE=InnoDB;

INSERT INTO lt_sales.products (test_db_product_id, list_price, status, attributes)
SELECT id, price * 1.2, 'active', JSON_OBJECT('size', id) FROM test_db.products;

INSERT INTO lt_sales.products (test_db_product_id, list_price, status, attributes) VALUES
    (NULL, 9.99, 'draft', NULL),
    (NULL, 1234.50, 'retired', '{"name": "Ωmega 日本語 🚀", "nested": {"k": [1, 2, 3]}}');

INSERT INTO lt_inventory.products (sales_product_id, warehouse, quantity)
SELECT id, 'north', id % 50 FROM lt_sales.products
UNION ALL
SELECT id, 'south', (id * 7) % 50 FROM lt_sales.products;

INSERT INTO lt_restricted.secrets (label, secret_value) VALUES ('api', 'not-for-lt_reader');

CREATE VIEW lt_sales.v_active_products AS
SELECT id, list_price FROM lt_sales.products WHERE status = 'active';

USE test_db;

CREATE TABLE all_types (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    c_tinyint TINYINT,
    c_smallint SMALLINT,
    c_mediumint MEDIUMINT,
    c_int INT,
    c_bigint BIGINT,
    c_unsigned BIGINT UNSIGNED,
    c_decimal DECIMAL(20, 6),
    c_float FLOAT,
    c_double DOUBLE,
    c_bit BIT(8),
    c_bool BOOLEAN,
    c_char CHAR(5),
    c_varchar VARCHAR(255),
    c_tinytext TINYTEXT,
    c_text TEXT,
    c_longtext LONGTEXT,
    c_binary BINARY(4),
    c_varbinary VARBINARY(16),
    c_blob BLOB,
    c_date DATE,
    c_time TIME,
    c_datetime DATETIME(6),
    c_timestamp TIMESTAMP NULL,
    c_year YEAR,
    c_enum ENUM('small', 'medium', 'large'),
    c_set SET('red', 'green', 'blue'),
    c_json JSON
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

INSERT INTO all_types (
    c_tinyint, c_smallint, c_mediumint, c_int, c_bigint, c_unsigned, c_decimal, c_float, c_double,
    c_bit, c_bool, c_char, c_varchar, c_tinytext, c_text, c_longtext, c_binary, c_varbinary, c_blob,
    c_date, c_time, c_datetime, c_timestamp, c_year, c_enum, c_set, c_json
) VALUES
    (1, 42, 8388607, 2147483647, 9007199254740993, 18446744073709551615, 12345.678901, 1.5, 2.718281828,
     b'10101010', TRUE, 'abc', 'plain ascii', 'tiny', 'short text', 'long', X'DEADBEEF', X'00FF', X'CAFEBABE',
     '2024-02-29', '13:37:00', '2024-01-01 00:00:00.123456', '2024-01-01 00:00:00', 2024, 'medium', 'red,blue', '{"a": 1}'),
    (-128, -32768, -8388608, -2147483648, -9223372036854775807, 0, -0.000001, -1.5, -1e300,
     b'00000000', FALSE, 'ü', 'ünïcödé 日本語 العربية emoji 🚀', '', '', REPEAT('long text line. ', 4000), X'00000000', X'', X'',
     '1000-01-01', '-838:59:59', '1000-01-01 00:00:00', '1970-01-01 00:00:01', 1901, 'small', '', '[]'),
    (NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,
     NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,
     NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);

-- Helper for generating sequences without recursive CTE limits (kept as a tiny fixture).
CREATE TABLE digits (d INT PRIMARY KEY) ENGINE=InnoDB;
INSERT INTO digits VALUES (0), (1), (2), (3), (4), (5), (6), (7), (8), (9);

CREATE TABLE big_events (
    id INT PRIMARY KEY,
    kind VARCHAR(20) NOT NULL,
    payload JSON NULL,
    note TEXT NULL,
    created_at DATETIME NOT NULL,
    INDEX big_events_kind_idx (kind)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

INSERT INTO big_events (id, kind, payload, note, created_at)
SELECT n,
       ELT(1 + n % 4, 'click', 'view', 'purchase', 'signup'),
       JSON_OBJECT('n', n, 'even', n % 2 = 0),
       CASE WHEN n % 7 = 0 THEN NULL WHEN n % 1000 = 0 THEN REPEAT('ü', 5000) ELSE CONCAT('event ', n) END,
       TIMESTAMPADD(SECOND, n, '2024-01-01 00:00:00')
FROM (
    SELECT 1 + a.d + 10 * b.d + 100 * c.d + 1000 * e.d + 10000 * f.d AS n
    FROM digits a, digits b, digits c, digits e, digits f
) AS seq;

CREATE USER IF NOT EXISTS 'lt_reader'@'%' IDENTIFIED BY 'lt_reader';
GRANT SELECT, SHOW VIEW ON test_db.* TO 'lt_reader'@'%';
GRANT SELECT, SHOW VIEW ON lt_sales.* TO 'lt_reader'@'%';
GRANT SELECT, SHOW VIEW ON lt_inventory.* TO 'lt_reader'@'%';
GRANT SELECT ON lt_empty.* TO 'lt_reader'@'%';
FLUSH PRIVILEGES;
