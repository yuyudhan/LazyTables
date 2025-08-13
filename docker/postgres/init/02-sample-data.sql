-- FilePath: docker/postgres/init/02-sample-data.sql

-- LazyTables Test Database Sample Data
-- This script populates the test database with realistic sample data

\echo 'Inserting sample data...'

-- ========================================
-- CATEGORIES DATA
-- ========================================

INSERT INTO public.categories (id, name, parent_id, description) VALUES
(1, 'Electronics', NULL, 'Electronic devices and accessories'),
(2, 'Computers', 1, 'Desktop and laptop computers'),
(3, 'Smartphones', 1, 'Mobile phones and accessories'),
(4, 'Audio', 1, 'Headphones, speakers, and audio equipment'),
(5, 'Clothing', NULL, 'Apparel and fashion'),
(6, 'Men''s Clothing', 5, 'Clothing for men'),
(7, 'Women''s Clothing', 5, 'Clothing for women'),
(8, 'Books', NULL, 'Books and publications'),
(9, 'Fiction', 8, 'Fiction books'),
(10, 'Non-Fiction', 8, 'Non-fiction books')
ON CONFLICT (id) DO NOTHING;

-- Reset sequence
SELECT setval('public.categories_id_seq', 10, true);

\echo 'Inserted categories data'

-- ========================================
-- PRODUCTS DATA
-- ========================================

INSERT INTO public.products (name, description, price, category_id, sku, stock_quantity, metadata) VALUES
('MacBook Pro 16"', 'Apple MacBook Pro with M3 chip, 16GB RAM, 512GB SSD', 2499.00, 2, 'MBP-16-M3-512', 15, '{"brand": "Apple", "warranty": "1 year", "color": "Space Gray"}'),
('Dell XPS 13', 'Dell XPS 13 laptop with Intel i7, 16GB RAM, 1TB SSD', 1299.00, 2, 'DELL-XPS13-I7', 8, '{"brand": "Dell", "warranty": "2 years", "color": "Silver"}'),
('iPhone 15 Pro', 'Apple iPhone 15 Pro with 128GB storage', 999.00, 3, 'IPH-15-PRO-128', 25, '{"brand": "Apple", "warranty": "1 year", "color": "Natural Titanium"}'),
('Samsung Galaxy S24', 'Samsung Galaxy S24 with 256GB storage', 899.00, 3, 'SAM-S24-256', 20, '{"brand": "Samsung", "warranty": "1 year", "color": "Phantom Black"}'),
('Sony WH-1000XM5', 'Wireless noise-canceling headphones', 399.00, 4, 'SONY-WH1000XM5', 30, '{"brand": "Sony", "warranty": "2 years", "color": "Black"}'),
('AirPods Pro 2', 'Apple AirPods Pro with MagSafe charging case', 249.00, 4, 'APP-PRO-2-MAGSAFE', 40, '{"brand": "Apple", "warranty": "1 year", "color": "White"}'),
('Levi''s 501 Jeans', 'Classic straight-leg jeans', 69.00, 6, 'LEVI-501-32-34', 50, '{"brand": "Levi''s", "size": "32x34", "material": "100% Cotton"}'),
('Nike Air Max 90', 'Classic Nike sneakers', 120.00, 6, 'NIKE-AM90-10', 35, '{"brand": "Nike", "size": "10", "color": "White/Black"}'),
('Zara Floral Dress', 'Summer floral print dress', 59.00, 7, 'ZARA-FD-M', 25, '{"brand": "Zara", "size": "M", "material": "Polyester blend"}'),
('H&M Basic T-Shirt', 'Cotton basic t-shirt', 12.99, 7, 'HM-BASIC-TEE-L', 100, '{"brand": "H&M", "size": "L", "material": "100% Cotton"}'),
('The Great Gatsby', 'Classic American novel by F. Scott Fitzgerald', 14.99, 9, 'BOOK-GATSBY', 75, '{"author": "F. Scott Fitzgerald", "pages": 180, "publisher": "Scribner"}'),
('To Kill a Mockingbird', 'Novel by Harper Lee', 13.99, 9, 'BOOK-MOCKINGBIRD', 60, '{"author": "Harper Lee", "pages": 281, "publisher": "J.B. Lippincott & Co."}'),
('Sapiens', 'A Brief History of Humankind by Yuval Noah Harari', 16.99, 10, 'BOOK-SAPIENS', 45, '{"author": "Yuval Noah Harari", "pages": 443, "publisher": "Harper"}'),
('Atomic Habits', 'Self-help book by James Clear', 18.99, 10, 'BOOK-ATOMIC-HABITS', 55, '{"author": "James Clear", "pages": 320, "publisher": "Avery"}'),
('Gaming Mechanical Keyboard', 'RGB mechanical keyboard for gaming', 129.00, 2, 'MECH-KB-RGB', 20, '{"brand": "Corsair", "switch_type": "Cherry MX Red", "backlight": "RGB"}'
);

