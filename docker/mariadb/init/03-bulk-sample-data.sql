-- FilePath: docker/mysql/init/03-bulk-sample-data.sql

-- Insert bulk sample data for MySQL testing
USE test_db;

-- Insert more users (50 total)
INSERT INTO users (username, email, full_name, is_active) VALUES
('user_6', 'user6@example.com', 'User Six', TRUE),
('user_7', 'user7@example.com', 'User Seven', TRUE),
('user_8', 'user8@example.com', 'User Eight', TRUE),
('user_9', 'user9@example.com', 'User Nine', FALSE),
('user_10', 'user10@example.com', 'User Ten', TRUE),
('michael_scott', 'michael@dundermifflin.com', 'Michael Scott', TRUE),
('dwight_schrute', 'dwight@dundermifflin.com', 'Dwight Schrute', TRUE),
('jim_halpert', 'jim@dundermifflin.com', 'Jim Halpert', TRUE),
('pam_beesly', 'pam@dundermifflin.com', 'Pam Beesly', TRUE),
('stanley_hudson', 'stanley@dundermifflin.com', 'Stanley Hudson', TRUE),
('angela_martin', 'angela@dundermifflin.com', 'Angela Martin', TRUE),
('kevin_malone', 'kevin@dundermifflin.com', 'Kevin Malone', TRUE),
('oscar_martinez', 'oscar@dundermifflin.com', 'Oscar Martinez', TRUE),
('phyllis_vance', 'phyllis@dundermifflin.com', 'Phyllis Vance', TRUE),
('andy_bernard', 'andy@dundermifflin.com', 'Andy Bernard', FALSE),
('toby_flenderson', 'toby@dundermifflin.com', 'Toby Flenderson', TRUE),
('creed_bratton', 'creed@dundermifflin.com', 'Creed Bratton', TRUE),
('meredith_palmer', 'meredith@dundermifflin.com', 'Meredith Palmer', TRUE),
('ryan_howard', 'ryan@dundermifflin.com', 'Ryan Howard', FALSE),
('kelly_kapoor', 'kelly@dundermifflin.com', 'Kelly Kapoor', TRUE);

-- Insert more categories
INSERT INTO categories (name, description, parent_id) VALUES
('Home & Garden', 'Home improvement and gardening', NULL),
('Furniture', 'Home and office furniture', 21),
('Tools', 'Hand and power tools', 21),
('Sports & Outdoors', 'Sporting goods and outdoor equipment', NULL),
('Fitness', 'Exercise and fitness equipment', 24),
('Camping', 'Camping and hiking gear', 24),
('Toys & Games', 'Children toys and games', NULL),
('Board Games', 'Family board games', 27),
('Video Games', 'Console and PC games', 27);

