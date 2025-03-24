-- Add down migration script here

ALTER TABLE links
RENAME COLUMN leading TO negative;

ALTER TABLE links
RENAME COLUMN following TO positive;

ALTER TABLE links
DROP COLUMN link_type;


ALTER TABLE possible_links 
RENAME COLUMN leading TO negative;

ALTER TABLE possible_links 
RENAME COLUMN following TO positive;

ALTER TABLE possible_links
DROP COLUMN link_type;
