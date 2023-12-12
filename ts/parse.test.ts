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

// s`text ${0}` is same as `text ${0}`
function s(strings: TemplateStringsArray, ...values: any[]) {
  return strings.reduce(
    (acc, curr, i) => acc += curr + values[i - 1]
  )
}

// Usage: expectErr <text> <err> where <text> and <err> are template strings
// Example: expectErr `:` `invalid atom : at \\d:\\d`
// Task: parse <text> & expect ParseError(<err>), <err> supports regex syntax
function expectErr(textParts: TemplateStringsArray, ...textValues: any[]) {
  const text = s(textParts, textValues)
  return function(errParts: TemplateStringsArray, ...errValues: any[]) {
    const err = s(errParts, errValues)
    expect(() => parse(text)).to.throw(ParseError, new RegExp(`^${ err }$`))
  }
}

it('parse errors', () => {
  expectErr ``        `unexpected end of text at 1:1`
  expectErr ` `       `unexpected end of text at 1:2`
  expectErr `:`       `invalid atom : at 1:1`
  expectErr `a`       `unexpected end of text at 1:2`
  expectErr `a(`      `no colon after key at 1:2`
  expectErr `a"a"`    `no colon after key at 1:2` // todo
  expectErr `\0:0`    `invalid atom ‹00› at 1:1`
  expectErr `a:(a())` `no whitespace between values at 1:5`
  expectErr `a:(a (`  `unexpected end of text at 1:7`
  expectErr `a:(a ()` `unexpected end of text at 1:8`
})

import { compileLexer } from './lex.ts'
import { lexer } from './lexer.ts'

const atto = { 
  parse, Parse, ParseError,
  STOP, END, CONTINUE, REQUIRED,
  compileLexer, lexer,
}

Object.assign(window, { atto, expectErr })

// Copyright see AUTHORS & LICENSE; SPDX-License-Identifier: ISC+
