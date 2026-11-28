-- When the last watering command was sent for this channel (UTC), whether it was
-- triggered manually or by auto-watering. NULL = never watered.
-- Interval based auto-watering counts from this moment.
ALTER TABLE channels
    ADD COLUMN last_watered DATETIME;
