-- Create messages table
CREATE TABLE messages(
    id SERIAL PRIMARY KEY,
    sender_id INTEGER NOT NULL ,
    addressee_id INTEGER NOT NULL,
    timestamp TIMESTAMP NOT NULL default now(),
    payload TEXT NOT NULL
);
