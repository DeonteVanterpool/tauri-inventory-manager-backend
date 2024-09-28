CREATE TABLE IF NOT EXISTS users (
    id serial PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    password VARCHAR(255) NOT NULL /* Don't worry, the passwords are hashed and salted */
);
