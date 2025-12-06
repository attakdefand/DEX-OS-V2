import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useNotifications } from "./useNotifications";

describe("useNotifications", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("auto dismisses a notification after the configured timeout", () => {
    const { result } = renderHook(() => useNotifications({ defaultAutoHideMs: 1_000 }));

    act(() => {
      result.current.notify({ type: "info", message: "hello", title: "Test" });
    });

    expect(result.current.notifications).toHaveLength(1);

    act(() => {
      vi.advanceTimersByTime(1_000);
    });

    expect(result.current.notifications).toHaveLength(0);
  });

  it("trims the oldest notifications when maxVisible is exceeded", () => {
    const { result } = renderHook(() => useNotifications({ maxVisible: 2, defaultAutoHideMs: 0 }));

    act(() => {
      result.current.notify({ type: "info", message: "first", title: "First" });
      result.current.notify({ type: "info", message: "second", title: "Second" });
      result.current.notify({ type: "info", message: "third", title: "Third" });
    });

    expect(result.current.notifications).toHaveLength(2);
    expect(result.current.notifications.map((item) => item.message)).toEqual(["second", "third"]);
  });

  it("deduplicates identical notifications by type, message, and source", () => {
    const { result } = renderHook(() => useNotifications({ defaultAutoHideMs: 0 }));

    act(() => {
      result.current.notify({
        type: "error",
        message: "Problem detected",
        title: "Alert",
        source: "api"
      });
      result.current.notify({
        type: "error",
        message: "Problem detected",
        title: "Alert",
        source: "api"
      });
    });

    expect(result.current.notifications).toHaveLength(1);
    expect(result.current.notifications[0].source).toBe("api");
  });

  it("clears all notifications and timers", () => {
    const { result } = renderHook(() => useNotifications({ defaultAutoHideMs: 0 }));

    act(() => {
      result.current.notify({ type: "success", message: "ok", title: "OK" });
      result.current.notify({ type: "info", message: "info", title: "Info" });
    });

    expect(result.current.notifications).toHaveLength(2);

    act(() => {
      result.current.clearNotifications();
    });

    expect(result.current.notifications).toHaveLength(0);
  });
});