-- Insert more products (100+ products)
INSERT INTO products (name, description, price, stock_quantity, category_id) VALUES
('Office Chair', 'Ergonomic office chair with lumbar support', 299.99, 25, 22),
('Standing Desk', 'Adjustable height standing desk', 599.99, 15, 22),
('Bookshelf', '5-tier wooden bookshelf', 149.99, 30, 22),
('Desk Lamp', 'LED desk lamp with adjustable brightness', 39.99, 100, 1),
('Power Drill', 'Cordless 20V power drill', 89.99, 45, 23),
('Hammer', 'Claw hammer 16oz', 19.99, 200, 23),
('Screwdriver Set', '20-piece screwdriver set', 29.99, 150, 23),
('Yoga Mat', 'Non-slip exercise yoga mat', 24.99, 80, 25),
('Dumbbell Set', 'Adjustable dumbbell set 5-50lbs', 299.99, 20, 25),
('Resistance Bands', 'Set of 5 resistance bands', 19.99, 100, 25),
('Tent 2-Person', 'Waterproof camping tent for 2', 129.99, 35, 26),
('Sleeping Bag', 'All-season sleeping bag', 59.99, 50, 26),
('Camping Stove', 'Portable gas camping stove', 44.99, 40, 26),
('Backpack', 'Hiking backpack 50L', 89.99, 60, 26),
('Chess Set', 'Wooden chess set with board', 49.99, 30, 28),
('Monopoly', 'Classic Monopoly board game', 29.99, 50, 28),
('Scrabble', 'Original Scrabble word game', 24.99, 40, 28),
('PlayStation 5', 'Gaming console PlayStation 5', 499.99, 10, 29),
('Xbox Series X', 'Gaming console Xbox Series X', 499.99, 8, 29),
('Nintendo Switch', 'Nintendo Switch console', 299.99, 20, 29),
('Gaming Headset', 'Wireless gaming headset', 79.99, 100, 29),
('Gaming Mouse', 'RGB gaming mouse', 49.99, 150, 1),
('Mechanical Keyboard', 'RGB mechanical keyboard', 89.99, 80, 1),
('Monitor 27"', '27-inch 4K monitor', 399.99, 40, 1),
('Webcam HD', '1080p HD webcam', 59.99, 120, 1),
('Microphone', 'USB condenser microphone', 69.99, 90, 1),
('Router WiFi 6', 'High-speed WiFi 6 router', 149.99, 70, 1),
('Smart Watch', 'Fitness tracking smart watch', 199.99, 85, 1),
('Tablet 10"', '10-inch Android tablet', 249.99, 55, 1),
('E-Reader', 'E-ink display e-reader', 129.99, 65, 1),
('Bluetooth Speaker', 'Portable Bluetooth speaker', 39.99, 200, 1),
('Headphones', 'Noise-canceling headphones', 149.99, 75, 1),
('Phone Case', 'Universal phone case', 14.99, 500, 1),
('Screen Protector', 'Tempered glass screen protector', 9.99, 1000, 1),
('Charging Cable', '10ft USB-C charging cable', 12.99, 800, 1),
('Power Bank', '20000mAh power bank', 29.99, 300, 1),
('Car Charger', 'Dual USB car charger', 15.99, 400, 1),
('Wireless Charger', 'Fast wireless charging pad', 24.99, 250, 1),
('HDMI Cable', '6ft HDMI 2.1 cable', 19.99, 600, 1),
('Ethernet Cable', '50ft Cat6 ethernet cable', 24.99, 350, 1);

-- Generate more orders with different statuses
INSERT INTO orders (user_id, status, total_amount, shipping_address) VALUES
(6, 'delivered', 449.98, '100 First St, Boston, MA 02101'),
(7, 'delivered', 329.97, '200 Second Ave, Seattle, WA 98101'),
(8, 'shipped', 189.99, '300 Third St, Denver, CO 80201'),
(9, 'processing', 599.99, '400 Fourth Ave, Austin, TX 78701'),
(10, 'pending', 79.98, '500 Fifth St, Miami, FL 33101'),
(11, 'delivered', 1299.97, '1 Office Park, Scranton, PA 18503'),
(12, 'shipped', 89.99, '2 Beet Farm Rd, Scranton, PA 18504'),
(13, 'delivered', 249.99, '3 Paper St, Scranton, PA 18505'),
(14, 'cancelled', 199.99, '4 Art Ave, Scranton, PA 18506'),
(15, 'delivered', 459.97, '5 Cat Lane, Scranton, PA 18507'),
(16, 'processing', 149.99, '6 Accounting Blvd, Scranton, PA 18508'),
(17, 'delivered', 39.99, '7 Chili Ave, Scranton, PA 18509'),
(18, 'shipped', 299.99, '8 Finance St, Scranton, PA 18510'),
(19, 'delivered', 179.97, '9 Sales Dr, Scranton, PA 18511'),
(20, 'pending', 89.99, '10 Cornell Way, Scranton, PA 18512'),
(1, 'delivered', 529.97, '123 Main St, New York, NY 10001'),
(2, 'processing', 299.99, '456 Oak Ave, Los Angeles, CA 90001'),
(3, 'shipped', 149.99, '789 Pine St, Chicago, IL 60601'),
(4, 'delivered', 899.98, '321 Elm St, Houston, TX 77001'),
(5, 'cancelled', 49.99, '654 Maple Dr, Phoenix, AZ 85001');

