import { esbuildPlugin } from "@web/dev-server-esbuild"

import { puppeteerLauncher } from '@web/test-runner-puppeteer'
const puppeteerChrome = puppeteerLauncher()

import { playwrightLauncher } from '@web/test-runner-playwright'
const playwrightChromium = playwrightLauncher({ product: 'chromium' })
const playwrightFirefox = playwrightLauncher({ product: 'firefox' })


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
    // console.log(ctx.url)
    return
  }
  
  await next(ctx)
}

console.warn("playwright does not work under Arch", 
  playwrightChromium, playwrightFirefox)

export default {
  files: "*.test.ts",
  watch: true,
  hostname,
  port,
  rootDir: ".",
  middleware: [ emptyFavicon ], 
  plugins: [ esbuild ],
  puppeteer: true,
  playwright: true,
  browsers: [ puppeteerChrome ],
  nodeResolve: true,
}

// Copyright see AUTHORS & LICENSE; SPDX-License-Identifier: ISC+
