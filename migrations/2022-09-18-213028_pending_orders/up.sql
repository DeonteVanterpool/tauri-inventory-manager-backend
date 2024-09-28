CREATE TABLE IF NOT EXISTS pending_orders (
    id serial PRIMARY KEY NOT NULL,
    product_id INT NOT NULL REFERENCES products,
    amount float NOT NULL
);
