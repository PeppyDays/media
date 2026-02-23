CREATE TABLE images (
    id           TEXT        PRIMARY KEY,
    status       TEXT        NOT NULL,
    content_type TEXT        NOT NULL,
    file_name    TEXT        NOT NULL,
    size_bytes   BIGINT,
    object_key   TEXT        NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL,
    updated_at   TIMESTAMPTZ NOT NULL
);
