-- Add down migration script here
-- Drop the table
DROP TABLE IF EXISTS public.manga;

-- Drop the user
DROP USER IF EXISTS mangaka;
