import { expect } from '@esm-bundle/chai'
import { compileLexer, type Token } from './lex.ts'

import { lexer } from './lexer.ts'

function test(text: string, dbg: boolean = false): string {
  function short(token: Token): string {
    const { name, text } = token
    return name + (name == text ? "" : ":" + text)
  }
  
  const tokenizer = lexer.tokenizerBuilder(text, dbg)
  return [...tokenizer].map(short).slice(0, -1).join(';') + ';'
}

Object.assign(lexer, { test })
const atto = { compileLexer, lexer }
Object.assign(window, { atto })

it('lex', () => {
  expect(test("a")).to.equal("bare:a;")
  expect(test(" ")).to.equal("ws: ;")
  expect(test("\0")).to.equal("invalid:\0;")
  expect(test("# a")).to.equal("comment:# a;")
  expect(test("##\t.")).to.equal("comment:##\t.;")
  expect(test(":")).to.equal(":;")
  expect(test("(")).to.equal("(;")
  expect(test(")")).to.equal(");")
  expect(test('"')).to.equal("\";")
  expect(test("\\a")).to.equal("invalid:\\a;")
})

it('lex strings', () => {
  expect(test(`""\0`)).to.equal(`";";invalid:\0;`)
  expect(test(`"ax" `)).to.equal(`";chars:ax;";ws: ;`)
  expect(test(`"text"#x`)).to.equal(`";chars:text;";invalid:#x;`)
  expect(test(`"\\e"x`)).to.equal(`";\\;esc:e;";bare:x;`)
  expect(test(`"\\x0f"`)).to.equal(`";\\;xEsc:x0f;";`)
  expect(test(`"\\""`)).to.equal(`";\\;esc:";";`)
  expect(test(`"\\a`)).to.equal(`";\\;invalid:a;`)
})

it('lex guarded strings', () => {
  expect(test(`#""#`)).to.equal(`":#";":"#;`)
  expect(test(`#"some text"#`)).to.equal(`":#";chars:some text;":"#;`)
  expect(test(`#"\\"#`)).to.equal(`":#";chars:\\;":"#;`)
  expect(test(`#"\\#e"#`)).to.equal(`":#";\\:\\#;esc:e;":"#;`)

})

// Copyright see AUTHORS; see LICENSE; SPDX-License-Identifier: ISC+

