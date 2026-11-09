let caught = 0;
for (const path of ['./invalid.mjs', './invalid-attribute.mjs']) {
  try { await import(path); } catch (error) {
    if (!(error instanceof SyntaxError)) throw error;
    caught++;
  }
}
try { await import('./absent.mjs', { with: { type: 'css' } }); } catch (error) {
  if (!(error instanceof TypeError)) throw error;
  caught++;
}
globalThis.importMarker = {};
for (let i = 0; i < 2; i++) {
  try { await import('./throwing.mjs'); } catch (error) {
    if (error !== globalThis.importMarker) throw error;
    caught++;
  }
}
const good = await import('./shared.mjs');
if (caught !== 5 || good.value !== 21 || globalThis.throwingLoads !== 1) throw 7;
