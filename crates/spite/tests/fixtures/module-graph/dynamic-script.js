const checked = Promise.all([
  import('./root.mjs'),
  import('././root.mjs'),
  import('./nested/bridge.mjs')
]).then(([left, right, bridge]) => {
  if (left !== right || left.readRoot() !== 42 || bridge.value !== 21) throw 7;
  if (globalThis.sharedLoads !== 1 || globalThis.rootLoads !== 1) throw 8;
  return 42;
});
// Throwing capability functions make the async result visible to this CLI test
// through Reported, rather than the host's no-op rejection tracker.
checked.constructor = {
  [Symbol.species]: class {
    constructor(executor) {
      executor(
        value => { throw new Error('fulfilled ' + value); },
        reason => { throw new Error('rejected ' + reason); }
      );
    }
  }
};
checked.then(value => value);
7;
