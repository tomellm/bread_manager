-- Add down migration script here

ALTER TABLE profiles 
DROP COLUMN deleted;

ALTER TABLE expense_records
DROP COLUMN deleted;

ALTER TABLE possible_links
DROP COLUMN state;

ALTER TABLE links 
DROP COLUMN deleted;

ALTER TABLE data_import 
DROP COLUMN deleted;


