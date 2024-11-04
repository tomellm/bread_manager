-- Your SQL goes here

create table profiles (
    uuid blob primary key not null,
    name text not null,
    origin_name text not null,
    data blob not null
);
