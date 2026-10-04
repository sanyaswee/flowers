// Minimal hash router: "#/" is the dashboard, "#/nodes/<node_id>" is a node.
// Hash routing needs no server-side fallback, which keeps hosting on the LAN trivial.

export type Route = { name: 'home' } | { name: 'node'; nodeId: string }

function parse(hash: string): Route {
  const parts = hash.replace(/^#\/?/, '').split('/').filter(Boolean)
  if (parts[0] === 'nodes' && parts[1]) return { name: 'node', nodeId: decodeURIComponent(parts[1]) }
  return { name: 'home' }
}

export const router = $state<{ route: Route }>({ route: parse(location.hash) })

window.addEventListener('hashchange', () => {
  router.route = parse(location.hash)
  window.scrollTo(0, 0)
})

export const homeHref = '#/'
export const nodeHref = (nodeId: string) => `#/nodes/${encodeURIComponent(nodeId)}`
