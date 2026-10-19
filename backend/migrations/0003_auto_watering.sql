ALTER TABLE channels
    ADD COLUMN auto_watering_enable INTEGER NOT NULL DEFAULT 0;

-- Threshold based / interval based (0 = interval, 1 = threshold)
ALTER TABLE channels
    ADD COLUMN auto_watering_type INTEGER NOT NULL DEFAULT 0;

-- In minutes
ALTER TABLE channels
    ADD COLUMN auto_watering_interval INTEGER NOT NULL DEFAULT 300;

-- In %
ALTER TABLE channels
    ADD COLUMN auto_watering_threshold INTEGER NOT NULL DEFAULT 10;
