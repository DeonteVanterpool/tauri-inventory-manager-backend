CREATE TABLE IF NOT EXISTS preferences (
    user_id INT PRIMARY KEY NOT NULL REFERENCES users
);
