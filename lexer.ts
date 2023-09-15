import { compileLexer, POP, type Spec, type Token } from './lex.ts'

// const _ = (rx: RegExp): RegExp => (console.log("rx: " + rx + ";"), rx)
const _ = (rx: RegExp): RegExp => rx

type TagFunction<T> = (strings: TemplateStringsArray, ...values: string[]) => T
export const rx: TagFunction<RegExp> = (raw, ...v) =>
  _(new RegExp(String.raw({ raw }, ...v), "vy"))

// Allowed codepoints in atto files: space, new-line, line-feed, tab and all
// codepoints in the categories: L, M, N, P, S and Cf. Cf is a special case:
// it containts ZWJ which iIs needed for some emojis, and soft hyphen and 
// bidi control characters, and others. All these make more or less sense in
// texts. Not allowed are codepoints in the categories Z and Cc, Cn, Co, Cs
// (except space, new-line, line-feed and tab) because they are too special,
// binary or not usable, for example thin space, non-breaking space, ASCII
// NUL, surrogates, non-assigned codepoints, and others. This choice is
// somewhat subjective, but a line needs to be drawn in the sand.

const raw = String.raw
 
const invalidCategories   = raw `\p{Cc}\p{Cn}\p{Co}\p{Cs}\p{Z}`
const ws                  = raw `\n\r\t `
const invalidBareChars    = raw `\(\)^"#:\\`
const invalidChars        = raw `[[${ invalidCategories }]&&[^${ ws }]]`
const notWs               = raw `[^${ invalidCategories }]`
const validChars          = raw `[${ notWs }[${ ws }]]`
const validCharsTabSpace  = raw `[${ notWs }[\t ]]`
const stringChars         = raw `[${ notWs }--["\\]]`
const guardedStringChars  = raw `[[^${ invalidCategories }"\\][${ ws }]]`
const hexDigit            = raw `[a-fA-F0-9]`  
const tabSpace            = raw `[ \t]`
const guardId             = raw `[_0-9]{0,9}`

export const rxBare = rx `[^${ invalidBareChars }${ invalidCategories }]+` 

const commonEscs: Spec[] = [
  [ 'xEsc',    rx `x${ hexDigit }{2}`,            POP ],
  [ 'invalid', rx `x.{1,2}`,                      POP ],
  [ 'uEsc',    rx `u${ hexDigit }{4}`,            POP ],
  [ 'invalid', rx `u.{1,4}`,                      POP ],
  [ 'uEsc2',   rx `u\\{${ hexDigit }{1,5}\\}`,    POP ],
  [ 'invalid', rx `u\\\{[^\\}]{1,5}`,             POP ],
  [ 'invalid', rx `.`,                            POP ],
]

const root: Spec[] = [
  [ 'ws',      rx `[${ ws }]+`                                            ],
  [ 'comment', rx `#+${ tabSpace }[${ tabSpace }${ validCharsTabSpace }]` ],
  [ 'bare',    rxBare                                                     ],
  [ '"',       rx `#${ guardId }"`, 'guardedStr', saveGuard               ],
  [ '"',                            'str' /* todo enforce ws bfore str */ ],
  [ ':'                                                                   ],
  [ '('                                                                   ],
  [ ')'                                                                   ],
  [ 'invalid', rx `[\\\\#]${ validChars  }{1,20}`                         ],
]
const str: Spec[] = [
  [ 'chars',                   rx `${ stringChars }+`     ],
  [ '\\',                                                 'esc' ], 
  [ '"',                                                 POP ],
  [ 'invalid',                 rx `"#`                       ],
]
const guardedStr: Spec [] = [
  [ '"',       rx `"#${ guardId }`, POP, checkGuard           ],
  [ 'chars',   rx `"`                                  ],
  [ '\\',      rx `\\\\#${ guardId }`, 'guardedEsc', checkGuard        ],
  [ 'chars',   rx `\\\\`                               ],
  [ 'chars',   rx `${ guardedStringChars }+`            ],
]
const esc: Spec[] =        [ [ 'esc', rx `["enrt0]`, POP ], ...commonEscs ]
const guardedEsc: Spec[] = [ [ 'esc', rx `["enrt0\n\r]`, POP ], ...commonEscs ]
const ALL: Spec[] =        [ [ 'invalid',  rx `${ invalidChars }+` ] ]

let guard: string = ""

function saveGuard(token: Token): Token | null {
  guard = token.text.slice(0, -1)
  return token
}

function checkGuard(token: Token): Token | null {
  return token.text.slice(1) === guard ? token : null
}

export const specs = { root, str, guardedStr, esc, guardedEsc, ALL }
export const lexer = compileLexer(specs)

// Copyright see AUTHORS; see LICENSE; SPDX-License-Identifier: ISC+

