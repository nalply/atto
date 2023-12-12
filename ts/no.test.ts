import { expect } from '@esm-bundle/chai'

import { compileLexer, type Token } from './lex.ts'
import { lexer } from './lexer.ts'
import { format } from './format.ts'
import { parse } from './parse.ts'

function asText2(text: string, dbg: boolean = false): string {
  function short(token: Token): string {
    const { name, group, text } = token
    return group + "-" + name + (name == text ? "" : ":" + text)
  }

  const tokenizer = lexer.tokenizerBuilder(text, dbg)
  return [...tokenizer].map(short).slice(0, -1).join(';') + ';'
}

Object.assign(lexer, { asText2 })
const atto = { compileLexer, lexer, format, parse }
Object.assign(window, { atto, expect })

// Copyright see AUTHORS & LICENSE; SPDX-License-Identifier: ISC+
