// Because binary armor is not converted to ArrayBuffer we define
// Atom = string and avoid using Map<Atom, Value>.

export type Atom = string

export type Value = Atom | Document | List

export interface Document extends Record<Atom, Value> { }

export type Entry = { key: Atom, value: Value }

export type List = Value[]

// Copyright see AUTHORS; see LICENSE; SPDX-License-Identifier: ISC+

