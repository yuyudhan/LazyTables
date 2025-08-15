-- FilePath: docker/mysql/init/02-sample-data.sql

-- Insert sample data for MySQL testing
USE test_db;

-- Insert users
INSERT INTO users (username, email, full_name, is_active) VALUES
('john_doe', 'john@example.com', 'John Doe', TRUE),
('jane_smith', 'jane@example.com', 'Jane Smith', TRUE),
('bob_wilson', 'bob@example.com', 'Bob Wilson', TRUE),
('alice_jones', 'alice@example.com', 'Alice Jones', TRUE),
('charlie_brown', 'charlie@example.com', 'Charlie Brown', FALSE);

-- Insert categories
INSERT INTO categories (name, description, parent_id) VALUES
('Electronics', 'Electronic devices and accessories', NULL),
('Computers', 'Desktop and laptop computers', 1),
('Smartphones', 'Mobile phones and tablets', 1),
('Clothing', 'Apparel and fashion items', NULL),
('Men', 'Men''s clothing', 4),
('Women', 'Women''s clothing', 4),
('Books', 'Physical and digital books', NULL),
('Fiction', 'Fiction books', 7),
('Non-Fiction', 'Non-fiction books', 7);

-- Insert products
INSERT INTO products (name, description, price, stock_quantity, category_id) VALUES
('Laptop Pro 15', 'High-performance laptop with 15-inch display', 1299.99, 50, 2),
('Smartphone X', 'Latest smartphone with advanced features', 899.99, 100, 3),
('Wireless Mouse', 'Ergonomic wireless mouse', 29.99, 200, 1),
('USB-C Cable', '2-meter USB-C charging cable', 19.99, 500, 1),
('T-Shirt Basic', 'Cotton basic t-shirt', 19.99, 150, 5),
('Jeans Classic', 'Classic fit denim jeans', 59.99, 80, 5),
('Summer Dress', 'Light summer dress', 79.99, 60, 6),
('Wool Sweater', 'Warm wool sweater', 89.99, 40, 6),
('Programming Guide', 'Complete programming reference', 49.99, 30, 9),
('Mystery Novel', 'Bestselling mystery novel', 24.99, 100, 8);

-- Insert orders
INSERT INTO orders (user_id, status, total_amount, shipping_address) VALUES
(1, 'delivered', 1329.98, '123 Main St, New York, NY 10001'),
(2, 'shipped', 929.98, '456 Oak Ave, Los Angeles, CA 90001'),
(3, 'pending', 109.97, '789 Pine St, Chicago, IL 60601'),
(1, 'processing', 89.99, '123 Main St, New York, NY 10001'),
(4, 'delivered', 49.99, '321 Elm St, Houston, TX 77001');

-- Insert order items
INSERT INTO order_items (order_id, product_id, quantity, unit_price) VALUES
(1, 1, 1, 1299.99),
(1, 3, 1, 29.99),
(2, 2, 1, 899.99),
(2, 3, 1, 29.99),
(3, 5, 2, 19.99),
(3, 4, 1, 19.99),
(3, 9, 1, 49.99),
(4, 8, 1, 89.99),
(5, 9, 1, 49.99);

-- Insert inventory log entries
INSERT INTO inventory_log (product_id, change_type, quantity_change, new_quantity, notes) VALUES
(1, 'purchase', 100, 150, 'Initial stock purchase'),
(1, 'sale', -100, 50, 'Black Friday sales'),
(2, 'purchase', 200, 300, 'Restocking for holiday season'),
(2, 'sale', -200, 100, 'Holiday season sales'),
(3, 'adjustment', -5, 195, 'Damaged items removed'),
(4, 'purchase', 1000, 1500, 'Bulk purchase'),
(4, 'sale', -1000, 500, 'Promotional campaign');

-- Create some additional test tables with various data types
CREATE TABLE IF NOT EXISTS data_types_test (
    id INT AUTO_INCREMENT PRIMARY KEY,
    tiny_int_col TINYINT,
    small_int_col SMALLINT,
    medium_int_col MEDIUMINT,
    big_int_col BIGINT,
    decimal_col DECIMAL(10, 2),
    float_col FLOAT,
    double_col DOUBLE,
    bit_col BIT(8),
    date_col DATE,
    time_col TIME,
    datetime_col DATETIME,
    timestamp_col TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    year_col YEAR,
    char_col CHAR(10),
    varchar_col VARCHAR(255),
    text_col TEXT,
    json_col JSON,
    blob_col BLOB,
    enum_col ENUM('small', 'medium', 'large'),
    set_col SET('read', 'write', 'execute')
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Insert sample data into data_types_test
INSERT INTO data_types_test (
    tiny_int_col, small_int_col, medium_int_col, big_int_col,
    decimal_col, float_col, double_col, bit_col,
    date_col, time_col, datetime_col, year_col,
    char_col, varchar_col, text_col, json_col,
    enum_col, set_col
) VALUES (
    127, 32767, 8388607, 9223372036854775807,
    12345.67, 123.456, 123456.789012, b'10101010',
    '2024-01-15', '14:30:00', '2024-01-15 14:30:00', 2024,
    'Fixed', 'Variable length string', 'Long text content here',
    '{"key": "value", "number": 42}',
    'medium', 'read,write'
);