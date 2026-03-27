CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE webhooks(
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    webhook_id text UNIQUE NOT NULL,
    rules jsonb NOT NULL,
);

CREATE INDEX idx_webhooks_webhook_id ON webhooks(webhook_id);

CREATE TABLE events(
    internal_id serial PRIMARY KEY,
    id uuid UNIQUE DEFAULT gen_random_uuid(),
    webhook_id text NOT NULL,
    headers jsonb NOT NULL,
    body jsonb NOT NULL,
    created_at timestamptz DEFAULT now(),
    updated_at timestamptz
);

INSERT INTO events(id, webhook_id, headers, body)
    VALUES ('00000000-0000-0000-0000-000000000000', 'test-webhook-id', '{"Content-Type": "application/json"}', '{"message": "test event message"}');

CREATE INDEX idx_webhook_id ON events(webhook_id);
