DROP TABLE IF EXISTS public."contacts";
DROP TABLE IF EXISTS public."contacts_t";
CREATE TABLE "contacts" (
    id SERIAL PRIMARY KEY,
    email TEXT NOT NULL,
    number TEXT NOT NULL
);
CREATE TABLE "contacts_t" (
    id SERIAL PRIMARY KEY,
    email TEXT NOT NULL,
    number TEXT NOT NULL
);


INSERT INTO public."contacts" (id, email , number)
VALUES
    -- Create a list of emails instead of names for each contact

    (1, 'alice@domain.com', '123-456-7890'),
    (2, 'bob@domain.com', '234-567-8901'),
    (3, 'charlie@domain.com', '345-678-9012'),
    (4, 'david@domain.com', '456-789-0123'),
    (5, 'eve@domain.com', '567-890-1234');
-- Add up migration script here