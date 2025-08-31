CREATE TABLE IF NOT EXISTS log (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v1mc(),
    email text[] NOT NULL,          -- array of emails
    subject text NOT NULL,
    filename text NOT NULL,
    user_agent text[],              -- array of user agents
    ip inet[],                      -- array of IPs
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz
);