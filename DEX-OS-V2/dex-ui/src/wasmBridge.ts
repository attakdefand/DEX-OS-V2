export type WasmBindings = typeof import("./wasm-pkg/dex_wasm.js");
export type WasmOrderBookClass = WasmBindings["WasmOrderBook"];
export type WasmAMMClass = WasmBindings["WasmAMM"];
export type WasmWalletSignerClass = WasmBindings["WasmWalletSigner"];
export type WasmSessionManagerClass = WasmBindings["WasmSessionManager"];
export type WasmOrderBook = InstanceType<WasmOrderBookClass>;
export type WasmAMM = InstanceType<WasmAMMClass>;
export type WasmWalletSigner = InstanceType<WasmWalletSignerClass>;
export type WasmSessionManager = InstanceType<WasmSessionManagerClass>;
export type WasmOrder = Parameters<WasmOrderBook["add_order"]>[0];
export type WasmTrade = ReturnType<WasmOrderBook["add_order"]>[number];
export interface WasmSession {
  address: string;
  token: string;
  expires_at: number;
}

let wasmModule: WasmBindings | null = null;

/**
 * Loads the generated wasm-bindgen bundle on demand.
 * This helper caches the module so React components can reuse it safely.
 */
export async function loadWasm(): Promise<WasmBindings> {
  if (wasmModule) {
    return wasmModule;
  }

  const mod = await import("./wasm-pkg/dex_wasm.js");
  // wasm-pack's default export initializes the wasm binary
  if (typeof mod.default === "function") {
    await mod.default();
  }

  wasmModule = mod;
  return mod;
}
