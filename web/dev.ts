import { mkdirSync, watch } from 'fs'

const PORT = Number(process.env.PORT) || 3000
const API_PORT = Number(process.env.API_PORT) || 9000

mkdirSync('./build/client/assets', { recursive: true })

const clients = new Set<WebSocket>()

const RELOAD_SCRIPT = `<script>
(function() {
  const ws = new WebSocket('ws://localhost:${PORT}/__ws');
  ws.onmessage = function(e) {
    if (e.data === 'reload') location.reload();
  };
  ws.onclose = function() {
    setTimeout(function() { location.reload(); }, 1000);
  };
})();
</script>`

const cssWatch = Bun.spawn([
  'bun', 'x', '@tailwindcss/cli',
  '-i', './app/app.css',
  '-o', './build/client/assets/app.css',
  '--watch',
], {
  stdio: ['inherit', 'inherit', 'inherit'],
})

async function serveModule(pathname: string): Promise<Response | null> {
  const filePath = '.' + pathname
  const result = await Bun.build({
    entrypoints: [filePath],
    target: 'browser',
    splitting: false,
    define: { 'import.meta.hot': 'undefined' },
  })
  if (!result.success) {
    console.error('Build failed:', result.logs)
    return null
  }
  const output = result.outputs[0]
  return new Response(output, {
    headers: { 'Content-Type': 'application/javascript' },
  })
}

const server = Bun.serve({
  port: PORT,
  development: true,
  websocket: {
    open(ws) {
      clients.add(ws)
    },
    close(ws) {
      clients.delete(ws)
    },
    message() {},
  },
  async fetch(req, server) {
    const url = new URL(req.url)

    if (url.pathname === '/__ws') {
      if (server.upgrade(req)) return
      return new Response('WebSocket upgrade failed', { status: 500 })
    }

    if (url.pathname === '/__trigger_reload') {
      for (const ws of clients) {
        try { ws.send('reload') } catch {}
      }
      return new Response('ok')
    }

    if (url.pathname.startsWith('/api/')) {
      const target = `http://localhost:${API_PORT}${url.pathname}${url.search}`
      return fetch(target, {
        method: req.method,
        headers: req.headers,
        body: req.body,
      })
    }

    if (url.pathname.startsWith('/assets/')) {
      const file = Bun.file(`./build/client${url.pathname}`)
      if (await file.exists()) {
        return new Response(file)
      }
    }

    if ((url.pathname.endsWith('.tsx') || url.pathname.endsWith('.ts')) && (url.pathname.startsWith('/app/') || url.pathname.startsWith('/src/'))) {
      const response = await serveModule(url.pathname)
      if (response) return response
    }

    let html = await Bun.file('./index.html').text()
    html = html.replace('</body>', `${RELOAD_SCRIPT}</body>`)
    return new Response(html, {
      headers: { 'Content-Type': 'text/html' },
    })
  },
})

console.log(`Dev server: http://localhost:${server.port} (proxy → localhost:${API_PORT})`)

function startFileWatcher() {
  watch('./app', { recursive: true }, () => {
    for (const ws of clients) {
      try { ws.send('reload') } catch {}
    }
  })
  watch('./src', { recursive: true }, () => {
    for (const ws of clients) {
      try { ws.send('reload') } catch {}
    }
  })
}

startFileWatcher()

process.on('SIGINT', () => {
  cssWatch.kill()
  server.stop()
  process.exit(0)
})

process.on('SIGTERM', () => {
  cssWatch.kill()
  server.stop()
  process.exit(0)
})
