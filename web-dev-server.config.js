import debug from "debug"
import { esbuildPlugin } from "@web/dev-server-esbuild"

// remap /**/*.ts and /**/*.js to /ts/**/*.ts and let esbuild transform
// don't remap some special urls
async function remap(ctx, next) {
  const dbg = debug("d-s:remap")
  
  const url = ctx.url
  dbg("url", url)
  
  const special = url.startsWith("/__")
  if ((url.endsWith(".ts") || url.endsWith(".js")) && !special) {
    ctx.url = "/ts" + url.substring(0, url.length - 3) + ".ts"
    dbg("-->", ctx.url)
  }
  
  await next(ctx)
}

const hostname = "localhost"
const port = 32118

// sourcemap is inline, a large sourcemap comment at the end of file but:
// 1. for development it is fine
// 2. this sidesteps the issue that esbuild would also transform the
//    linked file used as the sourcemap target
const esbuild = esbuildPlugin({
  ts: true,
  sourcemap: 'inline',
  tsconfig: 'tsconfig.json',
})

export default {
  watch: true,
  debug: true,
  hostname,
  port,
  rootDir: "root",
  middleware: [ remap ],
  plugins: [ esbuild ],
  http2: true,
  nodeResolve: true,
  sslKey: "ssl/localhost.key",
  sslCert: "ssl/localhost.crt",
}

// Copyright see AUTHORS; see LICENSE; SPDX-License-Identifier: ISC+


