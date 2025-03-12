-- Add down migration script here

ALTER TABLE expense_records
ADD COLUMN deleted integer not null default 0;

UPDATE expense_records
SET deleted = 1
WHERE state = 'Deleted';

ALTER TABLE expense_records
DROP COLUMN deleted;
