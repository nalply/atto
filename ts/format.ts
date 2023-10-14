import { type Document, type Atom } from "./types.ts"
import { rxBare } from "./lexer.ts"
import { strEsc } from "./lex.ts"

export type FormatOpts = {
  style?: 'compact' | 'pretty',
  indent?: 2 | 4 | 8,
  formatAtom?: (atom: Atom, path: string) => string
}

type Required<T> = T & { [p in keyof T]-?: T[p] }

type ReqFormatOpts = Required<FormatOpts>

type DefaultsFn = () => Required<FormatOpts>

const defaults: DefaultsFn = () => ({
  style: 'pretty',
  indent: 2,
  formatAtom: (atom: Atom, _path: string) => atom.toString(),
})

type State = ReqFormatOpts & { 
  level: 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9
}

// todo type Document | { entries() } or similar
export function format(doc: Document, opts: FormatOpts = defaults()): string {
  const defs = defaults()
  const style = opts.style ?? defs.style
  const indent = opts.indent ?? defs.indent
  const formatAtom = opts.formatAtom ?? defs.formatAtom
   
  const state: State = { level: 0, style, indent, formatAtom }

  // for non-TypeScript users
  if (! [ 'compact', 'pretty' ].includes(style))
    throw new TypeError(`unsupported style value '${style}'`)
  
  if (! [ 2, 4, 8 ].includes(indent))
    throw new TypeError(`unsupported indent \`${indent}\``)
  
  if (style === 'pretty') throw new TypeError("pretty unimplemented")

  if (opts.formatAtom) throw new TypeError("formatAtom unimplemented")
  
  return formatCompound(doc, state)
}


// todo method for converting into key-value pairs
function formatCompound(doc: Document, state: State): string {
  let result = ""
  
  if (Array.isArray(doc)) {
    for (const value of doc)
      result += " " + formatValue(value, state)

    return result.substring(1)
  }
  else if (typeof doc === 'object') {
    for (const [ key, value ] of Object.entries(doc))
     result += " " + formatBare(key) 
        + ": " + formatValue(value, state)

    return result.substring(1)
  }
  else {
    return formatValue(doc, state)
  }
}


// method for converting to string and check method for converting to entries
function formatValue(value: any, state: State): string {
  switch (typeof value) {
    case 'number': case 'boolean': return value.toString()
    case 'string': return formatBare(value)
  }

  if (Array.isArray(value) || typeof value === 'object') {
    return "(" + formatCompound(value, state) + ")"
  }
  
  throw new TypeError("invalid value " + typeof value)
}

function formatBare(s: string) {
  return rxBare?.test(s) ? s : '"' + strEsc(s) + '"'
}

// Copyright see AUTHORS; see LICENSE; SPDX-License-Identifier: ISC+

