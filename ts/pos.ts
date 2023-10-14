// Copied from leac; © by KillyMXI (MIT license) and modified

const linebreaksRe = /\n/g;

export type Position = {
  line: number;
  column: number;
}

/**
 * Create a function that will return a position
 * (line and column number) based on offset (index)
 * within the string.
 *
 * Matches all linebreaks (`\n`) on creation and then
 * uses a binary tree to compute the position on each call.
 *
 * @param str - Input string.
 */
export function createPosQuery (
  str: string,
): (offset: number) => Position {
  const offsets = [...str.matchAll(linebreaksRe)].map(m => m.index || 0);
  offsets.unshift(-1);
  const tree = createOffsetTree(offsets, 0, offsets.length);
  return (offset: number) => queryPosition(tree, offset);
}


type BranchNode = {
  offset: number;
  low: Node;
  high: Node;
}

type LeafNode = {
  offset: number;
  index: number;
}

type Node = BranchNode | LeafNode;

function createOffsetTree (offsets: number[], i: number, j: number): Node {
  if (j - i === 1) {
    // DL: strict type checking discovered that offset might be undefined
    // I don't know positionQuery well enough to find a static typecheck
    // so I added a run-time assert instead
    const offset = offsets[i]
    if (offset === undefined) throw new Error("internal error")
    return { offset, index: i + 1 };
  }
  const k = Math.ceil((i + j) / 2);
  const low = createOffsetTree(offsets, i, k);
  const high = createOffsetTree(offsets, k, j);
  return {
    offset: low.offset,
    low: low,
    high: high
  };
}

function queryPosition (node: Node, offset: number): Position {
  return (isLeafNode(node))
    ? { line: node.index, column: offset - node.offset }
    : queryPosition(
      (node.high.offset < offset) ? node.high : node.low,
      offset
    );
}

function isLeafNode (node: Node): node is LeafNode {
  return Object.prototype.hasOwnProperty.call(node, 'index');
}

// Copyright see AUTHORS; see LICENSE; SPDX-License-Identifier: ISC+
