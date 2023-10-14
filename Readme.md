# atto

## Overview

atto is an untyped and simple document/configuration language with atoms,
lists and documents.

An **atom** is a bare word, a string or a guarded string.

```
bare-words: ( name true 42 null )
string: "Hello, world!"
guarded-string: #"He said: "Hello!" and I nodded."#
```

A **list** consists of values enclosed by parentheses. A list can be empty.

```
list: ( example ( example ) ( key: example) )
empty-list: ()
```

A **document** consist of one or more entries enclosed by parentheses¹. An
entry has an atom key and a value.

```
name: "John Doe"
age: 42
```

<sub>¹except for the root document: can be empty and has no parentheses</sub>

## Grammar

```
Document := ( Atom ":" Value )+
Value    := Atom | "(" List | Document ")"
List     := Value*
```

## Rationale

Data in software always has type. Sometimes the type is implicit or an union,
but the type does exist. One example: JSON does not support the `Date` type.
This is not a problem: people just use ISO 8601. This looks like this:
`"date": "2023-10-14T15:06:05Z"`. And why is this not misinterpreted as a
string? Because the software knows that the data target is of type `Date`.

This let me to the conclusion that a type system is mostly unneccessary. And
atto was born.

## Status

TypeScript POC in development with `@web/test-runner` without bundling,
sources in subdirectory ts. Currently not everything implemented, especially
guarded strings. Tests but without coverage.

Thanks to KillyMXI for inspiring me with his TypeScript lexer
[leac](https://github.com/mxxii/leac/blob/main/docs/index.md). With his
permission I copied his line-column query virtually unchanged
(`positionQuery.ts`).

Rust POC in development, sources in subdirectory rust. A lot is missing yet.

## Examples

```
name: atto
type: module
version: 0.1.1
description: "atto - a simple document language"
license: ISC
devDependencies: (
  @esm-bundle/chai: ^4.3.4-fix.0
  @web/dev-server: 0
  @web/dev-server-esbuild: ^0.4.1
  @web/test-runner: ^0.16.1
  typescript: 5
  @web/test-runner-playwright: ^0.10.1
)
author: "see AUTHORS"
dependencies: ()

# Copyright see AUTHORS; see LICENSE; SPDX-License-Identifier: ISC+
```

```
squadName: "Super hero squad"
homeTown: "Metro City"
members: (
  ( name: Sandman age: 53 powers: (Sandstorm "Magic carpet") )
  ( name: "Molecule Man" age: 29
    powers: ("Radiation resistance" "Turning tiny" "Radiation blast")
  )
  ( name: "Madame Very Large Uppercut"
    age: 39
    powers: ("Million tonne punch" "Damage resistance" "Superhuman reflexes")
  )
)
```

<sub>Copyright see AUTHORS; see LICENSE; SPDX-License-Identifier: ISC+</sub>