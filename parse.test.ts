import { expect } from '@esm-bundle/chai'

import { 
  parse, Parse, ParseError,
  STOP, END, CONTINUE, REQUIRED 
} from './parse.ts'

it('parse', () => {
  expect(parse("a:1")).to.deep.equal({ a: "1" })
  expect(parse("a: ()")).to.deep.equal({ a: [] })
  expect(parse(`"": ()`)).to.deep.equal({ "": [] })
  expect(parse("💩: 0")).to.deep.equal({"💩": "0" }) // astral emoji
  expect(parse("$: ()")).to.deep.equal({ "$": [] })
  expect(parse("ö:1")).to.deep.equal({ ö: "1" })

  const ch = "🇨🇭" // Switzerland flag
  expect([...ch].length).to.equal(2)
  expect(parse(ch + ": 1")).to.deep.equal({ [ch]: "1" })

  const family = "👨‍👩‍👧‍👦" // Family emoji with ZWJ
  expect([...family].length).to.equal(7)
  expect(parse(family + ": 4")).to.deep.equal({ [family]: "4" })
})

it('parse strings', () => {
  expect(parse(`a: ""`).a).to.equal("")
  expect(parse(`a: "x"`).a).to.equal("x")
  
})

// Delayed evaluation for error handling in expect()
const dparse = (s: string) => (() => parse(s))
const err = (s: string) => [ ParseError, new RegExp(`^${ s }$`) ] as any

it('parse errors', () => {
  expect(dparse(":")).to.throw(...err('invalid atom `:` at 1:1'))
  expect(dparse("\0:0")).to.throw(...err('invalid atom `‹00›` at 1:1'))
  // todo handle parse("") and parse(" ") (error empty document)
})

import { compileLexer } from './lex.ts'
import { lexer } from './lexer.ts'

const atto = { 
  parse, Parse, ParseError,
  STOP, END, CONTINUE, REQUIRED,
  compileLexer, lexer,
}

Object.assign(window, { atto })

// Copyright see AUTHORS; see LICENSE; SPDX-License-Identifier: ISC+

