# Web App

Dashboard for the nodes: soil moisture, water tank and climate readings, manual watering, per-channel and per-node settings, and history charts

## Running

The backend sends no CORS headers and doesn't serve static files, so the browser has to reach `/api` through the same origin as the app. Vite proxies `/api` to the backend.

### Debian/Ubuntu
```sh
cd web
npm install

# development, with hot reload (http://<this-machine>:5173)
npm run dev

# "production" on the local network (http://<this-machine>:4173)
npm run build
npm run preview
```

The backend is expected at `http://127.0.0.1:3000`. If it runs elsewhere:

```sh
FLOWERS_BACKEND=http://192.168.1.20:3000 npm run preview
```

### Android/Termux
Assuming you've already installed `npm` and `pm2` while setting up [backend](../backend/README.md):
```sh
cd web
npm install
pm2 start npm --name "flowers-web" -- run dev -- --host
```

## Layout

```
src/
  App.svelte               app shell: header, sync status, routing
  routes/Dashboard.svelte  all nodes at a glance, water buttons
  routes/NodePage.svelte   one node: channels, history charts, settings
  lib/api.ts               typed client for the backend
  lib/types.ts             API types (see "Contract notes")
  lib/store.svelte.ts      polled node/channel lists and latest readings
  lib/actions.ts           every call that changes something, with toasts
  lib/components/          chart, soil probe, forms, etc.
```

## Behavior worth knowing

- **Online status.** The backend has no heartbeat yet. A node counts as online if its last telemetry arrived within three report intervals plus 15 s. The browser reads the server clock from the `Date` header
- **Watering.** `POST .../water` only publishes an MQTT message (QoS 0, no acknowledgement), so the UI says the command was *sent*. The button is disabled for the watering time plus 3 s: the firmware queues one extra run if a second command arrives while the pump is on. It is also disabled for disabled channels, because the firmware keeps such a command and runs it the moment the channel is enabled. Watering a node that looks offline asks for confirmation first.
- **Auto-watering.** Runs on the server only (`backend/src/auto_watering.rs`), nodes know nothing about it. Every 30 s the server sends an ordinary water command to each enabled channel that is due. "Due" means the interval has passed since `last_watered`, which manual watering updates too, so pressing the button restarts the interval. A channel that was never watered is due right away, so turning auto-watering on for it waters it within about 30 s. The server skips a due channel (and retries on the next check) while the node looks offline or reports an empty water tank, and it never auto-waters disabled channels, for the same reason the button is disabled for them. After downtime a channel is watered once, not once per missed interval. Only the schedule mode exists for now: the moisture-based mode is shown as unavailable, and the server rejects it with [422](https://http.cat/422).
- **Node settings.** `POST .../settings` takes the whole settings struct and overwrites every channel's settings too. The app re-reads the channels right before saving and sends them back unchanged.
- **History.** Rows come back newest first with a default limit of 100, so the app asks for up to 8,000 rows and draws gaps where a node was silent.

## Contract notes

Differences between the generated OpenAPI file and the backend source:

- Timestamps are UTC with no offset (`2026-10-03T16:00:00.123456`). The `start` and `end` query parameters must be sent the same way. A trailing `Z` is rejected with 400.
- `NodeSettings.plant_settings` is a fixed array and must contain exactly 8 entries (`MAX_PLANT_CHANNELS`), even for nodes with fewer channels.
- All durations and frequencies are `u16` (1 to 65535 s), not `int32`.
- `GET /api/nodes/{id}` and `GET .../channels/{id}` return `""` for an unnamed node/channel where the list endpoints return `null`. The client treats both as unnamed.
- `POST .../channels/{id}/auto-watering` takes all four `auto_watering_*` fields of `ChannelEntry` and stores them without contacting the node. The interval is in whole minutes (1 to 65535), the threshold is 0 to 100 %, and `auto_watering_type` must be `IntervalBased` for now. Anything else is a [422](https://http.cat/422). `ChannelEntry.last_watered` is `null` until the first water command.
- `NodeEntry.water_tank_level` is a capability flag (the tank reports a percentage), not a reading.
- Units: soil moisture and tank level in %, temperature °C, pressure hPa, light lux. Higher soil moisture means wetter
