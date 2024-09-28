CREATE TABLE IF NOT EXISTS products (
    id serial PRIMARY KEY NOT NULL,
    upc TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    amount FLOAT NOT NULL,
    case_size INT,
    measure_by_weight BOOLEAN NOT NULL,
    cost_price_per_unit NUMERIC(10, 4) NOT NULL,
    selling_price_per_unit NUMERIC(10, 4) NOT NULL,
    sale_end TIMESTAMP,
    buy_level FLOAT,
    sale_price NUMERIC(10, 4)
);
