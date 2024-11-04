-- Your SQL goes here

create table possible_links (
    uuid blob primary key not null,
    negative blob not null,
    positive blob not null,
    probability double not null
);
