-- Add migration script here
CREATE TABLE IF NOT EXISTS SONGS (
    "ID" SERIAL PRIMARY KEY,
    "TITLE" VARCHAR(255) NOT NULL,
    "ARTIST" VARCHAR(255) NOT NULL,
    "YOUTUBE_ID" VARCHAR(255) NOT NULL
);