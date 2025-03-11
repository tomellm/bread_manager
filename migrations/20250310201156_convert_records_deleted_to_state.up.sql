-- Add up migration script here

ALTER TABLE expense_records 
ADD COLUMN state text not null default 'Active';

UPDATE expense_records
SET state = 'Deleted'
WHERE deleted = 1;

ALTER TABLE expense_records
DROP COLUMN deleted;
