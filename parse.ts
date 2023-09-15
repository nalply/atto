import type { Document, List, Entry, Atom, Value } from "./types.ts"
import type { Token, Tokenizer } from "./lex.ts"
import { strEsc } from "./lex.ts"
import { lexer } from "./lexer.ts"

export function parse(text: string, dbg: boolean = false): Document {
  const tokenizer = lexer.tokenizerBuilder(text)
  lexer.dbg = dbg

  const parse = new Parse(tokenizer)
  const result = parse.document()
  if (result === STOP) throw new ParseError(parse.errors.join("\n"))

  return result
}

export class ParseError extends Error {
  constructor(msg: string) { super(msg) }
}

export const STOP = Symbol('STOP')
export const CONTINUE = Symbol('CONTINUE')
export const END = Symbol('END')
export const REQUIRED = Symbol('REQUIRED')

type STOP = typeof STOP
type CONTINUE = typeof CONTINUE
type END = typeof END
type REQUIRED = typeof REQUIRED

export class Parse {
  line: number = 0
  column: number = 0
  readonly held: Token[] = []
  readonly tokenizer: Tokenizer
  readonly errors: string[] = []

  constructor(tokenizer: Tokenizer) {
    this.tokenizer = tokenizer
  }

  next(): Token {
    const token = this.held.pop() ?? this.tokenizer.nextToken()
    this.line = token.line
    this.column = token.column
    return token
  }

  back<T>(token: Token, result: T | undefined = undefined): T {
    this.held.push(token)
    return result as T
  }

  error(error: string): STOP {
    const location = " at " + this.line + ":" + this.column
    this.errors.push(error + location)
    return STOP
  }
  
  document(firstKey?: Atom): Document | STOP {
    this.ws()
    const firstEntry = this.entry(firstKey)
    if (firstEntry == STOP) return STOP
    
    const document = { [firstEntry.key]: firstEntry.value }
    while (true) {
      const ws = this.ws(REQUIRED)
      if (ws == END) return document
      if (ws == STOP) return this.error("no whitespace between entries")
       
      const entry = this.entry()
      if (entry === STOP) return STOP
      document[entry.key] = entry.value
    }
  }

  ws(opt?: REQUIRED): END | STOP | void {
    let token = this.next()
    if (token.name === ')') return END
    if (token.name === 'FINAL') return this.back(token, END)
    if (token.name !== 'ws')
      return opt === REQUIRED ? STOP : this.back(token)

    while (true) {
      token = this.next()
      if (token.name !== 'ws' && token.name !== 'comment') {
        if (token.name === ')') return END
        this.back(token)
        break
      }
    }
  }

  entry(key?: Atom | STOP): Entry | STOP {
    if (key == null) {
      const atom = this.atom()
      if (atom === CONTINUE) // TS didn't catch that CONTINUE isn't returned
        return this.error("internal error") // so let's guard by this error
      key = atom
    }
    if (key === STOP) return STOP

    this.ws()
    const token = this.next()
    if (token.name !== ":")
      return this.error("no colon between key and value")
    this.ws()

    const value = this.value()
    if (value === STOP || value === END)
      return this.error("no value for entry")

    return { key, value }
  }

  atom(opt?: CONTINUE): Atom | STOP | CONTINUE {
    let token = this.next()

    if (token.name === "bare") return token.text
    if (token.text === `"`) return this.str()
    if (token.name === `#`) return this.guardStr()
    
    if (token.name === 'FINAL') return this.error("unexpected end of text")

    if (opt === CONTINUE) {
      this.back(token)
      return CONTINUE
    }

    const escOpt = "guillemet"
    return this.error("invalid atom `" + strEsc(token.text, escOpt) + "`")
  }

  value(): Value | STOP | END {
    const atom = this.atom(CONTINUE)
    if (atom !== CONTINUE) return atom // Atom | STOP

    const token = this.next()
    if (token.name === "(") {
      const first = this.value()
      if (first === STOP) return STOP
      if (first === END) return [] // ")" follows immediately == emtpy list

      const optColon = this.next()
      if (optColon.name === ":") return this.document(first as Atom)

      this.back(optColon)
      return this.list(first)
    }

    if (token.name === ")")
      return END

    return this.error("invalid value `" + token.text + "`")
  }

  str(): Atom | STOP {
    let atom = ""
    while (true) {
      const token = this.next()

      switch (token.name) {
        case "FINAL": return this.error("unexpected end of text in string")
        case `"`: return atom
        case "chars": atom += token.text; break
        case `\\`: break
        case "esc": atom += unesc(token.text); break
        default: return this.error("internal error, unexpected " + token.name)
      }
    }
  }

  guardStr(): Atom | STOP {
    return this.error("unimplemented")
  }

  list(firstValue: Value): List | STOP {
    const list = [ firstValue ]
        
    while (true) {
      const ws = this.ws(REQUIRED)  
      if (ws === END) return list
      if (ws == STOP) return this.error("no whitespace between values")
       
      const value = this.value()
      if (value === END) return list
      if (value === STOP) return STOP

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

// Copyright see AUTHORS; see LICENSE; SPDX-License-Identifier: ISC+
