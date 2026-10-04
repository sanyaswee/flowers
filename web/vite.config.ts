import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

// The backend sends no CORS headers and doesn't serve static files, so the browser must reach
// it through the same origin as the web app. Both the dev server and `vite preview` proxy /api.
// Override the target with FLOWERS_BACKEND=http://host:3000 when the backend runs elsewhere.
const backend = process.env.FLOWERS_BACKEND ?? 'http://127.0.0.1:3000'

const proxy = {
  '/api': {
    target: backend,
    changeOrigin: true,
  },
}

export default defineConfig({
  plugins: [svelte()],
  server: {
    port: 5173,
    // Listen on the LAN so phones and other computers can open the dev server
    host: true,
    proxy,
  },
  preview: {
    port: 4173,
    host: true,
    // Accept any Host header (LAN IPs, mDNS names) instead of only localhost
    allowedHosts: true,
    proxy,
  },
})
