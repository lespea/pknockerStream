-- Your SQL goes here
CREATE TABLE added
(
    src_ip   inet                     NOT NULL,
    dst_ip   inet                     NOT NULL,
    added_on TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    CONSTRAINT added_pk PRIMARY KEY (src_ip, dst_ip)
);

CREATE INDEX ON added (added_on);
