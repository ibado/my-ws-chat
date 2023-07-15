-- Create messages table
CREATE TABLE messages(
    id SERIAL PRIMARY KEY,
    sender_id TEXT NOT NULL ,
    addressee_id TEXT NOT NULL,
    timestamp TIMESTAMP NOT NULL default now(),
    payload TEXT NOT NULL
);

