-- Add up migration script here

ALTER TABLE links
RENAME COLUMN negative TO leading;

ALTER TABLE links
RENAME COLUMN positive TO following;

ALTER TABLE links
ADD COLUMN link_type text not null default 'Transfer';


ALTER TABLE possible_links 
RENAME COLUMN negative TO leading;

ALTER TABLE possible_links 
RENAME COLUMN positive TO following;

ALTER TABLE possible_links
ADD COLUMN link_type text not null default 'Transfer';

