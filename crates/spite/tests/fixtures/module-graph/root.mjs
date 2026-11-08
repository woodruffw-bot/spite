import { left } from './left.mjs';
import { right } from './right.mjs';
import { cycle } from './cycle.mjs';
export function readRoot() { return left + right; }
globalThis.rootLoads = (globalThis.rootLoads || 0) + 1;
if (left !== 21 || right !== 21 || globalThis.sharedLoads !== 1 || globalThis.rootLoads !== 1 || cycle() !== 42) throw 7;
