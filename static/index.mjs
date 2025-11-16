import { loadIcons } from './icons.mjs';
import { startWebAssembly } from './wasm.mjs';

await loadIcons();
await startWebAssembly();
