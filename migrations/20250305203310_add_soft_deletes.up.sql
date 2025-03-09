-- Add up migration script here

ALTER TABLE profiles 
ADD COLUMN deleted integer not null default 0;

ALTER TABLE expense_records
ADD COLUMN deleted integer not null default 0;

ALTER TABLE possible_links
ADD COLUMN state text not null default 'Active';

ALTER TABLE links 
ADD COLUMN deleted integer not null default 0;

ALTER TABLE data_import 
ADD COLUMN deleted integer not null default 0;
