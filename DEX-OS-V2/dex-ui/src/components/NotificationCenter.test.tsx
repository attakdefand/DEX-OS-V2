import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { NotificationCenter } from "./NotificationCenter";
import { Notification as UiNotification } from "../types/notifications";

const sampleNotifications: UiNotification[] = [
  {
    id: "1",
    type: "info",
    title: "API status",
    message: "Gateway connected",
    source: "api",
    createdAt: Date.now()
  },
  {
    id: "2",
    type: "error",
    title: "Order",
    message: "Signature rejected",
    source: "orders",
    createdAt: Date.now() + 5
  }
];

describe("NotificationCenter", () => {
  it("renders notifications with metadata and supports clearing all", async () => {
    const onDismiss = vi.fn();
    const onClearAll = vi.fn();
    const user = userEvent.setup();

    render(
      <NotificationCenter
        notifications={sampleNotifications}
        onDismiss={onDismiss}
        onClearAll={onClearAll}
      />
    );

    expect(screen.getByText("API status")).toBeInTheDocument();
    expect(screen.getByText("Gateway connected")).toBeInTheDocument();
    expect(screen.getByText("orders")).toBeInTheDocument();
    expect(screen.getAllByRole("status").length).toBeGreaterThan(0);
    expect(screen.getByRole("alert")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: /clear all/i }));
    expect(onClearAll).toHaveBeenCalledTimes(1);
  });

  it("invokes dismiss handler for the targeted notification", async () => {
    const onDismiss = vi.fn();
    const user = userEvent.setup();

    render(<NotificationCenter notifications={[sampleNotifications[0]]} onDismiss={onDismiss} />);

    await user.click(screen.getByRole("button", { name: /dismiss api status notification/i }));

    expect(onDismiss).toHaveBeenCalledTimes(1);
    expect(onDismiss).toHaveBeenCalledWith("1");
  });
});
