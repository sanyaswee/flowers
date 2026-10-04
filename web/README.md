# Web App

Dashboard for the Flowers nodes: live soil moisture, water tank and climate readings, manual watering, per-channel and per-node settings, and history charts. Built with Svelte 5, Vite and TypeScript

## Running

The backend sends no CORS headers and doesn't serve static files, so the browser has to reach `/api` through the same origin as the app. Vite proxies `/api` to the backend

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
- **Node settings.** `POST .../settings` takes the whole settings struct and overwrites every channel's settings too. The app re-reads the channels right before saving and sends them back unchanged.
- **History.** Rows come back newest first with a default limit of 100, so the app asks for up to 8,000 rows and draws gaps where a node was silent.

## Contract notes

Differences between the generated OpenAPI file and the backend source:

- Timestamps are UTC with no offset (`2026-10-03T16:00:00.123456`). The `start` and `end` query parameters must be sent the same way. A trailing `Z` is rejected with 400.
- `NodeSettings.plant_settings` is a fixed array and must contain exactly 8 entries (`MAX_PLANT_CHANNELS`), even for nodes with fewer channels.
- All durations and frequencies are `u16` (1 to 65535 s), not `int32`.
- `GET /api/nodes/{id}` and `GET .../channels/{id}` return `""` for an unnamed node/channel where the list endpoints return `null`. The client treats both as unnamed.
- `NodeEntry.water_tank_level` is a capability flag (the tank reports a percentage), not a reading.
- Units: soil moisture and tank level in %, temperature °C, pressure hPa, light lux. Higher soil moisture means wetter
