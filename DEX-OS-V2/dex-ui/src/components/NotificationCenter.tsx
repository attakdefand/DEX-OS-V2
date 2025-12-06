import { Notification as UiNotification } from "../types/notifications";

interface NotificationCenterProps {
  notifications: UiNotification[];
  onDismiss: (id: string) => void;
  onClearAll?: () => void;
}

const roleFor = (type: UiNotification["type"]) => (type === "error" ? "alert" : "status");
const liveRegionFor = (type: UiNotification["type"]) => (type === "error" ? "assertive" : "polite");

export const NotificationCenter = ({ notifications, onDismiss, onClearAll }: NotificationCenterProps) => {
  if (notifications.length === 0) {
    return null;
  }

  const ordered = [...notifications].sort((a, b) => b.createdAt - a.createdAt);

  return (
    <aside className="notification-tray" aria-label="User notifications">
      <div className="notification-tray__header">
        <p className="eyebrow">Notifications</p>
        <div className="notification-tray__actions">
          <span className="pill">{ordered.length} active</span>
          {onClearAll && (
            <button type="button" className="ghost" onClick={onClearAll}>
              Clear all
            </button>
          )}
        </div>
      </div>

      <ul className="notification-list">
        {ordered.map((item) => (
          <li key={item.id} className={`notice notice--${item.type}`}>
            <div className="notice__body" role={roleFor(item.type)} aria-live={liveRegionFor(item.type)}>
              <div className="notice__head">
                <p className="notice__title">{item.title}</p>
                {item.source && <span className="notice__source">{item.source}</span>}
              </div>
              <p className="notice__message">{item.message}</p>
              <p className="notice__meta">{new Date(item.createdAt).toLocaleTimeString()}</p>
            </div>
            <button
              type="button"
              className="ghost"
              onClick={() => onDismiss(item.id)}
              aria-label={`Dismiss ${item.title} notification`}
            >
              Dismiss
            </button>
          </li>
        ))}
      </ul>
    </aside>
  );
};
