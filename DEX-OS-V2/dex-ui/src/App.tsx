import { FormEvent, useEffect, useMemo, useState } from "react";
import { NotificationCenter } from "./components/NotificationCenter";
import { OrderFormValues, useDex } from "./hooks/useDex";
import "./styles/app.css";

const DEFAULT_ORDER_FORM: OrderFormValues = {
  baseToken: "ETH",
  quoteToken: "USDC",
  side: "buy",
  orderType: "limit",
  price: 1700,
  quantity: 1
};

const statusCopy = {
  idle: "Idle",
  connecting: "Connecting",
  ready: "API Ready",
  error: "API Error"
} as const;

const depthStatusCopy = {
  idle: "Idle",
  connecting: "Connecting",
  live: "Live",
  error: "Error"
} as const;

const currencyFormatter = new Intl.NumberFormat("en-US", {
  style: "currency",
  currency: "USD",
  maximumFractionDigits: 2
});

const numberFormatter = new Intl.NumberFormat("en-US", {
  maximumFractionDigits: 6
});

const EMPTY_VALUE = "\u2014";

const App = () => {
  const {
    status,
    apiBaseUrl,
    setApiBaseUrl,
    wallet,
    setTraderId,
    setAuthToken,
    connectWallet,
    market,
    refreshMarketData,
    depth,
    refreshDepth,
    trades,
    loadTrades,
    isRefreshingMarket,
    isFetchingTrades,
    isSubmittingOrder,
    isIssuingToken,
    isLoadingWasmWallet,
    submitOrder,
    lastOrderId,
    notifications,
    dismissNotification,
    clearNotifications,
    issueTokenWithSecret,
    issueTokenWithWallet,
    loadWasmWallet,
    issueTokenWithWasmWallet,
    wasmSession
  } = useDex();

  const [orderForm, setOrderForm] = useState<OrderFormValues>(DEFAULT_ORDER_FORM);
  const [apiUrlDraft, setApiUrlDraft] = useState(apiBaseUrl);
  const [sharedSecret, setSharedSecret] = useState("");
  const [secretTtl, setSecretTtl] = useState("900");
  const [walletTtl, setWalletTtl] = useState("900");
  const [wasmPrivateKey, setWasmPrivateKey] = useState("");
  const [wasmTtl, setWasmTtl] = useState("900");

  useEffect(() => {
    setApiUrlDraft(apiBaseUrl);
  }, [apiBaseUrl]);

  const tradesSorted = useMemo(
    () => [...trades].sort((a, b) => b.timestamp - a.timestamp),
    [trades]
  );

  const spread =
    typeof market.bestBid === "number" && typeof market.bestAsk === "number"
      ? market.bestAsk - market.bestBid
      : undefined;

  const canLoadTrades = Boolean(wallet.traderId && wallet.token);
  const depthSnapshot = depth.snapshot;
  const depthUpdatedAt = depthSnapshot
    ? new Date(depthSnapshot.timestamp * 1000).toLocaleTimeString()
    : undefined;
  const depthBids = depthSnapshot?.bids ?? [];
  const depthAsks = depthSnapshot?.asks ?? [];

  const parseTtl = (value: string) => {
    const trimmed = value.trim();
    if (!trimmed) {
      return undefined;
    }
    const parsed = Number(trimmed);
    return Number.isFinite(parsed) && parsed > 0 ? parsed : undefined;
  };

  const handleApiUrlSubmit = (event: FormEvent) => {
    event.preventDefault();
    setApiBaseUrl(apiUrlDraft);
  };

  const handleOrderSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    try {
      await submitOrder(orderForm);
    } catch {
      // errors surface through notification banner
    }
  };

  const handleSharedSecretToken = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    await issueTokenWithSecret(sharedSecret, parseTtl(secretTtl));
  };

  const handleWalletToken = async () => {
    await issueTokenWithWallet(parseTtl(walletTtl));
  };

  const handleWasmWalletLoad = async () => {
    await loadWasmWallet(wasmPrivateKey);
  };

  const handleWasmWalletToken = async () => {
    await issueTokenWithWasmWallet(wasmPrivateKey, parseTtl(wasmTtl));
  };

  const resetOrderForm = () => setOrderForm(DEFAULT_ORDER_FORM);

  return (
    <main className="app">
      <header className="app__header">
        <div>
          <p className="eyebrow">DEX-OS</p>
          <h1>Execution Console</h1>
          <p className="muted">
            Submit authenticated orders against the Rust orderbook and watch fills land in real
            time.
          </p>
        </div>
        <div className="app__status">
          <span data-state={status}>{statusCopy[status]}</span>
          <button
            type="button"
            className="ghost"
            onClick={() => refreshMarketData()}
            disabled={isRefreshingMarket}
          >
            {isRefreshingMarket ? "Refreshing..." : "Refresh market"}
          </button>
        </div>
      </header>

      <NotificationCenter
        notifications={notifications}
        onDismiss={dismissNotification}
        onClearAll={clearNotifications}
      />

      <section className="grid layout-two">
        <section className="card">
          <div className="card__header">
            <h2>Session & Wallet</h2>
            <button type="button" className="ghost" onClick={() => connectWallet()}>
              Connect wallet
            </button>
          </div>

          <form className="form-control" onSubmit={handleApiUrlSubmit}>
            <label htmlFor="api-url">API base URL</label>
            <div className="control-inline">
              <input
                id="api-url"
                value={apiUrlDraft}
                onChange={(event) => setApiUrlDraft(event.target.value)}
                placeholder="http://localhost:3030"
              />
              <button type="submit">Apply</button>
            </div>
          </form>

          <dl className="session-meta">
            <div>
              <dt>Wallet</dt>
              <dd>{wallet.address ?? "Not connected"}</dd>
            </div>
            <div>
              <dt>Trader ID</dt>
              <dd>{wallet.traderId ?? EMPTY_VALUE}</dd>
            </div>
            <div>
              <dt>Auth token</dt>
              <dd>{wallet.token ? "Configured" : "Missing"}</dd>
            </div>
          </dl>

          <label htmlFor="trader-id">Trader ID (JWT subject)</label>
          <input
            id="trader-id"
            value={wallet.traderId ?? ""}
            onChange={(event) => setTraderId(event.target.value)}
            placeholder="alice"
          />

          <label htmlFor="jwt-token">JWT token</label>
          <input
            id="jwt-token"
            type="password"
            value={wallet.token ?? ""}
            onChange={(event) => setAuthToken(event.target.value)}
            placeholder="Paste token from auth issuer"
          />

          <div className="token-tools">
            <h3>Auth helpers</h3>
            <form className="token-tools__secret" onSubmit={handleSharedSecretToken}>
              <label>
                Shared secret
                <input
                  type="password"
                  value={sharedSecret}
                  onChange={(event) => setSharedSecret(event.target.value)}
                  placeholder="Secret mapped to trader ID"
                />
              </label>
              <label>
                TTL (seconds)
                <input
                  type="number"
                  min="60"
                  step="60"
                  value={secretTtl}
                  onChange={(event) => setSecretTtl(event.target.value)}
                />
              </label>
              <button type="submit" disabled={isIssuingToken}>
                {isIssuingToken ? "Issuing..." : "Issue token via secret"}
              </button>
            </form>

            <div className="token-tools__wallet">
              <div>
                <label>
                  TTL (seconds)
                  <input
                    type="number"
                    min="60"
                    step="60"
                    value={walletTtl}
                    onChange={(event) => setWalletTtl(event.target.value)}
                  />
                </label>
                <p className="helper">
                  Request a challenge, sign it in MetaMask, and we'll mint a JWT scoped to your
                  wallet address.
                </p>
              </div>
              <button type="button" onClick={handleWalletToken} disabled={isIssuingToken}>
                {isIssuingToken ? "Awaiting signature..." : "Issue token via wallet"}
              </button>
            </div>

            <div className="token-tools__wallet">
              <div>
                <label>
                  WASM private key (hex)
                  <input
                    type="password"
                    value={wasmPrivateKey}
                    onChange={(event) => setWasmPrivateKey(event.target.value)}
                    placeholder="0x..."
                  />
                </label>
                <label>
                  TTL (seconds)
                  <input
                    type="number"
                    min="60"
                    step="60"
                    value={wasmTtl}
                    onChange={(event) => setWasmTtl(event.target.value)}
                  />
                </label>
                <p className="helper">
                  Use the WASM signer when no injected wallet is available. Keys are never persisted;
                  signing happens entirely in the WASM module.
                </p>
                {wasmSession && (
                  <p className="muted">
                    WASM session token: {wasmSession.token.slice(0, 10)}… (exp{" "}
                    {new Date(wasmSession.expires_at * 1000).toLocaleTimeString()})
                  </p>
                )}
              </div>
              <div className="token-tools__wallet-actions">
                <button
                  type="button"
                  onClick={handleWasmWalletLoad}
                  disabled={isLoadingWasmWallet || isIssuingToken}
                >
                  {isLoadingWasmWallet ? "Loading..." : "Load WASM wallet"}
                </button>
                <button
                  type="button"
                  onClick={handleWasmWalletToken}
                  disabled={isIssuingToken || isLoadingWasmWallet}
                >
                  {isIssuingToken ? "Awaiting signature..." : "Issue token via WASM"}
                </button>
              </div>
            </div>
          </div>
        </section>

        <section className="card order-card">
          <div className="card__header">
            <h2>Submit Order</h2>
            <button type="button" className="ghost" onClick={resetOrderForm}>
              Reset
            </button>
          </div>

          <form className="form-grid" onSubmit={handleOrderSubmit}>
            <label>
              Base token
              <input
                value={orderForm.baseToken}
                onChange={(event) =>
                  setOrderForm((prev) => ({ ...prev, baseToken: event.target.value }))
                }
                required
              />
            </label>

            <label>
              Quote token
              <input
                value={orderForm.quoteToken}
                onChange={(event) =>
                  setOrderForm((prev) => ({ ...prev, quoteToken: event.target.value }))
                }
                required
              />
            </label>

            <label>
              Side
              <select
                value={orderForm.side}
                onChange={(event) =>
                  setOrderForm((prev) => ({ ...prev, side: event.target.value as OrderFormValues["side"] }))
                }
              >
                <option value="buy">Buy</option>
                <option value="sell">Sell</option>
              </select>
            </label>

            <label>
              Order type
              <select
                value={orderForm.orderType}
                onChange={(event) =>
                  setOrderForm((prev) => ({
                    ...prev,
                    orderType: event.target.value as OrderFormValues["orderType"],
                    price: event.target.value === "market" ? undefined : prev.price ?? 0
                  }))
                }
              >
                <option value="limit">Limit</option>
                <option value="market">Market</option>
              </select>
            </label>

            {orderForm.orderType === "limit" && (
              <label>
                Price (quote units)
                <input
                  type="number"
                  min="1"
                  step="1"
                  value={orderForm.price ?? ""}
                  onChange={(event) =>
                    setOrderForm((prev) => ({
                      ...prev,
                      price: Number(event.target.value)
                    }))
                  }
                  required
                />
              </label>
            )}

            <label>
              Quantity (base units)
              <input
                type="number"
                min="1"
                step="1"
                value={orderForm.quantity}
                onChange={(event) =>
                  setOrderForm((prev) => ({
                    ...prev,
                    quantity: Number(event.target.value)
                  }))
                }
                required
              />
            </label>

            <p className="helper">
              Prices and quantities are unsigned integers; normalize to smallest token units before
              submitting.
            </p>

            <button type="submit" disabled={isSubmittingOrder}>
              {isSubmittingOrder ? "Submitting..." : "Send order"}
            </button>

            {lastOrderId && (
              <p className="muted">Last accepted order ID: #{lastOrderId.toLocaleString()}</p>
            )}
          </form>
        </section>
      </section>

      <section className="grid layout-two">
        <section className="card market-card">
          <div className="card__header">
            <h2>Orderbook Snapshot</h2>
            <span className="pill">{market.isLoading ? "Updating..." : "Live"}</span>
          </div>
          <div className="metrics metrics--dense">
            <div>
              <p>Best bid</p>
              <strong>
                {typeof market.bestBid === "number"
                  ? currencyFormatter.format(market.bestBid)
                  : EMPTY_VALUE}
              </strong>
            </div>
            <div>
              <p>Best ask</p>
              <strong>
                {typeof market.bestAsk === "number"
                  ? currencyFormatter.format(market.bestAsk)
                  : EMPTY_VALUE}
              </strong>
            </div>
            <div>
              <p>Spread</p>
              <strong>{typeof spread === "number" ? currencyFormatter.format(spread) : EMPTY_VALUE}</strong>
            </div>
            <div>
              <p>Last update</p>
              <strong>
                {market.lastUpdated
                  ? new Date(market.lastUpdated).toLocaleTimeString()
                  : EMPTY_VALUE}
              </strong>
            </div>
          </div>
          <p className="muted">
            Source: REST endpoint <code>/orderbook/prices</code> on {apiBaseUrl}.
          </p>
        </section>

        <section className="card depth-card">
          <div className="card__header">
            <h2>Orderbook Depth (Top 10)</h2>
            <button
              type="button"
              className="ghost"
              onClick={() => refreshDepth()}
              disabled={depth.isLoading}
            >
              {depth.isLoading ? "Refreshing..." : "Refresh"}
            </button>
          </div>
          <div className="depth-status">
            <span className={`pill pill--${depth.connection}`}>{depthStatusCopy[depth.connection]}</span>
            {depth.lastError && <span className="warn">{depth.lastError}</span>}
            {depthUpdatedAt && <span className="muted">Updated {depthUpdatedAt}</span>}
          </div>
          {depthSnapshot ? (
            <div className="depth-grid">
              <div>
                <h3>Bids</h3>
                <table className="depth-table">
                  <thead>
                    <tr>
                      <th>Price</th>
                      <th>Qty</th>
                    </tr>
                  </thead>
                  <tbody>
                    {depthBids.map((level, idx) => (
                      <tr key={`bid-${idx}`}>
                        <td>{currencyFormatter.format(level.price)}</td>
                        <td>{numberFormatter.format(level.quantity)}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
              <div>
                <h3>Asks</h3>
                <table className="depth-table">
                  <thead>
                    <tr>
                      <th>Price</th>
                      <th>Qty</th>
                    </tr>
                  </thead>
                  <tbody>
                    {depthAsks.map((level, idx) => (
                      <tr key={`ask-${idx}`}>
                        <td>{currencyFormatter.format(level.price)}</td>
                        <td>{numberFormatter.format(level.quantity)}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          ) : (
            <p className="muted">Waiting for depth data...</p>
          )}
        </section>

        <section className="card trades-card">
          <div className="card__header">
            <h2>Recent Trades</h2>
            <button
              type="button"
              className="ghost"
              onClick={() => loadTrades()}
              disabled={!canLoadTrades || isFetchingTrades}
            >
              {!canLoadTrades
                ? "Auth required"
                : isFetchingTrades
                ? "Loading..."
                : "Sync trades"}
            </button>
          </div>

          {!canLoadTrades && (
            <p className="muted">
              Connect a wallet / trader ID and JWT token to pull fills scoped to that identity.
            </p>
          )}

          {canLoadTrades && tradesSorted.length === 0 && (
            <p className="muted">No trades recorded for this trader yet.</p>
          )}

          {canLoadTrades && tradesSorted.length > 0 && (
            <div className="table-wrapper">
              <table>
                <thead>
                  <tr>
                    <th>ID</th>
                    <th>Pair</th>
                    <th>Price</th>
                    <th>Quantity</th>
                    <th>Maker / Taker</th>
                    <th>Time</th>
                  </tr>
                </thead>
                <tbody>
                  {tradesSorted.map((trade) => (
                    <tr key={trade.id}>
                      <td>#{trade.id}</td>
                      <td>
                        {trade.base_token}/{trade.quote_token}
                      </td>
                      <td>{currencyFormatter.format(trade.price)}</td>
                      <td>{numberFormatter.format(trade.quantity)}</td>
                      <td>
                        #{trade.maker_order_id} / #{trade.taker_order_id}
                      </td>
                      <td>{new Date(trade.timestamp * 1000).toLocaleTimeString()}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </section>
      </section>
    </main>
  );
};

export default App;
