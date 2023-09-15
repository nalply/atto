# Atto Document Language

The Atto Document Language (atto) is untyped and minimal. It has atoms,
values, lists and documents and also comments.

An atom is a bare word, a string or a guarded string.

```
bare-words: ( name true 42 null )
string: "Hello, world!"
guarded-string: #"He said: "Hello!" and I nodded."#
```

A value is an atom or a document or a list.

```
atom: example
list: ( example )
document: ( key: example )
```

A list consists of values enclosed by parentheses. A list can be empty.

```
list: ( example ( example ) ( key: example) )
empty-list: ()
```

A Document consist of one or more entries enclosed by parentheses¹. An entry
has an atom key and a value.

<sub>¹except for the root document</sub>

```
name: "John Doe"
age: 42
```

Comments start with hash and one white-space.

```
# This is a comment
```

That's all, folks.

## Grammar

```
Document := ( Atom ":" Value )+
Value    := Atom | "(" List | Document ")"
List     := Value*
```

## Status

TypeScript POC in development with `@web/test-runner` without bundling

Thanks to KillyMXI for inspiring me with his TypeScript lexer
[leac](https://github.com/mxxii/leac/blob/main/docs/index.md). I copied
his line-column query virtually unchanged (positionQuery.ts).

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