\echo 'Inserted products data'

-- ========================================
-- USERS DATA
-- ========================================

INSERT INTO public.users (username, email, first_name, last_name, is_active, profile_data) VALUES
('john_doe', 'john.doe@example.com', 'John', 'Doe', true, '{"age": 28, "city": "San Francisco", "interests": ["technology", "gaming"]}'),
('jane_smith', 'jane.smith@example.com', 'Jane', 'Smith', true, '{"age": 32, "city": "New York", "interests": ["reading", "travel"]}'),
('bob_wilson', 'bob.wilson@example.com', 'Bob', 'Wilson', true, '{"age": 45, "city": "Chicago", "interests": ["music", "cooking"]}'),
('alice_brown', 'alice.brown@example.com', 'Alice', 'Brown', true, '{"age": 26, "city": "Austin", "interests": ["fitness", "photography"]}'),
('charlie_davis', 'charlie.davis@example.com', 'Charlie', 'Davis', true, '{"age": 35, "city": "Seattle", "interests": ["hiking", "coffee"]}'),
('diana_lee', 'diana.lee@example.com', 'Diana', 'Lee', true, '{"age": 29, "city": "Los Angeles", "interests": ["art", "design"]}'),
('frank_miller', 'frank.miller@example.com', 'Frank', 'Miller', false, '{"age": 41, "city": "Boston", "interests": ["sports", "movies"]}'),
('grace_taylor', 'grace.taylor@example.com', 'Grace', 'Taylor', true, '{"age": 23, "city": "Portland", "interests": ["music", "food"]}'),
('henry_clark', 'henry.clark@example.com', 'Henry', 'Clark', true, '{"age": 38, "city": "Denver", "interests": ["outdoors", "tech"]}'),
('ivy_johnson', 'ivy.johnson@example.com', 'Ivy', 'Johnson', true, '{"age": 31, "city": "Miami", "interests": ["beach", "yoga"]}');

\echo 'Inserted users data'

-- ========================================
-- ORDERS DATA
-- ========================================

INSERT INTO public.orders (user_id, total_amount, status, shipping_address, billing_address) VALUES
(1, 2748.00, 'completed', '123 Main St, San Francisco, CA 94102', '123 Main St, San Francisco, CA 94102'),
(2, 82.98, 'shipped', '456 Oak Ave, New York, NY 10001', '456 Oak Ave, New York, NY 10001'),
(3, 648.00, 'completed', '789 Pine St, Chicago, IL 60601', '789 Pine St, Chicago, IL 60601'),
(4, 189.00, 'processing', '321 Elm St, Austin, TX 73301', '321 Elm St, Austin, TX 73301'),
(5, 1318.99, 'completed', '654 Cedar Ave, Seattle, WA 98101', '654 Cedar Ave, Seattle, WA 98101'),
(1, 249.00, 'shipped', '123 Main St, San Francisco, CA 94102', '123 Main St, San Francisco, CA 94102'),
(6, 59.00, 'completed', '987 Birch St, Los Angeles, CA 90001', '987 Birch St, Los Angeles, CA 90001'),
(8, 43.97, 'pending', '147 Maple Ave, Portland, OR 97201', '147 Maple Ave, Portland, OR 97201'),
(9, 999.00, 'completed', '258 Spruce St, Denver, CO 80201', '258 Spruce St, Denver, CO 80201'),
(10, 120.00, 'shipped', '369 Willow Ave, Miami, FL 33101', '369 Willow Ave, Miami, FL 33101');

\echo 'Inserted orders data'

-- ========================================
-- ORDER ITEMS DATA
-- ========================================

INSERT INTO public.order_items (order_id, product_id, quantity, unit_price) VALUES
-- Order 1 (john_doe): MacBook Pro + AirPods Pro
(1, 1, 1, 2499.00),
(1, 6, 1, 249.00),

-- Order 2 (jane_smith): Books
(2, 11, 1, 14.99),
(2, 12, 1, 13.99),
(2, 13, 1, 16.99),
(2, 14, 1, 18.99),
(2, 10, 1, 12.99),

