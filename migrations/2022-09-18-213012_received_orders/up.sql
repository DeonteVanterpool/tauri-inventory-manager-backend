CREATE TABLE IF NOT EXISTS received_orders (
    id serial PRIMARY KEY NOT NULL,
    received TIMESTAMP,
    product_id INT NOT NULL REFERENCES products,
    gross_amount float NOT NULL,
    actually_received float NOT NULL,
    damaged float NOT NULL
);
