-- Apply manually for local development.
-- There is no updated_at trigger; application code sets updated_at in all UPDATE queries.

CREATE TABLE images (
    id           TEXT        PRIMARY KEY,
    status       TEXT        NOT NULL,
    content_type TEXT        NOT NULL,
    file_name    TEXT        NOT NULL,
    size_bytes   BIGINT,
    object_key   TEXT        NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