-- Order 3 (bob_wilson): Sony headphones + Gaming keyboard
(3, 5, 1, 399.00),
(3, 15, 1, 129.00),
(3, 7, 1, 69.00),
(3, 8, 1, 120.00),

-- Order 4 (alice_brown): Clothing
(4, 9, 1, 59.00),
(4, 8, 1, 120.00),
(4, 10, 1, 12.99),

-- Order 5 (charlie_davis): Dell laptop + Books
(5, 2, 1, 1299.00),
(5, 14, 1, 18.99),

-- Order 6 (john_doe): AirPods Pro
(6, 6, 1, 249.00),

-- Order 7 (diana_lee): Zara dress
(7, 9, 1, 59.00),

-- Order 8 (grace_taylor): T-shirt + Books
(8, 10, 2, 12.99),
(8, 11, 1, 14.99),

-- Order 9 (henry_clark): iPhone
(9, 3, 1, 999.00),

-- Order 10 (ivy_johnson): Nike shoes
(10, 8, 1, 120.00);

\echo 'Inserted order items data'

-- ========================================
-- PAYMENTS DATA
-- ========================================

INSERT INTO public.payments (order_id, amount, payment_method, status, transaction_id, processed_at) VALUES
(1, 2748.00, 'credit_card', 'completed', 'TXN_001_20240101', '2024-01-01 10:30:00'),
(2, 82.98, 'paypal', 'completed', 'TXN_002_20240102', '2024-01-02 14:15:00'),
(3, 648.00, 'credit_card', 'completed', 'TXN_003_20240103', '2024-01-03 09:45:00'),
(4, 189.00, 'credit_card', 'pending', 'TXN_004_20240104', NULL),
(5, 1318.99, 'bank_transfer', 'completed', 'TXN_005_20240105', '2024-01-05 16:20:00'),
(6, 249.00, 'credit_card', 'completed', 'TXN_006_20240106', '2024-01-06 11:00:00'),
(7, 59.00, 'paypal', 'completed', 'TXN_007_20240107', '2024-01-07 13:30:00'),
(8, 43.97, 'credit_card', 'pending', 'TXN_008_20240108', NULL),
(9, 999.00, 'apple_pay', 'completed', 'TXN_009_20240109', '2024-01-09 15:45:00'),
(10, 120.00, 'credit_card', 'completed', 'TXN_010_20240110', '2024-01-10 12:15:00');

\echo 'Inserted payments data'

-- ========================================
-- ANALYTICS DATA
-- ========================================

-- Sessions
INSERT INTO analytics.sessions (id, user_id, started_at, ended_at, duration_seconds, page_views, ip_address, user_agent) VALUES
('sess_001', 1, '2024-01-01 10:00:00', '2024-01-01 10:45:00', 2700, 12, '192.168.1.100', 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)'),
('sess_002', 2, '2024-01-02 14:00:00', '2024-01-02 14:30:00', 1800, 8, '192.168.1.101', 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)'),
('sess_003', 3, '2024-01-03 09:30:00', '2024-01-03 10:15:00', 2700, 15, '192.168.1.102', 'Mozilla/5.0 (X11; Linux x86_64)'),
('sess_004', 4, '2024-01-04 16:00:00', '2024-01-04 16:20:00', 1200, 5, '192.168.1.103', 'Mozilla/5.0 (iPhone; CPU iPhone OS 17_0)'),
('sess_005', 5, '2024-01-05 11:00:00', '2024-01-05 11:40:00', 2400, 10, '192.168.1.104', 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)');

-- Events
INSERT INTO analytics.events (user_id, event_type, event_data, session_id, ip_address, user_agent) VALUES
(1, 'page_view', '{"page": "/", "referrer": "google.com"}', 'sess_001', '192.168.1.100', 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)'),
(1, 'product_view', '{"product_id": 1, "product_name": "MacBook Pro 16\""}', 'sess_001', '192.168.1.100', 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)'),
(1, 'add_to_cart', '{"product_id": 1, "quantity": 1, "price": 2499.00}', 'sess_001', '192.168.1.100', 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)'),
(1, 'purchase', '{"order_id": 1, "total": 2748.00, "items": 2}', 'sess_001', '192.168.1.100', 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)'),
(2, 'page_view', '{"page": "/books", "referrer": "direct"}', 'sess_002', '192.168.1.101', 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)'),
(2, 'search', '{"query": "fiction books", "results": 25}', 'sess_002', '192.168.1.101', 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)'),
(2, 'purchase', '{"order_id": 2, "total": 82.98, "items": 5}', 'sess_002', '192.168.1.101', 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)'),
(3, 'page_view', '{"page": "/electronics", "referrer": "facebook.com"}', 'sess_003', '192.168.1.102', 'Mozilla/5.0 (X11; Linux x86_64)'),
(3, 'product_view', '{"product_id": 5, "product_name": "Sony WH-1000XM5"}', 'sess_003', '192.168.1.102', 'Mozilla/5.0 (X11; Linux x86_64)'),
(4, 'page_view', '{"page": "/clothing", "referrer": "instagram.com"}', 'sess_004', '192.168.1.103', 'Mozilla/5.0 (iPhone; CPU iPhone OS 17_0)');

