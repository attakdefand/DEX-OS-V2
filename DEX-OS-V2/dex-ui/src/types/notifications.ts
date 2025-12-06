export type NotificationKind = "success" | "error" | "info" | "warning";

export type NotificationSource =
  | "api"
  | "wallet"
  | "orders"
  | "trades"
  | "depth"
  | "auth"
  | "security"
  | "system"
  | "network";

export interface Notification {
  id: string;
  type: NotificationKind;
  title: string;
  message: string;
  source?: NotificationSource;
  createdAt: number;
  autoHideMs?: number;
}
