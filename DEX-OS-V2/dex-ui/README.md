# DEX-OS Web UI

This package delivers the production-facing React console for DEX-OS. It now ships with:

- Wallet session & JWT management so you can point the UI at any running `dex-api`.
- A real order form wired to `POST /orderbook/orders`.
- Live market snapshots from `GET /orderbook/prices`.
- Authenticated trade history pulls scoped to the connected trader ID.
- Real-time orderbook depth from the new `/ws/depth` WebSocket feed (with `/orderbook/depth` for manual sync).

## Prerequisites

- Node.js 18+
- npm 9+ (or pnpm/yarn if you prefer)
- `wasm-pack` (install with `cargo install wasm-pack --locked`)

## Quick Start

```bash
# from the repository root
cd dex-ui
npm install
# optional: export base URL if not using localhost
echo "VITE_API_BASE_URL=http://localhost:3030" > .env.local
npm run wasm:build  # compiles ../dex-wasm into src/wasm-pkg
npm run dev         # launches Vite on http://localhost:5173
```

To produce an optimized build:

```bash
npm run build
npm run preview
```

## Project Layout

- `src/api/` exposes a thin client + shared types for `dex-api`.
- `src/hooks/useDex.ts` orchestrates wallet status, JWT/session persistence, REST calls, and order submission.
- `src/App.tsx` renders the execution console (wallet panel, order form, market data, trades).
- `src/wasmBridge.ts` still lazy-loads the wasm-bindgen bundle for future advanced interactions.
- `src/wasm-pkg/` is populated by `npm run wasm:build`; the directory is git-ignored.
- `public/index.html` bootstraps the Vite client and serves static assets.

## Connecting to the API

1. Start `dex-api` locally (`cargo run -p dex-api`) so the REST endpoints are reachable.
2. Export `VITE_API_BASE_URL` or set it inside the UI under **Session & Wallet** if you are not on `http://localhost:3030`.
3. Obtain a JWT for your trader (see backend docs) and paste it into the UI. The subject (`sub`) value must match the trader ID you submit in orders.
4. Click **Connect wallet** to populate the trader ID from MetaMask, or type any identifier manually if you are not on an EVM wallet.

Once the JWT and trader ID are present you can:

- Submit authenticated limit/market orders.
- Refresh the market snapshot on demand.
- Pull trade history for the trader currently signed in.

## Wallet & JWT handling

- The wallet button requests `eth_requestAccounts` from any injected provider (MetaMask, Rabby, etc.) and stores the address locally.
- Session details (API base URL, trader ID, token) persist in `localStorage` so you do not lose them between refreshes.
- The UI never transmits the JWT anywhere except the direct API calls initiated from the browser; clear the token field if you need to revoke access.
- Use the "Auth helpers" panel to mint tokens either with `POST /auth/token/shared` (shared secret) or via the signature-based `/auth/token/wallet` flow and `/auth/challenge`.

## Regenerating the WASM Bundle

`npm run wasm:build` calls:

```bash
wasm-pack build ../dex-wasm --target web --out-dir src/wasm-pkg --release
```

Run this after changes inside `dex-wasm/`. Remove the `src/wasm-pkg` directory if you want to force a rebuild.

## UI Capabilities & Next Steps

- Order form enforces the same validation rules as the Rust API (unsigned integers, distinct tokens, etc.).
- Market data auto-refreshes every 15 seconds while also allowing manual refreshes.
- Depth card subscribes to `/ws/depth?levels=10` and falls back to the REST endpoint for manual refreshes.
- Trade history requires JWT authentication and stays scoped to the current trader ID.
- To extend the experience, add charting, depth books, or integrate additional REST endpoints inside `src/api/client.ts`.
