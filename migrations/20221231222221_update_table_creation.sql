-- Add migration script here
create extension if not exists "uuid-ossp";

create table users (
    id uuid default uuid_generate_v4() primary key,
    title varchar(255) not null,
    artist varchar(255) not null,
    youtube_id varchar(255) not null
);