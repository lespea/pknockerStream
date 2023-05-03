-- Your SQL goes here
CREATE PROCEDURE clean_db()
    LANGUAGE SQL
AS
$$
-- Clean added
DELETE
FROM blocks
WHERE event_ts < NOW() - '1 day'::INTERVAL
   OR insert_ts < NOW() - '1 day'::INTERVAL ;

-- Clean added
DELETE
FROM added
WHERE added_on < NOW() - '1 day'::INTERVAL;

-- Clean denies
DELETE
FROM denies
WHERE added_on < NOW() - '1 day'::INTERVAL;
$$;
