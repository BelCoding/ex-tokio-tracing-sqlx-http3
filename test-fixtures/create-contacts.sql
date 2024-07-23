CREATE TABLE IF NOT EXISTS public.contacts(
    id SERIAL PRIMARY KEY,
    email TEXT NOT NULL,
    number TEXT NOT NULL
);