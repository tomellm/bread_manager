-- Your SQL goes here

create table expense_records (
    datetime_created timestamp not null,
    uuid blob primary key not null,
    amount integer not null,
    datetime timestamp not null,
    description text,
    description_container blob not null,
    tags text not null,
    origin text not null,
    data blob not null
);
