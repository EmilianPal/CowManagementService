import { Icon } from "../components/Icon";
import { formatDate, today, type Cow, type Birth } from "../features/herd/types";
import { navigation, type Page } from "./navigation";

export function Topbar({ page }: { page: Page }) {
  return (
    <header className="topbar">
      <div>
        Ferma mea <Icon name="chevron" size={14} />
        <strong>{navigation.find((n) => n.page === page)?.label}</strong>
      </div>
      <span>
        <Icon name="calendar" size={17} />
        {formatDate(today())}
      </span>
    </header>
  );
}

interface PageHeadingProps {
  page: Page;
  writable: boolean;
  busy: boolean;
  onExport: () => void;
  onAdd: () => void;
}

export function PageHeading({ page, writable, busy, onExport, onAdd }: PageHeadingProps) {
  return (
    <div className="page-heading">
      <div>
        <span className="eyebrow">
          {page === "herd"
            ? "FIECARE BOVINĂ CONTEAZĂ"
            : "TOTUL ÎNTR-UN SINGUR LOC"}
        </span>
        <h1>{navigation.find((n) => n.page === page)?.label}</h1>
        <p>
          {
            {
              exits: "Selectează bovinele și înregistrează ieșirea lor împreună.",
              herd: "O evidență clară a efectivului și a istoricului fermei tale.",
              extract:
                "Numărul de bovine pe vârste și sexe, pentru completarea actelor.",
              overview: "Starea fermei tale, dintr-o singură privire.",
              births: "Noile generații și istoricul fătărilor din fermă.",
              inseminations:
                "Urmărește montele și însămânțările bovinelor.",
              reports:
                "Pregătește situația efectivului la data de care ai nevoie.",
            }[page]
          }
        </p>
      </div>
      <div className="heading-actions">
        {page === "herd" && (
          <button
            className="secondary"
            onClick={onExport}
          >
            <Icon name="download" size={18} />
            Exportă
          </button>
        )}
        {writable && page !== "reports" && page !== "extract" && page !== "exits" && (
          <button
            className="primary"
            disabled={busy}
            onClick={onAdd}
          >
            <Icon name="plus" size={18} />
            {page === "births"
              ? "Adaugă o fătare"
              : page === "inseminations"
                ? "Adaugă o montă"
                : "Adaugă o bovină"}
          </button>
        )}
      </div>
    </div>
  );
}

export function FarmStats({ present, births, loading }: { present: Cow[]; births: Birth[]; loading: boolean }) {
  return (
    <section className="stats" aria-label="Statistici ale fermei">
      {[
        {
          label: "Bovine în fermă",
          value: present.length,
          icon: "herd",
          note: "Efectiv prezent astăzi",
          tone: "green",
        },
        {
          label: "Femele",
          value: present.filter((c) => c.sex === "Female").length,
          icon: "heart",
          note: "Din efectivul actual",
          tone: "rose",
        },
        {
          label: "Masculi",
          value: present.filter((c) => c.sex === "Male").length,
          icon: "herd",
          note: "Din efectivul actual",
          tone: "blue",
        },
        {
          label: "Fătări anul acesta",
          value: births.filter((b) =>
            b.date.startsWith(today().slice(0, 4)),
          ).length,
          icon: "leaf",
          note: `Înregistrate în ${today().slice(0, 4)}`,
          tone: "amber",
        },
      ].map((s) => (
        <article className="stat" key={s.label}>
          <div>
            <span>{s.label}</span>
            <strong>{loading ? "—" : s.value}</strong>
            <small>{s.note}</small>
          </div>
          <span className={`stat-icon ${s.tone}`}>
            <Icon name={s.icon} size={23} />
          </span>
        </article>
      ))}
    </section>
  );
}

export function WorkspaceFooter({ loading }: { loading: boolean }) {
  return (
    <footer className="workspace-footer">
      <span>
        <span className="status-dot" />
        {loading
          ? "Se actualizează datele…"
          : "Evidența fermei · Stocare locală"}
      </span>
      <span>Grijă pentru animale. Claritate pentru tine.</span>
    </footer>
  );
}
