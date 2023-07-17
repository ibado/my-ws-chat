--  create users table
CREATE TABLE users(
    id SERIAL PRIMARY KEY,
    nickname TEXT NOT NULL ,
    password_hash TEXT NOT NULL
);