-- Add corresponding order items
INSERT INTO order_items (order_id, product_id, quantity, unit_price) VALUES
(6, 11, 1, 299.99),
(6, 12, 1, 149.99),
(7, 13, 1, 39.99),
(7, 14, 1, 89.99),
(7, 15, 1, 19.99),
(7, 16, 1, 29.99),
(7, 17, 5, 24.99),
(8, 18, 1, 299.99),
(9, 12, 1, 599.99),
(10, 19, 2, 19.99),
(10, 13, 1, 39.99),
(11, 1, 1, 1299.99),
(12, 14, 1, 89.99),
(13, 30, 1, 249.99),
(14, 31, 1, 199.99),
(15, 11, 1, 299.99),
(15, 20, 1, 59.99),
(15, 21, 2, 44.99),
(16, 22, 1, 89.99),
(16, 21, 1, 44.99),
(17, 36, 1, 39.99),
(18, 18, 1, 299.99),
(19, 23, 1, 49.99),
(19, 24, 1, 89.99),
(19, 13, 1, 39.99),
(20, 25, 1, 399.99),
(21, 11, 1, 299.99),
(21, 22, 1, 89.99),
(21, 28, 1, 79.99),
(21, 26, 1, 59.99),
(22, 18, 1, 299.99),
(23, 33, 1, 149.99),
(24, 1, 1, 1299.99),
(24, 3, 2, 29.99),
(24, 38, 2, 14.99),
(25, 27, 1, 69.99);

-- Create a customers_view for easy reporting
CREATE OR REPLACE VIEW customer_activity AS
SELECT 
    u.id,
    u.username,
    u.email,
    u.full_name,
    COUNT(DISTINCT o.id) as total_orders,
    COALESCE(SUM(o.total_amount), 0) as total_spent,
    MAX(o.order_date) as last_order_date,
    u.is_active
FROM users u
LEFT JOIN orders o ON u.id = o.user_id
GROUP BY u.id, u.username, u.email, u.full_name, u.is_active;

-- Create a product_inventory view
CREATE OR REPLACE VIEW product_inventory AS
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
JOIN categories c ON p.category_id = c.id
ORDER BY p.stock_quantity ASC;

-- Create an audit_log table for tracking changes
CREATE TABLE IF NOT EXISTS audit_log (
    id INT AUTO_INCREMENT PRIMARY KEY,
    table_name VARCHAR(50) NOT NULL,
    operation ENUM('INSERT', 'UPDATE', 'DELETE') NOT NULL,
    user_id INT,
    record_id INT,
    old_values JSON,
    new_values JSON,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_table_operation (table_name, operation),
    INDEX idx_created (created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Insert some audit records
INSERT INTO audit_log (table_name, operation, user_id, record_id, old_values, new_values) VALUES
('orders', 'INSERT', 1, 1, NULL, '{"status": "pending", "total_amount": 1329.98}'),
('orders', 'UPDATE', 1, 1, '{"status": "pending"}', '{"status": "processing"}'),
('orders', 'UPDATE', 1, 1, '{"status": "processing"}', '{"status": "shipped"}'),
('orders', 'UPDATE', 1, 1, '{"status": "shipped"}', '{"status": "delivered"}'),
('products', 'UPDATE', NULL, 1, '{"stock_quantity": 150}', '{"stock_quantity": 50}'),
('users', 'UPDATE', 15, 15, '{"is_active": true}', '{"is_active": false}'),
('products', 'INSERT', NULL, 40, NULL, '{"name": "New Product", "price": 99.99}');

-- Create a settings table for application configuration
CREATE TABLE IF NOT EXISTS settings (
    id INT AUTO_INCREMENT PRIMARY KEY,
    setting_key VARCHAR(100) UNIQUE NOT NULL,
    setting_value TEXT,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Insert settings
INSERT INTO settings (setting_key, setting_value, description) VALUES
('app_name', 'LazyTables Test Database', 'Application name'),
('version', '1.0.0', 'Current version'),
('maintenance_mode', 'false', 'Maintenance mode flag'),
('items_per_page', '20', 'Default pagination size'),
('max_export_rows', '10000', 'Maximum rows for data export'),
('enable_audit_log', 'true', 'Enable audit logging'),
('default_theme', 'dark', 'Default UI theme');