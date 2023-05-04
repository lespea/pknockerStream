-- Your SQL goes here
CREATE VIEW view_to_check AS
SELECT b.src_ip,
       b.dst_ip,
       JSON_AGG(JSON_BUILD_ARRAY(b.proto, b.port)) AS conns
--        JSON_AGG(JSON_BUILD_OBJECT('proto', b.proto, 'port', b.port) ORDER BY b.event_ts) AS conns
FROM blocks b
         LEFT OUTER JOIN denies d ON b.src_ip = d.ip
         LEFT OUTER JOIN added a ON b.src_ip = a.src_ip AND b.dst_ip = a.dst_ip
WHERE b.port != 22
  AND d.ip IS NULL
  AND a.dst_ip IS NULL
GROUP BY b.src_ip, b.dst_ip
HAVING COUNT(b.src_ip) BETWEEN 3 AND 10
;
