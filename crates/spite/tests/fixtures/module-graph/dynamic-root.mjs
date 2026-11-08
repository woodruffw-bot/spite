import { value } from './shared.mjs';
const [left, right] = await Promise.all([import('./root.mjs'), import('././root.mjs')]);
const bridge = await import('./nested/bridge.mjs');
if (value !== 21 || left !== right || left.readRoot() !== 42 || bridge.value !== 21) throw 7;
if (globalThis.sharedLoads !== 1 || globalThis.rootLoads !== 1) throw 8;
