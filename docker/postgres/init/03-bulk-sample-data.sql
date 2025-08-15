-- FilePath: docker/postgres/init/03-bulk-sample-data.sql

-- LazyTables Bulk Test Data Generation
-- This script generates hundreds of additional records for testing

\echo 'Generating bulk sample data...'

-- ========================================
-- GENERATE MORE USERS (190 additional for total of 200)
-- ========================================

INSERT INTO public.users (username, email, first_name, last_name, is_active, profile_data)
SELECT 
    'user_' || generate_series || '_auto',
    'user' || generate_series || '@testdata.com',
    (ARRAY['Emma', 'William', 'Olivia', 'James', 'Ava', 'Robert', 'Isabella', 'Michael', 'Sophia', 'David', 'Charlotte', 'Joseph', 'Mia', 'Thomas', 'Amelia'])[1 + floor(random() * 15)::int],
    (ARRAY['Johnson', 'Williams', 'Brown', 'Jones', 'Garcia', 'Miller', 'Davis', 'Rodriguez', 'Martinez', 'Hernandez', 'Lopez', 'Gonzalez', 'Wilson', 'Anderson', 'Thomas'])[1 + floor(random() * 15)::int],
    (random() > 0.1)::boolean,
    jsonb_build_object(
        'age', 18 + floor(random() * 50)::int,
        'city', (ARRAY['New York', 'Los Angeles', 'Chicago', 'Houston', 'Phoenix', 'Philadelphia', 'San Antonio', 'San Diego', 'Dallas', 'San Jose', 'Austin', 'Jacksonville', 'Fort Worth', 'Columbus', 'Charlotte'])[1 + floor(random() * 15)::int],
        'interests', ARRAY[(ARRAY['technology', 'gaming', 'reading', 'travel', 'music', 'cooking', 'fitness', 'photography', 'hiking', 'coffee', 'art', 'design', 'sports', 'movies', 'food'])[1 + floor(random() * 15)::int], 
                          (ARRAY['technology', 'gaming', 'reading', 'travel', 'music', 'cooking', 'fitness', 'photography', 'hiking', 'coffee', 'art', 'design', 'sports', 'movies', 'food'])[1 + floor(random() * 15)::int]]
    )
FROM generate_series(11, 200);

\echo 'Generated 190 additional users (total: 200)'

-- ========================================
-- GENERATE MORE CATEGORIES (40 additional for total of 50)
-- ========================================

INSERT INTO public.categories (name, parent_id, description)
SELECT 
    'Category_' || generate_series,
    CASE 
        WHEN random() > 0.4 THEN floor(random() * 10 + 1)::int
        ELSE NULL
    END,
    'Auto-generated category for testing purposes - ' || generate_series
FROM generate_series(11, 50);

\echo 'Generated 40 additional categories (total: 50)'

-- ========================================
-- GENERATE MORE PRODUCTS (285 additional for total of 300)
-- ========================================

INSERT INTO public.products (name, description, price, category_id, sku, stock_quantity, metadata)
SELECT 
    (ARRAY['Ultra', 'Pro', 'Elite', 'Premium', 'Standard', 'Basic', 'Advanced', 'Digital', 'Smart', 'Eco'])[1 + floor(random() * 10)::int] || ' ' ||
    (ARRAY['Widget', 'Gadget', 'Device', 'Tool', 'System', 'Module', 'Component', 'Kit', 'Pack', 'Set'])[1 + floor(random() * 10)::int] || ' ' ||
    generate_series,
    'High-quality product with advanced features. Model #' || generate_series || ' - ' ||
    (ARRAY['Perfect for professionals', 'Ideal for home use', 'Great for beginners', 'Enterprise-grade solution', 'Budget-friendly option', 'Top-rated by experts', 'Award-winning design', 'Industry standard', 'Revolutionary technology', 'Next-generation performance'])[1 + floor(random() * 10)::int],
    (random() * 2000 + 10)::numeric(10,2),
    floor(random() * 50 + 1)::int,
    'SKU-' || LPAD(generate_series::text, 6, '0'),
    floor(random() * 500)::int,
    jsonb_build_object(
        'brand', (ARRAY['TechCorp', 'GlobalTech', 'InnovateCo', 'FutureTech', 'SmartSystems', 'EliteProducts', 'ProManufacturing', 'QualityGoods', 'PremiumBrands', 'TopChoice'])[1 + floor(random() * 10)::int],
        'warranty', (ARRAY['6 months', '1 year', '2 years', '3 years', '5 years', 'Lifetime'])[1 + floor(random() * 6)::int],
        'weight', (random() * 50)::numeric(5,2) || ' kg',
        'dimensions', jsonb_build_object(
            'length', floor(random() * 100 + 10)::int,
            'width', floor(random() * 100 + 10)::int,
            'height', floor(random() * 50 + 5)::int
        ),
        'rating', (random() * 2 + 3)::numeric(2,1),
        'reviews', floor(random() * 1000)::int
    )