-- Daily metrics
INSERT INTO analytics.daily_metrics (date, total_users, active_users, new_users, total_orders, total_revenue, avg_order_value) VALUES
('2024-01-01', 1, 1, 1, 1, 2748.00, 2748.00),
('2024-01-02', 2, 1, 1, 1, 82.98, 82.98),
('2024-01-03', 3, 1, 1, 1, 648.00, 648.00),
('2024-01-04', 4, 1, 1, 1, 189.00, 189.00),
('2024-01-05', 5, 1, 1, 1, 1318.99, 1318.99),
('2024-01-06', 5, 1, 0, 1, 249.00, 249.00),
('2024-01-07', 6, 1, 1, 1, 59.00, 59.00),
('2024-01-08', 7, 1, 1, 1, 43.97, 43.97),
('2024-01-09', 8, 1, 1, 1, 999.00, 999.00),
('2024-01-10', 9, 1, 1, 1, 120.00, 120.00);

\echo 'Inserted analytics data'

-- ========================================
-- ADMIN DATA
-- ========================================

-- Admin users (password is 'admin123' hashed with bcrypt)
INSERT INTO admin.admin_users (username, email, password_hash, role, permissions) VALUES
('admin', 'admin@lazytables.dev', '$2a$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LeNormJWkmOZeKDyq', 'super_admin', '{"users": "all", "products": "all", "orders": "all", "analytics": "read"}'),
('manager', 'manager@lazytables.dev', '$2a$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LeNormJWkmOZeKDyq', 'manager', '{"users": "read", "products": "all", "orders": "read", "analytics": "read"}'),
('analyst', 'analyst@lazytables.dev', '$2a$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LeNormJWkmOZeKDyq', 'analyst', '{"analytics": "read", "orders": "read"}');

-- System configuration
INSERT INTO admin.system_config (key, value, description) VALUES
('app_name', 'LazyTables Test Store', 'Application display name'),
('maintenance_mode', 'false', 'Enable/disable maintenance mode'),
('max_order_items', '50', 'Maximum items per order'),
('default_currency', 'USD', 'Default currency for pricing'),
('email_notifications', 'true', 'Enable email notifications'),
('analytics_retention_days', '365', 'How long to keep analytics data');

-- Audit log entries
INSERT INTO admin.audit_log (admin_user_id, action, table_name, record_id, new_data, ip_address) VALUES
(1, 'CREATE_PRODUCT', 'products', 1, '{"name": "MacBook Pro 16\"", "price": 2499.00}', '192.168.1.200'),
(1, 'UPDATE_PRODUCT', 'products', 1, '{"stock_quantity": 15}', '192.168.1.200'),
(2, 'CREATE_CATEGORY', 'categories', 1, '{"name": "Electronics"}', '192.168.1.201'),
(1, 'UPDATE_CONFIG', 'system_config', NULL, '{"key": "maintenance_mode", "value": "false"}', '192.168.1.200');

\echo 'Inserted admin data'

-- ========================================
-- UPDATE STATISTICS
-- ========================================

-- Update table statistics for better query planning
ANALYZE public.users;
ANALYZE public.products;
ANALYZE public.categories;
ANALYZE public.orders;
ANALYZE public.order_items;
ANALYZE public.payments;
ANALYZE analytics.events;
ANALYZE analytics.sessions;
ANALYZE analytics.daily_metrics;
ANALYZE admin.admin_users;
ANALYZE admin.audit_log;
ANALYZE admin.system_config;

\echo 'Updated table statistics'

\echo 'Sample data insertion completed successfully!'
\echo 'Database contains:'
\echo '  - 10 categories across multiple levels'
\echo '  - 15 products with rich metadata'
\echo '  - 10 users with profile data'
\echo '  - 10 orders with various statuses'
\echo '  - Corresponding order items and payments'
\echo '  - Analytics events and sessions'
\echo '  - Admin users and audit logs'
\echo '  - System configuration'