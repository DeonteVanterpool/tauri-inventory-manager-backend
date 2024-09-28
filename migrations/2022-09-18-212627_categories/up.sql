CREATE TABLE IF NOT EXISTS categories (
    id serial PRIMARY KEY NOT NULL,
    products INT[] NOT NULL,
    name TEXT NOT NULL
);
