import { useCallback, useEffect, useRef, useState } from "react";
import { Notification as UiNotification, NotificationKind, NotificationSource } from "../types/notifications";

interface NotifyInput {
  type: NotificationKind;
  message: string;
  title?: string;
  source?: NotificationSource;
  autoHideMs?: number;
}

interface UseNotificationsOptions {
  maxVisible?: number;
  defaultAutoHideMs?: number;
}

export interface UseNotificationsResult {
  notifications: UiNotification[];
  notify: (input: NotifyInput) => string;
  dismissNotification: (id: string) => void;
  clearNotifications: () => void;
}

const DEFAULT_AUTO_HIDE_MS = 6_000;
const DEFAULT_MAX_VISIBLE = 4;

export function useNotifications(options?: UseNotificationsOptions): UseNotificationsResult {
  const [notifications, setNotifications] = useState<UiNotification[]>([]);
  const timers = useRef<Map<string, number>>(new Map());

  const defaultAutoHideMs = options?.defaultAutoHideMs ?? DEFAULT_AUTO_HIDE_MS;
  const maxVisible = options?.maxVisible ?? DEFAULT_MAX_VISIBLE;

  const clearTimer = useCallback((id: string) => {
    const timerId = timers.current.get(id);
    if (timerId && typeof window !== "undefined") {
      window.clearTimeout(timerId);
    }
    timers.current.delete(id);
  }, []);

  const dismissNotification = useCallback(
    (id: string) => {
      clearTimer(id);
      setNotifications((prev) => prev.filter((item) => item.id !== id));
    },
    [clearTimer]
  );

  const clearNotifications = useCallback(() => {
    if (typeof window !== "undefined") {
      timers.current.forEach((timerId) => window.clearTimeout(timerId));
    }
    timers.current.clear();
    setNotifications([]);
  }, []);

  const notify = useCallback(
    (input: NotifyInput) => {
      const id = createNotificationId();
      const autoHideMs = input.autoHideMs ?? defaultAutoHideMs;

      const next: UiNotification = {
        id,
        type: input.type,
        title: input.title ?? getDefaultTitle(input.type),
        message: input.message.trim(),
        source: input.source,
        createdAt: Date.now(),
        autoHideMs
      };

      setNotifications((prev) => {
        const deduped = prev.filter(
          (item) => item.message !== next.message || item.type !== next.type || item.source !== next.source
        );
        const combined = [...deduped, next];
        const overflow = combined.length - maxVisible;
        if (overflow <= 0) {
          return combined;
        }
        const trimmed = combined.slice(overflow);
        const removedIds = combined.slice(0, overflow).map((item) => item.id);
        removedIds.forEach(clearTimer);
        return trimmed;
      });

      if (autoHideMs && autoHideMs > 0 && typeof window !== "undefined") {
        const timerId = window.setTimeout(() => dismissNotification(id), autoHideMs);
        timers.current.set(id, timerId);
      }

      return id;
    },
    [clearTimer, defaultAutoHideMs, dismissNotification, maxVisible]
  );

  useEffect(() => {
    return () => {
      clearNotifications();
    };
  }, [clearNotifications]);

  return { notifications, notify, dismissNotification, clearNotifications };
}

function createNotificationId() {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return crypto.randomUUID();
  }
  return `${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

function getDefaultTitle(type: NotificationKind) {
  switch (type) {
    case "success":
      return "Success";
    case "warning":
      return "Warning";
    case "error":
      return "Attention needed";
    case "info":
    default:
      return "Heads up";
  }
}
