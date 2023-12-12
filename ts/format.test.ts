import { expect } from '@esm-bundle/chai'

import { format } from './format.ts'

function fmt(doc: any): string {
  const s = format(doc, { style: 'compact' })
  if (!s.startsWith("# atto\n"))
    throw new Error("bad or no header (# atto)")
  return s.substring(7)
}
  
/*
it('format', () => {
  expect(fmt("1")).to.equal("1")
  expect(fmt(1)).to.equal("1")
  expect(fmt("\x00")).to.equal('"\\x00"')

  // poop emoji can be bare
  expect(fmt("\u{1f4a9}")).to.equal("\u{1f4a9}")

  // the last codepoint 10ffff is not bare
  expect(fmt("\u{10ffff}")).to.equal('"\\u{10ffff}"')

  expect(fmt(true)).to.equal("true")
  expect(fmt(false)).to.equal("false")
  expect(fmt([1, "a"])).to.equal("1 a")
  expect(fmt([1, [ 2 ] ])).to.equal("1 (2)")
  expect(fmt({ a: 2, b: "c" })).to.equal("a: 2 b: c")
})
*/

it('format errors', () => {
  expect(() => fmt(Symbol())).to.throw()
})

// Copyright see AUTHORS & LICENSE; SPDX-License-Identifier: ISC+
