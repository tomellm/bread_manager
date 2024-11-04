-- Your SQL goes here

create table links (
    uuid blob primary key not null,
    negative blob not null,
    positive blob not null
);
