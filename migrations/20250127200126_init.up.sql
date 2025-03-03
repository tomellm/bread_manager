-- Add up migration script here
CREATE TABLE IF NOT EXISTS profiles (
    uuid blob primary key not null,
    name text not null,
    origin_name text not null,
    data blob not null
);

CREATE TABLE IF NOT EXISTS expense_records (
    datetime_created timestamp not null,
    uuid blob primary key not null,
    amount integer not null,
    datetime timestamp not null,
    description text,
    description_container blob not null,
    tags text not null,
    origin text not null,
    data blob not null,
    data_import blob not null,
    FOREIGN KEY (data_import) REFERENCES data_import(uuid)
);

CREATE TABLE IF NOT EXISTS possible_links (
    uuid blob primary key not null,
    negative blob not null,
    positive blob not null,
    probability double not null
);

CREATE TABLE IF NOT EXISTS links (
    uuid blob primary key not null,
    negative blob not null,
    positive blob not null
);

CREATE TABLE IF NOT EXISTS data_import (
    uuid blob primary key not null,
    imported_at string not null,
    profile_used blob not null,
    FOREIGN KEY (profile_used) REFERENCES profiles(uuid)
);
