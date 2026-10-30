import { Icon } from "../components/Icon";
import type { Session } from "../features/auth/types";
import { navigation, type Page } from "./navigation";

interface Props {
  session: Session;
  page: Page;
  presentCount: number;
  busy: boolean;
  loading: boolean;
  onNavigate: (page: Page) => void;
  onAccounts: () => void;
  onLogout: () => void;
}

export function Sidebar({ session, page, presentCount, busy, loading, onNavigate, onAccounts, onLogout }: Props) {
  return (
    <aside className="sidebar">
      <div className="brand">
        <span className="brand-symbol">
          <Icon name="herd" size={27} />
        </span>
        <span>
          Ferma<span className="brand-dot">.</span>
        </span>
      </div>
      <div className="farm-switch">
        <span className="farm-avatar">
          <Icon name="leaf" />
        </span>
        <div>
          <strong>{session.farm_name}</strong>
          <small>Exploatația mea</small>
        </div>
      </div>
      <span className="nav-label">ADMINISTRARE</span>
      <nav>
        {navigation.map((item) => (
          <button
            key={item.page}
            className={`nav-item ${page === item.page ? "active" : ""}`}
            onClick={() => onNavigate(item.page)}
          >
            <Icon name={item.icon} />
            <span>{item.label}</span>
            {item.page === "herd" && (
              <span className="nav-count">{presentCount}</span>
            )}
          </button>
        ))}
      </nav>
      <div className="sidebar-bottom">
        <div className="local-card">
          <span className="status-dot" />
          <strong>Spațiul tău de lucru</strong>
          <p>Date salvate local, pe calculator.</p>
        </div>
        <button className="secondary full" disabled={busy || loading} onClick={onAccounts}>
          Schimbă contul
        </button>
        <div className="profile">
          <span className="avatar">
            {session.user.username.slice(0, 2).toUpperCase()}
          </span>
          <div>
            <strong>{session.user.username}</strong>
            <small>
              {
                {
                  Admin: "Administrator",
                  Editor: "Editor",
                  Viewer: "Vizualizare",
                }[session.user.role]
              }
            </small>
          </div>
          <button
            className="icon-button"
            aria-label="Deconectare"
            title="Deconectare"
            disabled={busy}
            onClick={onLogout}
          >
            <Icon name="logout" size={18} />
          </button>
        </div>
      </div>
    </aside>
  );
}
