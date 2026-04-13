import { mkdirSync, rmSync, readFileSync, writeFileSync } from 'fs'
import { brotliCompressSync } from 'zlib'
import { createHash } from 'crypto'
import { join, basename as pathBasename } from 'path'

const outDir = '../apps/eai/static'
const assetsDir = join(outDir, 'assets')

const manifest: Record<string, string> = {}

function hashContent(data: Uint8Array): string {
  return createHash('sha256').update(data).digest('hex').slice(0, 8)
}

rmSync(outDir, { recursive: true, force: true })
mkdirSync(assetsDir, { recursive: true })

console.log('Building CSS...')
const cssResult = Bun.spawnSync([
  'bun', 'x', '@tailwindcss/cli',
  '-i', './app/app.css',
  '-o', join(assetsDir, 'app.css'),
])
if (!cssResult.success) {
  console.error('CSS build failed:', cssResult.stderr.toString())
  process.exit(1)
}

const cssData = readFileSync(join(assetsDir, 'app.css'))
const cssHash = hashContent(cssData)
const cssHashed = `app.${cssHash}.css`
writeFileSync(join(assetsDir, cssHashed), cssData)
manifest['app.css'] = cssHashed
rmSync(join(assetsDir, 'app.css'))
console.log(`  app.css → ${cssHashed}`)

console.log('Building JS...')
const jsResult = await Bun.build({
  entrypoints: ['./app/entry.tsx'],
  outdir: assetsDir,
  minify: true,
  naming: '[name].[hash].[ext]',
  splitting: true,
  target: 'browser',
})
if (!jsResult.success) {
  console.error('JS build failed:', jsResult.logs)
  process.exit(1)
}
console.log(`  Bundled ${jsResult.outputs.length} files`)

for (const output of jsResult.outputs) {
  const basename = pathBasename(output.path)
  const data = output.size !== undefined ? readFileSync(output.path) : new Uint8Array()

  const hash = hashContent(data)
  const dotIndex = basename.lastIndexOf('.')
  const ext = basename.slice(dotIndex)
  const namePart = basename.slice(0, dotIndex)
  const baseName = namePart.includes('.') ? namePart.slice(0, namePart.lastIndexOf('.')) : namePart
  const originalName = `${baseName}${ext}`
  manifest[originalName] = basename

  console.log(`  ${originalName} → ${basename}`)
}

writeFileSync(join(assetsDir, 'manifest.json'), JSON.stringify(manifest, null, 2))

const entryJs = manifest['entry.js'] || 'entry.js'
const appCss = manifest['app.css'] || 'app.css'

const html = `<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <link rel="icon" type="image/svg+xml" href="/favicon.svg" />
    <title>Engai</title>
    <link rel="preload" href="/assets/${appCss}" as="style" />
    <link rel="stylesheet" href="/assets/${appCss}" />
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/assets/${entryJs}"></script>
  </body>
</html>`

await Bun.write(`${outDir}/index.html`, html)

console.log('Compressing assets with Brotli...')
for (const file of Object.values(manifest)) {
  const filePath = join(assetsDir, file)
  if (!filePath.endsWith('.js') && !filePath.endsWith('.css')) continue
  const data = readFileSync(filePath)
  const compressed = brotliCompressSync(data)
  writeFileSync(`${filePath}.br`, compressed)
  rmSync(filePath)
  console.log(`  ${file}: ${data.length} -> ${compressed.length} bytes (${Math.round((1 - compressed.length / data.length) * 100)}% reduction)`)
}

console.log('Build complete.')