FROM generate_series(16, 300);

\echo 'Generated 285 additional products (total: 300)'

-- ========================================
-- GENERATE MORE ORDERS (490 additional for total of 500)
-- ========================================

INSERT INTO public.orders (user_id, total_amount, status, shipping_address, billing_address, created_at, updated_at)
SELECT 
    floor(random() * 200 + 1)::int,
    (random() * 3000 + 50)::numeric(10,2),
    (ARRAY['pending', 'processing', 'shipped', 'completed', 'cancelled', 'refunded'])[1 + floor(random() * 6)::int],
    floor(random() * 999 + 1)::text || ' ' ||
    (ARRAY['Main', 'Oak', 'Pine', 'Elm', 'Maple', 'Cedar', 'Birch', 'Willow', 'Cherry', 'Walnut'])[1 + floor(random() * 10)::int] || ' ' ||
    (ARRAY['St', 'Ave', 'Rd', 'Blvd', 'Ln', 'Dr', 'Ct', 'Pl', 'Way', 'Pkwy'])[1 + floor(random() * 10)::int] || ', ' ||
    (ARRAY['New York, NY 10001', 'Los Angeles, CA 90001', 'Chicago, IL 60601', 'Houston, TX 77001', 'Phoenix, AZ 85001', 'Philadelphia, PA 19101', 'San Antonio, TX 78201', 'San Diego, CA 92101', 'Dallas, TX 75201', 'San Jose, CA 95101'])[1 + floor(random() * 10)::int],
    floor(random() * 999 + 1)::text || ' ' ||
    (ARRAY['Main', 'Oak', 'Pine', 'Elm', 'Maple', 'Cedar', 'Birch', 'Willow', 'Cherry', 'Walnut'])[1 + floor(random() * 10)::int] || ' ' ||
    (ARRAY['St', 'Ave', 'Rd', 'Blvd', 'Ln', 'Dr', 'Ct', 'Pl', 'Way', 'Pkwy'])[1 + floor(random() * 10)::int] || ', ' ||
    (ARRAY['New York, NY 10001', 'Los Angeles, CA 90001', 'Chicago, IL 60601', 'Houston, TX 77001', 'Phoenix, AZ 85001', 'Philadelphia, PA 19101', 'San Antonio, TX 78201', 'San Diego, CA 92101', 'Dallas, TX 75201', 'San Jose, CA 95101'])[1 + floor(random() * 10)::int],
    CURRENT_TIMESTAMP - (random() * interval '365 days'),
    CURRENT_TIMESTAMP - (random() * interval '30 days')
FROM generate_series(11, 500);

\echo 'Generated 490 additional orders (total: 500)'

-- ========================================
-- GENERATE ORDER ITEMS (1500+ items)
-- ========================================

INSERT INTO public.order_items (order_id, product_id, quantity, unit_price)
SELECT 
    order_id,
    floor(random() * 300 + 1)::int,
    floor(random() * 5 + 1)::int,
    (random() * 500 + 10)::numeric(10,2)
FROM (
    SELECT 
        floor(random() * 500 + 1)::int as order_id,
        generate_series as item_num
    FROM generate_series(1, 1500)
) as items
ON CONFLICT DO NOTHING;

\echo 'Generated ~1500 order items'

-- ========================================
-- GENERATE PAYMENTS (490 additional for 500 total)
-- ========================================

INSERT INTO public.payments (order_id, amount, payment_method, status, transaction_id, processed_at, created_at)
SELECT 
    generate_series,
    o.total_amount,
    (ARRAY['credit_card', 'debit_card', 'paypal', 'bank_transfer', 'apple_pay', 'google_pay', 'stripe', 'square', 'bitcoin', 'wire_transfer'])[1 + floor(random() * 10)::int],
    CASE 
        WHEN o.status IN ('completed', 'shipped') THEN 'completed'
        WHEN o.status = 'cancelled' THEN 'cancelled'
        WHEN o.status = 'refunded' THEN 'refunded'
        ELSE 'pending'
    END,
    'TXN_' || LPAD(generate_series::text, 6, '0') || '_' || to_char(CURRENT_DATE, 'YYYYMMDD'),
    CASE 
        WHEN o.status IN ('completed', 'shipped') THEN o.created_at + interval '1 hour'
        ELSE NULL
    END,
    o.created_at
FROM generate_series(11, 500)
JOIN public.orders o ON o.id = generate_series;

\echo 'Generated 490 additional payments (total: 500)'

