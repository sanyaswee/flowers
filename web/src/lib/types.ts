// Types for the Flowers backend API.
//
// These follow the backend source (backend/src/db/entries/*, shared/src/node_settings.rs),
// not just the generated OpenAPI file. Where the two disagree, the comment says how.

/** shared::MAX_PLANT_CHANNELS. `NodeSettings.plant_settings` must always have exactly this many entries. */
export const MAX_PLANT_CHANNELS = 8

/** Every duration/frequency field is a u16 on the server and firmware. The OpenAPI file says int32. */
export const U16_MAX = 65535

export interface PlantSettings {
  enabled: boolean
  /** Soil moisture measurement interval, seconds (1..=65535) */
  moisture_m_freq_s: number
  /** Pump run time, seconds (1..=65535) */
  watering_time_s: number
}

export interface MeasurementFrequencies {
  /** Light sensor interval, seconds */
  light_intensity_s: number
  /** Temperature / pressure / humidity interval, seconds */
  bmpe_s: number
}

export interface NodeSettings {
  /** How often the node packs readings into a telemetry packet, seconds */
  telemetry_packet_creation_freq_s: number
  /** Fixed-size array: always MAX_PLANT_CHANNELS entries, even if the node has fewer channels */
  plant_settings: PlantSettings[]
  m_freq: MeasurementFrequencies
}

/** Timestamps are UTC without an offset, e.g. "2026-10-03T16:00:00.123456". Use parseServerTime(). */
export type ServerTime = string

export interface NodeEntry {
  id: number
  node_id: string
  verbose_name: string | null
  n_channels: number
  /** Capability flags: what this node can measure, not current values */
  water_tank: boolean
  /** true = reports a tank level (%), false (with water_tank) = only empty / not empty */
  water_tank_level: boolean
  temperature: boolean
  humidity: boolean
  pressure: boolean
  light: boolean
  telemetry_report_freq: number
  light_m_freq: number
  bmpe_m_freq: number
  last_boot: ServerTime
  /** Updated only when a telemetry packet arrives */
  last_active: ServerTime
}

export interface ChannelEntry {
  id: number
  node_id: string
  channel_id: number
  verbose_name: string | null
  enabled: boolean
  moisture_m_freq: number
  watering_time: number
}

/** Rows come back newest first. Units: temperature °C, pressure hPa, light lux, humidity and tank level %. */
export interface NodeTelemetryEntry {
  id: number
  node_id: string
  timestamp: ServerTime
  uptime: number
  water_tank_has_water: boolean | null
  water_tank_level: number | null
  temperature: number | null
  pressure: number | null
  humidity: number | null
  light_intensity: number | null
}

/** Rows come back newest first. soil_moisture is a percentage, higher is wetter. */
export interface ChannelTelemetryEntry {
  id: number
  node_id: string
  channel_id: number
  timestamp: ServerTime
  uptime: number
  soil_moisture: number
}

export const DEFAULT_PLANT_SETTINGS: PlantSettings = {
  enabled: false,
  moisture_m_freq_s: 600,
  watering_time_s: 5,
}
