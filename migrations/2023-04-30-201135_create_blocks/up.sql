CREATE TYPE inet_proto AS ENUM ('tcp', 'udp', 'icmp');

-- Your SQL goes here
CREATE TABLE blocks
(
    id        BIGSERIAL                NOT NULL PRIMARY KEY,
    src_ip    inet                     NOT NULL,
    dst_ip    inet                     NOT NULL,
    proto     inet_proto               NOT NULL,
    port      int4                     NOT NULL,
    event_ts  TIMESTAMP WITH TIME ZONE NOT NULL,
    insert_ts TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX unique_block_idx ON blocks (src_ip, dst_ip, proto, port);
CREATE INDEX event_idx ON blocks (event_ts);
