-- FilePath: docker/sqlite/init.sql

-- SQLite initialization script
-- This can be run with: sqlite3 test.db < docker/sqlite/init.sql

-- Enable foreign keys
PRAGMA foreign_keys = ON;

-- Create users table
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    full_name TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT 1
);

-- Create categories table
CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    parent_id INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (parent_id) REFERENCES categories(id) ON DELETE SET NULL
);

-- Create products table
CREATE TABLE IF NOT EXISTS products (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    price DECIMAL(10, 2) NOT NULL,
    stock_quantity INTEGER DEFAULT 0,
    category_id INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL
);

-- Create orders table
CREATE TABLE IF NOT EXISTS orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    order_date DATETIME DEFAULT CURRENT_TIMESTAMP,
    status TEXT CHECK(status IN ('pending', 'processing', 'shipped', 'delivered', 'cancelled')) DEFAULT 'pending',
    total_amount DECIMAL(10, 2) NOT NULL,
    shipping_address TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create order_items table
CREATE TABLE IF NOT EXISTS order_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id INTEGER NOT NULL,
    product_id INTEGER NOT NULL,
    quantity INTEGER NOT NULL,
    unit_price DECIMAL(10, 2) NOT NULL,
    FOREIGN KEY (order_id) REFERENCES orders(id) ON DELETE CASCADE,
    FOREIGN KEY (product_id) REFERENCES products(id)
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_categories_parent ON categories(parent_id);
CREATE INDEX IF NOT EXISTS idx_products_name ON products(name);
CREATE INDEX IF NOT EXISTS idx_products_category ON products(category_id);
CREATE INDEX IF NOT EXISTS idx_orders_user ON orders(user_id);
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status);
CREATE INDEX IF NOT EXISTS idx_order_items_order ON order_items(order_id);
CREATE INDEX IF NOT EXISTS idx_order_items_product ON order_items(product_id);

-- Insert sample users
INSERT INTO users (username, email, full_name, is_active) VALUES
('john_doe', 'john@example.com', 'John Doe', 1),
('jane_smith', 'jane@example.com', 'Jane Smith', 1),
('bob_wilson', 'bob@example.com', 'Bob Wilson', 1),
('alice_jones', 'alice@example.com', 'Alice Jones', 1),
('charlie_brown', 'charlie@example.com', 'Charlie Brown', 0),
('david_lee', 'david@example.com', 'David Lee', 1),
('emma_davis', 'emma@example.com', 'Emma Davis', 1),
('frank_miller', 'frank@example.com', 'Frank Miller', 1),
('grace_taylor', 'grace@example.com', 'Grace Taylor', 1),
('henry_wilson', 'henry@example.com', 'Henry Wilson', 0);

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
('Non-Fiction', 'Non-fiction books', 7),
('Home & Garden', 'Home improvement and gardening', NULL),
('Furniture', 'Home and office furniture', 10),
('Tools', 'Hand and power tools', 10);

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
('Mystery Novel', 'Bestselling mystery novel', 24.99, 100, 8),
('Office Chair', 'Ergonomic office chair', 299.99, 25, 11),
('Standing Desk', 'Adjustable height desk', 599.99, 15, 11),
('Power Drill', 'Cordless 20V power drill', 89.99, 45, 12),
('Hammer', 'Claw hammer 16oz', 19.99, 200, 12),
('Tablet 10"', '10-inch Android tablet', 249.99, 55, 3);

-- Insert orders
INSERT INTO orders (user_id, status, total_amount, shipping_address) VALUES
(1, 'delivered', 1329.98, '123 Main St, New York, NY 10001'),
(2, 'shipped', 929.98, '456 Oak Ave, Los Angeles, CA 90001'),
(3, 'pending', 109.97, '789 Pine St, Chicago, IL 60601'),
(1, 'processing', 89.99, '123 Main St, New York, NY 10001'),
(4, 'delivered', 49.99, '321 Elm St, Houston, TX 77001'),
(5, 'cancelled', 299.99, '654 Cedar Rd, Miami, FL 33101'),
(6, 'delivered', 449.98, '987 Birch Ln, Seattle, WA 98101'),
(7, 'shipped', 189.99, '246 Spruce Ave, Denver, CO 80201'),
(8, 'pending', 599.99, '135 Ash Dr, Boston, MA 02101'),
(9, 'delivered', 79.98, '864 Willow Way, Austin, TX 78701');

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
(5, 9, 1, 49.99),
(6, 11, 1, 299.99),
(6, 5, 1, 19.99),
(6, 10, 5, 24.99),
(7, 13, 1, 89.99),
(7, 14, 1, 19.99),
(7, 6, 1, 59.99),
(8, 12, 1, 599.99),
(9, 7, 1, 79.99),
(10, 4, 2, 19.99),
(10, 3, 1, 29.99);

-- Create views
CREATE VIEW IF NOT EXISTS order_summaries AS
SELECT 
    o.id AS order_id,
    u.username,
    u.email,
    o.order_date,
    o.status,
    o.total_amount,
    COUNT(oi.id) AS item_count
FROM orders o
JOIN users u ON o.user_id = u.id
LEFT JOIN order_items oi ON o.id = oi.order_id
GROUP BY o.id, u.username, u.email, o.order_date, o.status, o.total_amount;

CREATE VIEW IF NOT EXISTS product_inventory AS
SELECT 
    p.id,
    p.name,
    c.name as category,
    p.price,
    p.stock_quantity,
    p.stock_quantity * p.price as inventory_value,
    CASE 
        WHEN p.stock_quantity = 0 THEN 'Out of Stock'
        WHEN p.stock_quantity < 10 THEN 'Low Stock'
        ELSE 'In Stock'
    END as stock_status
FROM products p
LEFT JOIN categories c ON p.category_id = c.id
ORDER BY p.stock_quantity ASC;

-- Create a settings table
CREATE TABLE IF NOT EXISTS settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    setting_key TEXT UNIQUE NOT NULL,
    setting_value TEXT,
    description TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Insert settings
INSERT INTO settings (setting_key, setting_value, description) VALUES
('app_name', 'LazyTables SQLite Test', 'Application name'),
('version', '1.0.0', 'Current version'),
('maintenance_mode', 'false', 'Maintenance mode flag'),
('items_per_page', '20', 'Default pagination size'),
('max_export_rows', '10000', 'Maximum rows for data export');

-- Create a data_types_test table to showcase SQLite types
CREATE TABLE IF NOT EXISTS data_types_test (
    id INTEGER PRIMARY KEY,
    int_col INTEGER,
    real_col REAL,
    text_col TEXT,
    blob_col BLOB,
    numeric_col NUMERIC,
    boolean_col BOOLEAN,
    date_col DATE,
    datetime_col DATETIME,
    time_col TIME
);

-- Insert test data
INSERT INTO data_types_test VALUES
(1, 42, 3.14159, 'Hello SQLite', X'48656C6C6F', 123.45, 1, '2024-01-15', '2024-01-15 14:30:00', '14:30:00'),
(2, -100, 2.71828, 'Testing types', NULL, 999.99, 0, '2024-12-31', '2024-12-31 23:59:59', '23:59:59'),
(3, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);