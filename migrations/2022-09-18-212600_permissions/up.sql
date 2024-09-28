CREATE TABLE IF NOT EXISTS permissions (
    user_id INT PRIMARY KEY NOT NULL REFERENCES users,
    admin BOOLEAN NOT NULL,
    view_pending BOOLEAN NOT NULL,
    view_received BOOLEAN NOT NULL,
    edit_pending BOOLEAN NOT NULL,
    create_orders BOOLEAN NOT NULL,
    edit_received BOOLEAN NOT NULL,
    view_products BOOLEAN NOT NULL,
    edit_products BOOLEAN NOT NULL,
    remove_orders BOOLEAN NOT NULL,
    view_suppliers BOOLEAN NOT NULL
);
