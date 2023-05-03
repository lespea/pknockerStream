-- Your SQL goes here

CREATE TABLE denies
(
    ip       inet                     NOT NULL PRIMARY KEY,
    added_on TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX ON denies (added_on);
