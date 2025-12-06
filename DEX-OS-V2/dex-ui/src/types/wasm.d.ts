declare module "./wasm-pkg/dex_wasm.js" {
  export default function init(
    input?: RequestInfo | URL | Response | BufferSource | WebAssembly.Module
  ): Promise<any>;

  export type TradingPair = {
    base: string;
    quote: string;
  };

  export type OrderSide = "Buy" | "Sell";
  export type OrderType = "Limit" | "Market";

  export interface Order {
    id: number;
    trader_id: string;
    pair: TradingPair;
    side: OrderSide;
    order_type: OrderType;
    price?: number;
    quantity: number;
    timestamp: number;
  }

  export interface Trade {
    id: number;
    maker_order_id: number;
    taker_order_id: number;
    base_token: string;
    quote_token: string;
    price: number;
    quantity: number;
    timestamp: number;
  }

  export class WasmOrderBook {
    constructor();
    add_order(order: Order): Trade[];
    best_bid(): number | undefined;
    best_ask(): number | undefined;
    get_order(order_id: number): Order;
  }

  export class WasmAMM {
    constructor(fee: number);
    add_liquidity(
      token_a: string,
      amount_a: number,
      token_b: string,
      amount_b: number
    ): number;
    get_price(from_token: string, to_token: string): number;
    swap(from_token: string, to_token: string, amount_in: number): number;
  }
}