-- ========================================
-- GENERATE ANALYTICS EVENTS (5000+ events)
-- ========================================

INSERT INTO analytics.events (user_id, event_type, event_data, session_id, ip_address, user_agent, created_at)
SELECT 
    floor(random() * 200 + 1)::int,
    (ARRAY['page_view', 'product_view', 'add_to_cart', 'remove_from_cart', 'purchase', 'search', 'login', 'logout', 'signup', 'profile_update', 'wishlist_add', 'review_submit', 'share', 'download', 'video_play'])[1 + floor(random() * 15)::int],
    jsonb_build_object(
        'page', '/' || (ARRAY['home', 'products', 'categories', 'cart', 'checkout', 'profile', 'orders', 'search', 'help', 'about'])[1 + floor(random() * 10)::int],
        'referrer', (ARRAY['google.com', 'facebook.com', 'twitter.com', 'instagram.com', 'direct', 'email', 'reddit.com', 'linkedin.com', 'youtube.com', 'bing.com'])[1 + floor(random() * 10)::int],
        'duration_seconds', floor(random() * 300)::int,
        'clicked_elements', floor(random() * 20)::int
    ),
    'sess_' || LPAD(floor(random() * 1000 + 1)::text, 6, '0'),
    ('192.168.' || floor(random() * 255)::int || '.' || floor(random() * 255)::int)::inet,
    (ARRAY['Mozilla/5.0 (Windows NT 10.0; Win64; x64)', 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)', 'Mozilla/5.0 (X11; Linux x86_64)', 'Mozilla/5.0 (iPhone; CPU iPhone OS 17_0)', 'Mozilla/5.0 (Android 13; Mobile)', 'Mozilla/5.0 (iPad; CPU OS 17_0)'])[1 + floor(random() * 6)::int],
    CURRENT_TIMESTAMP - (random() * interval '365 days')
FROM generate_series(1, 5000);

\echo 'Generated 5000 analytics events'

-- ========================================
-- GENERATE MORE SESSIONS (495 additional for 500 total)
-- ========================================

INSERT INTO analytics.sessions (id, user_id, started_at, ended_at, duration_seconds, page_views, ip_address, user_agent)
SELECT 
    'sess_' || LPAD(generate_series::text, 6, '0'),
    floor(random() * 200 + 1)::int,
    CURRENT_TIMESTAMP - (random() * interval '365 days'),
    CURRENT_TIMESTAMP - (random() * interval '365 days') + (random() * interval '2 hours'),
    floor(random() * 7200)::int,
    floor(random() * 50 + 1)::int,
    ('192.168.' || floor(random() * 255)::int || '.' || floor(random() * 255)::int)::inet,
    (ARRAY['Mozilla/5.0 (Windows NT 10.0; Win64; x64)', 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)', 'Mozilla/5.0 (X11; Linux x86_64)', 'Mozilla/5.0 (iPhone; CPU iPhone OS 17_0)', 'Mozilla/5.0 (Android 13; Mobile)', 'Mozilla/5.0 (iPad; CPU OS 17_0)'])[1 + floor(random() * 6)::int]
FROM generate_series(6, 500);

\echo 'Generated 495 additional sessions (total: 500)'

-- ========================================
-- GENERATE MORE DAILY METRICS (355 additional for 365 total - full year)
-- ========================================

INSERT INTO analytics.daily_metrics (date, total_users, active_users, new_users, total_orders, total_revenue, avg_order_value, created_at)
SELECT 
    CURRENT_DATE - (generate_series || ' days')::interval,
    floor(random() * 500 + 100)::int,
    floor(random() * 200 + 50)::int,
    floor(random() * 50 + 5)::int,
    floor(random() * 100 + 10)::int,
    (random() * 50000 + 1000)::numeric(12,2),
    (random() * 500 + 50)::numeric(10,2),
    CURRENT_TIMESTAMP - (generate_series || ' days')::interval
FROM generate_series(11, 365)
ON CONFLICT (date) DO NOTHING;

\echo 'Generated daily metrics for full year (365 days)'

-- ========================================
-- GENERATE MORE ADMIN AUDIT LOG ENTRIES (500 total)
-- ========================================

INSERT INTO admin.audit_log (admin_user_id, action, table_name, record_id, old_data, new_data, ip_address, created_at)
SELECT 
    floor(random() * 3 + 1)::int,
    (ARRAY['CREATE_USER', 'UPDATE_USER', 'DELETE_USER', 'CREATE_PRODUCT', 'UPDATE_PRODUCT', 'DELETE_PRODUCT', 'CREATE_ORDER', 'UPDATE_ORDER', 'CANCEL_ORDER', 'UPDATE_CONFIG', 'VIEW_ANALYTICS', 'EXPORT_DATA', 'IMPORT_DATA', 'BACKUP_DATABASE', 'RESTORE_DATABASE'])[1 + floor(random() * 15)::int],
    (ARRAY['users', 'products', 'orders', 'categories', 'payments', 'system_config'])[1 + floor(random() * 6)::int],
    floor(random() * 100 + 1)::int,
    CASE 
        WHEN random() > 0.5 THEN jsonb_build_object('status', 'old_value', 'updated_at', to_char(CURRENT_TIMESTAMP - interval '1 day', 'YYYY-MM-DD HH24:MI:SS'))
        ELSE NULL
    END,
    jsonb_build_object(
        'status', (ARRAY['active', 'inactive', 'pending', 'completed', 'cancelled'])[1 + floor(random() * 5)::int],
        'updated_at', to_char(CURRENT_TIMESTAMP, 'YYYY-MM-DD HH24:MI:SS'),
        'updated_by', 'admin_' || floor(random() * 3 + 1)::int
    ),
    ('192.168.1.' || floor(random() * 255)::int)::inet,
    CURRENT_TIMESTAMP - (random() * interval '365 days')
FROM generate_series(5, 500);

\echo 'Generated 496 additional audit log entries (total: 500)'

-- ========================================
-- GENERATE MORE SYSTEM CONFIG ENTRIES
-- ========================================

INSERT INTO admin.system_config (key, value, description)
VALUES 
    ('session_timeout', '3600', 'Session timeout in seconds'),
    ('max_login_attempts', '5', 'Maximum failed login attempts before lockout'),
    ('password_min_length', '8', 'Minimum password length'),
    ('enable_two_factor', 'false', 'Enable two-factor authentication'),
    ('backup_retention_days', '30', 'Number of days to retain backups'),
    ('max_file_upload_size', '10485760', 'Maximum file upload size in bytes'),
    ('enable_api_access', 'true', 'Enable API access for external applications'),
    ('api_rate_limit', '1000', 'API requests per hour limit'),
    ('enable_notifications', 'true', 'Enable system notifications'),
    ('notification_email', 'admin@lazytables.dev', 'Admin notification email address'),
    ('system_timezone', 'UTC', 'System timezone'),
    ('date_format', 'YYYY-MM-DD', 'Default date format'),
    ('currency_symbol', '$', 'Currency symbol'),
    ('tax_rate', '0.08', 'Default tax rate'),
    ('shipping_base_cost', '5.99', 'Base shipping cost'),
    ('free_shipping_threshold', '100.00', 'Order amount for free shipping'),
    ('inventory_warning_threshold', '10', 'Low inventory warning threshold'),
    ('enable_guest_checkout', 'true', 'Allow guest checkout'),
    ('order_number_prefix', 'ORD-', 'Prefix for order numbers'),
    ('invoice_number_prefix', 'INV-', 'Prefix for invoice numbers')
ON CONFLICT (key) DO NOTHING;

\echo 'Added additional system configuration entries'

-- ========================================
-- UPDATE STATISTICS FOR QUERY OPTIMIZATION
-- ========================================

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

\echo 'Updated table statistics for query optimization'

-- ========================================
-- DISPLAY FINAL COUNTS
-- ========================================

\echo ''
\echo '====================================='
\echo 'Bulk data generation completed!'
\echo '====================================='
\echo ''
\echo 'Final record counts:'

SELECT 'public.users' as table_name, COUNT(*) as record_count FROM public.users
UNION ALL
SELECT 'public.products', COUNT(*) FROM public.products
UNION ALL
SELECT 'public.categories', COUNT(*) FROM public.categories
UNION ALL
SELECT 'public.orders', COUNT(*) FROM public.orders
UNION ALL
SELECT 'public.order_items', COUNT(*) FROM public.order_items
UNION ALL
SELECT 'public.payments', COUNT(*) FROM public.payments
UNION ALL
SELECT 'analytics.events', COUNT(*) FROM analytics.events
UNION ALL
SELECT 'analytics.sessions', COUNT(*) FROM analytics.sessions
UNION ALL
SELECT 'analytics.daily_metrics', COUNT(*) FROM analytics.daily_metrics
UNION ALL
SELECT 'admin.audit_log', COUNT(*) FROM admin.audit_log
UNION ALL
SELECT 'admin.system_config', COUNT(*) FROM admin.system_config
ORDER BY table_name;

\echo ''
\echo 'Database is now populated with hundreds of test records!'
\echo 'Connection details:'
\echo '  Host: localhost'
\echo '  Port: 5432'
\echo '  Database: test_db'
\echo '  Username: lazytables'
\echo '  Password: lazytables_dev'