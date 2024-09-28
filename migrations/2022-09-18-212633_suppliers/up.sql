CREATE TABLE IF NOT EXISTS suppliers (
    id serial PRIMARY KEY NOT NULL,
    products INT[] NOT NULL,
    name TEXT NOT NULL,
    phone_number TEXT,
    email TEXT 
);