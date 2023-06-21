-- Add migration script here
-- create table subscriptions_token table
CREATE TABLE subscriptions_token (
    subscriptions_token TEXT NOT NULL,
    subscription_id uuid NOT NULL REFERENCES subscriptions(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
);