CREATE TABLE IF NOT EXISTS brands (
    id serial PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    products INT[] NOT NULL
);
