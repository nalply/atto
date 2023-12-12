import type { Document, List, Entry, Atom, Value } from "./types.ts"
import type { Token, Tokenizer } from "./lex.ts"
import { strEsc } from "./lex.ts"
import { lexer } from "./lexer.ts"

export function parse(text: string, dbg: boolean = false): Document {
  const tokenizer = lexer.tokenizerBuilder(text)
  lexer.dbg = dbg

  const document = new Parse(tokenizer).document(undefined, "top")
  if (isStop(document)) error("todo unexpected")
  return document
}

export class ParseError extends Error {
  constructor(msg: string) { super(msg) }
}

export type Location = { line: number, column: number }

export const STOP = Symbol('STOP')

export type Stop = Partial<Location> & { stop: typeof STOP }

export const CONTINUE = Symbol('CONTINUE')
export const END = Symbol('END')
export const FINAL = Symbol('FINAL')
export const REQUIRED = Symbol('REQUIRED')

type CONTINUE = typeof CONTINUE
type END = typeof END
type REQUIRED = typeof REQUIRED

function stop(location: Partial<Location> = {}): Stop {
  return { stop: STOP, ...location}
}

function isStop(value: any): value is Stop {
  return value?.stop === STOP
}

function error(error: string, { line, column }: Partial<Location> = {}): never {
  const loc = line ? " at " + line + (column ? ":" + column : "") : ""
  throw new ParseError(error + loc)
}

export class Parse {
  readonly held: Token[] = []
  readonly tokenizer: Tokenizer
  readonly errors: string[] = []

  constructor(tokenizer: Tokenizer) {
    this.tokenizer = tokenizer
  }

  next(): Token {
    const token = this.held.pop() ?? this.tokenizer.nextToken()
    return token
  }

  back<T>(token: Token, result: T | undefined = undefined): T {
    this.held.push(token)
    return result as T
  }

  document(firstKey?: Atom, top?: "top"): Document | Stop {
    this.ws()
    const firstEntry = this.entry(firstKey)
    if (isStop(firstEntry)) return firstEntry
    
    const document = { [firstEntry.key]: firstEntry.value }
    while (true) {
      const ws = this.ws(REQUIRED, top)
      if (ws === END) return document
      if (isStop(ws)) error("no whitespace between entries", ws)
       
      const entry = this.entry()
      if (isStop(entry)) return entry
      document[entry.key] = entry.value
    }
  }

  ws(opt?: REQUIRED, top?: "top"): END | Stop | void {
    let token = this.next()
    if (token.name === ')') return END
    if (token.name === 'FINAL')
      return top === "top" 
        ? this.back(token, END)
        : error("unexpected end of text", token)
    if (token.name !== 'ws')
      return opt === REQUIRED ? stop(token) : this.back(token)

    while (true) {
      token = this.next()
      if (token.name !== 'ws' && token.name !== 'comment') {
        if (token.name === ')') return END
        this.back(token)
        break
      }
    }
  }

  entry(key?: Atom | Stop): Entry | Stop {
    if (key == null) {
      const atom = this.atom()
      if (atom === CONTINUE) // TS didn't catch that CONTINUE isn't returned
        return error("internal error") // so let's guard this error
      key = atom
    }
    if (isStop(key)) return key

    this.ws()
    const token = this.next()
    if (token.name !== ":")
      return error("no colon after key", token)
    this.ws()

    const value = this.value()
    if (isStop(value) || value === END)
      return error("no value for entry")

    return { key, value }
  }

  atom(opt?: CONTINUE): Atom | Stop | CONTINUE {
    let token = this.next()

    if (token.name === "bare") return token.text
    if (token.text === `"`) return this.str()
    if (token.name === `#`) return this.guardStr()
    
    if (token.name === 'FINAL')
      return error("unexpected end of text", token)

    if (opt === CONTINUE) {
      this.back(token)
      return CONTINUE
    }

    const esc = strEsc(token.text, "guillemet")
    return error("invalid atom " + esc, token)
  }

  value(): Value | Stop | END {
    const atom = this.atom(CONTINUE)
    if (atom !== CONTINUE) return atom // Atom | Stop

    const token = this.next()
    if (token.name === "(") {
      const first = this.value()
      if (isStop(first)) return first
      if (first === END) return [] // ")" follows immediately == emtpy list

      const optColon = this.next()
      if (optColon.name === ":") return this.document(first as Atom)

      this.back(optColon)
      return this.list(first)
    }

    if (token.name === ")")
      return END

    return error("invalid value `" + token.text + "`")
  }

  str(): Atom | Stop {
    let atom = ""
    while (true) {
      const token = this.next()

      switch (token.name) {
        case "FINAL": return error("unexpected end of text in string")
        case `"`: return atom
        case "chars": atom += token.text; break
        case `\\`: break
        case "esc": atom += unesc(token.text); break
        default: return error("internal error, unexpected " + token.name)
      }
    }
  }

  guardStr(): Atom | Stop {
    return error("unimplemented")
  }

  list(firstValue: Value): List | Stop {
    const list = [ firstValue ]
        
    while (true) {
      const ws = this.ws(REQUIRED)  
      if (ws === END) return list
      if (isStop(ws)) return error("no whitespace between values", ws)
       
      const value = this.value()
      if (value === END) return list
      if (isStop(value)) return value

      list.push(value)
    }
  }
}

function unesc(esc: String): string {
  switch (esc[0]) {
    case "e": return "\x1b"
    case "n": return "\n"
    case "r": return "\r"
    case "t": return "\t"
    case "x": return x()
    case "u": return u()
    default: return ""
  }

  function x() {
    const code = parseInt(esc.slice(1), 16)

    try {
      return String.fromCodePoint(code)
    }
    catch {
      return ""
    }
  }

  function u(): string {
    // todo
    return ""
  }
}

// Copyright see AUTHORS & LICENSE; SPDX-License-Identifier: ISC+
