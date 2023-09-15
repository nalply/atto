import { playwrightLauncher } from '@web/test-runner-playwright'
import { esbuildPlugin } from "@web/dev-server-esbuild"

const chromium = playwrightLauncher({ product: 'chromium' })
const firefox = playwrightLauncher({ product: 'firefox' })

const hostname = "localhost"
const port = 32117

const esbuild = esbuildPlugin({
  ts: true,
  tsconfig: 'tsconfig.json',
})

// Silence 404 in browser for favicon.ico by returning empty file
async function emptyFavicon(ctx, next) {
  if (ctx.url === "/favicon.ico") {
    ctx.status = 200
    ctx.body = ""
    console.log(ctx.url)
    return
  }
  
  await next(ctx)
}

export default {
  files: "*.test.ts",
  watch: true,
  hostname,
  port,
  rootDir: ".",
  middleware: [ emptyFavicon ], 
  plugins: [ esbuild ],
  playwright: true,
  browsers: [ chromium ],
  nodeResolve: true,
}

// Copyright see AUTHORS; see LICENSE; SPDX-License-Identifier: ISC+


