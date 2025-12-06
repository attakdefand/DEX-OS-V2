import { DEFAULT_API_BASE_URL } from "../config";
import {
  ApiAbstainRequest,
  ApiCreateOrderRequest,
  ApiCreateOrderResponse,
  ApiCreateProposalRequest,
  ApiCreateProposalResponse,
  ApiDepthSnapshot,
  ApiGetTradesResponse,
  ApiProposal,
  ApiPriceResponse,
  ApiSharedTokenRequest,
  ApiTokenResponse,
  ApiVoteRequest,
  ApiWalletChallengeResponse,
  ApiWalletTokenRequest
} from "./types";

interface ClientOptions {
  baseUrl?: string;
  token?: string;
}

export class DexApiClient {
  private baseUrl: string;
  private token?: string;

  constructor(options?: ClientOptions) {
    this.baseUrl = options?.baseUrl ? sanitizeBaseUrl(options.baseUrl) : DEFAULT_API_BASE_URL;
    this.token = options?.token;
  }

  setBaseUrl(url: string) {
    this.baseUrl = sanitizeBaseUrl(url || DEFAULT_API_BASE_URL);
  }

  setToken(token?: string) {
    this.token = token?.trim() || undefined;
  }

  async fetchPrices(): Promise<ApiPriceResponse> {
    return this.request<ApiPriceResponse>("/orderbook/prices");
  }

  async createOrder(payload: ApiCreateOrderRequest): Promise<ApiCreateOrderResponse> {
    return this.request<ApiCreateOrderResponse>("/orderbook/orders", {
      method: "POST",
      body: JSON.stringify(payload)
    });
  }

  async fetchDepth(levels: number): Promise<ApiDepthSnapshot> {
    const params = new URLSearchParams({ levels: String(levels) });
    return this.request<ApiDepthSnapshot>(`/orderbook/depth?${params.toString()}`);
  }

  async fetchTradesForTrader(traderId: string): Promise<ApiGetTradesResponse> {
    return this.request<ApiGetTradesResponse>(`/orderbook/traders/${encodeURIComponent(traderId)}/trades`);
  }

  async fetchTradesForOrder(orderId: number): Promise<ApiGetTradesResponse> {
    return this.request<ApiGetTradesResponse>(`/orderbook/orders/${orderId}/trades`);
  }

  async fetchProposals(): Promise<ApiProposal[]> {
    return this.request<ApiProposal[]>("/dao/proposals");
  }

  async requestSharedToken(payload: ApiSharedTokenRequest): Promise<ApiTokenResponse> {
    return this.request<ApiTokenResponse>("/auth/token/shared", {
      method: "POST",
      body: JSON.stringify(payload)
    });
  }

  async requestWalletChallenge(address: string): Promise<ApiWalletChallengeResponse> {
    return this.request<ApiWalletChallengeResponse>("/auth/challenge", {
      method: "POST",
      body: JSON.stringify({ address })
    });
  }

  async requestWalletToken(payload: ApiWalletTokenRequest): Promise<ApiTokenResponse> {
    return this.request<ApiTokenResponse>("/auth/token/wallet", {
      method: "POST",
      body: JSON.stringify(payload)
    });
  }

  async createProposal(payload: ApiCreateProposalRequest): Promise<ApiCreateProposalResponse> {
    return this.request<ApiCreateProposalResponse>("/dao/proposals", {
      method: "POST",
      body: JSON.stringify(payload)
    });
  }

  async submitProposal(proposalId: string): Promise<void> {
    await this.request<void>(`/dao/proposals/${encodeURIComponent(proposalId)}/submit`, {
      method: "POST"
    });
  }

  async voteOnProposal(
    proposalId: string,
    payload: ApiVoteRequest
  ): Promise<ApiProposal> {
    return this.request<ApiProposal>(
      `/dao/proposals/${encodeURIComponent(proposalId)}/vote`,
      {
        method: "POST",
        body: JSON.stringify(payload)
      }
    );
  }

  async abstainOnProposal(
    proposalId: string,
    payload: ApiAbstainRequest
  ): Promise<ApiProposal> {
    return this.request<ApiProposal>(
      `/dao/proposals/${encodeURIComponent(proposalId)}/abstain`,
      {
        method: "POST",
        body: JSON.stringify(payload)
      }
    );
  }

  private async request<T>(path: string, init?: RequestInit): Promise<T> {
    const response = await fetch(this.buildUrl(path), {
      ...init,
      headers: this.mergeHeaders(init?.headers, Boolean(init?.body))
    });

    if (!response.ok) {
      throw new Error(await this.parseError(response));
    }

    if (response.status === 204) {
      return undefined as T;
    }

    return (await response.json()) as T;
  }

  private buildUrl(path: string) {
    return `${this.baseUrl}${path.startsWith("/") ? path : `/${path}`}`;
  }

  private mergeHeaders(extra: HeadersInit | undefined, hasBody: boolean) {
    const headers = new Headers(extra);
    headers.set("Accept", "application/json");
    if (hasBody && !headers.has("Content-Type")) {
      headers.set("Content-Type", "application/json");
    }
    if (this.token && !headers.has("Authorization")) {
      headers.set("Authorization", `Bearer ${this.token}`);
    }
    return headers;
  }

  private async parseError(response: Response) {
    try {
      const payload = (await response.json()) as { message?: string };
      if (payload?.message) {
        return payload.message;
      }
    } catch {
      // ignore json parse errors
    }
    return `Request failed with status ${response.status}`;
  }
}

function sanitizeBaseUrl(url: string) {
  return url.trim().replace(/\/+$/, "") || DEFAULT_API_BASE_URL;
}
