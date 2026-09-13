PRAGMA foreign_keys = ON;

-- Create node table (int instead of bools)
CREATE TABLE nodes (
    id INTEGER PRIMARY KEY,
    node_id TEXT UNIQUE NOT NULL,
    verbose_name TEXT,
    -- Node config
    n_channels INTEGER NOT NULL,
    water_tank INTEGER NOT NULL ,
    water_tank_level INTEGER NOT NULL,
    temperature INTEGER NOT NULL,
    humidity INTEGER NOT NULL,
    pressure INTEGER NOT NULL,
    light INTEGER NOT NULL,
    -- Node settings
    telemetry_report_freq INTEGER NOT NULL,
    light_m_freq INTEGER NOT NULL,
    bmpe_m_freq INTEGER NOT NULL,
    -- Other data
    last_boot DATETIME,
    last_active DATETIME
);

-- Channels (attached to node)
CREATE TABLE channels (
    id INTEGER PRIMARY KEY,
    node_id TEXT NOT NULL REFERENCES nodes (node_id),
    channel_id INTEGER NOT NULL,
    enabled INTEGER NOT NULL,
    moisture_m_freq INTEGER NOT NULL
);

-- Node telemetry
CREATE TABLE node_telemetry (
    -- Auth
    id INTEGER PRIMARY KEY,
    node_id TEXT NOT NULL REFERENCES nodes (node_id),
    -- Timestamp
    timestamp DATETIME NOT NULL,
    uptime INTEGER NOT NULL,
    -- Actual telemetry
    water_tank_has_water INTEGER,
    water_tank_level FLOAT,
    temperature FLOAT,
    pressure FLOAT,
    humidity FLOAT,
    light_intensity FLOAT
);

-- Channel telemetry
CREATE TABLE channel_telemetry (
    -- Auth
    id INTEGER PRIMARY KEY,
    node_id TEXT NOT NULL REFERENCES nodes (node_id),
    channel_id INTEGER NOT NULL,
    -- Timestamp
    timestamp DATETIME NOT NULL,
    uptime INTEGER NOT NULL,
    -- Telemetry
    soil_moisture FLOAT
);