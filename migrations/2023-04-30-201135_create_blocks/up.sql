CREATE TYPE inet_proto AS ENUM ('tcp', 'udp', 'icmp');

-- Your SQL goes here
CREATE TABLE blocks
(
    id        SERIAL PRIMARY KEY,
    src_ip    inet                     NOT NULL,
    dst_ip    inet                     NOT NULL,
    proto     inet_proto               NOT NULL,
    event_ts  TIMESTAMP WITH TIME ZONE NOT NULL,
    insert_ts TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
