export interface Toast {
  id: number
  kind: 'success' | 'error'
  title: string
  detail?: string
}

let nextId = 1

export const toasts = $state<{ items: Toast[] }>({ items: [] })

function push(kind: Toast['kind'], title: string, detail?: string) {
  const id = nextId++
  toasts.items.push({ id, kind, title, detail })
  setTimeout(() => dismiss(id), kind === 'error' ? 8000 : 3500)
}

export function dismiss(id: number) {
  const i = toasts.items.findIndex((t) => t.id === id)
  if (i >= 0) toasts.items.splice(i, 1)
}

export const toast = {
  success: (title: string, detail?: string) => push('success', title, detail),
  error: (title: string, detail?: string) => push('error', title, detail),
}
