import { HerdExits } from "./features/herd/HerdExits";
import { DateInput } from "./components/DateInput";
import {
  useCallback,
  useEffect,
  useMemo,
  useState,
  type FormEvent,
} from "react";
import { isTauri } from "@tauri-apps/api/core";
import { command, romanianError } from "./lib/api";
import { auth } from "./features/auth/services";
import type { Session } from "./features/auth/types";
import {
  breeds,
  sexes,
  categories,
  today,
  formatDate,
  isPresent,
  type Cow,
  type Birth,
  type Insemination,
} from "./features/herd/types";
import { Icon } from "./components/Icon";
import { Modal } from "./components/Modal";
import { FilterFields } from "./features/herd/FilterFields";
import { HerdExtract } from "./features/herd/HerdExtract";
import {
  emptyFilter,
  validateFilters,
  toCowFilter,
} from "./features/herd/filters";
import "./App.css";

type Page =
  "herd" | "overview" | "births" | "inseminations" | "reports" | "extract" | "exits";
type Editor =
  | { kind: "cow"; value?: Cow }
  | { kind: "birth"; value?: Birth }
  | { kind: "insemination"; value?: Insemination }
  | {
      kind: "delete";
      command: string;
      args: Record<string, unknown>;
      label: string;
    };
const navigation: { page: Page; label: string; icon: string }[] = [
  { page: "overview", label: "Privire de ansamblu", icon: "grid" },
  { page: "herd", label: "Registrul bovinelor", icon: "herd" },
  { page: "exits", label: "Ieșiri din fermă", icon: "logout" },
  { page: "extract", label: "Extras efectiv", icon: "chart" },
  { page: "inseminations", label: "Montări", icon: "heart" },
  { page: "births", label: "Fătări", icon: "calendar" },
  { page: "reports", label: "Rapoarte și export", icon: "chart" },
];
function Options({ values }: { values: Record<string, string> }) {
  return (
    <>
      {Object.entries(values).map(([value, label]) => (
        <option key={value} value={value}>
          {label}
        </option>
      ))}
    </>
  );
}

function App() {
  const [session, setSession] = useState<Session | null>(null);
  const [starting, setStarting] = useState(true);
  const [authMode, setAuthMode] = useState<"login" | "register">("login");
  const [page, setPage] = useState<Page>("herd");
  const [cows, setCows] = useState<Cow[]>([]);
  const [births, setBirths] = useState<Birth[]>([]);
  const [inseminations, setInseminations] = useState<Insemination[]>([]);
  const [history, setHistory] = useState({ undo: false, redo: false });
  const [busy, setBusy] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [search, setSearch] = useState("");
  const [filter, setFilter] = useState(emptyFilter);
  const [showFilters, setShowFilters] = useState(false);
  const [status, setStatus] = useState<"all" | "present" | "exited">("present");
  const [selected, setSelected] = useState<number | null>(null);
  const [editor, setEditor] = useState<Editor | null>(null);
  const [modalError, setModalError] = useState("");
  const [pageIndex, setPageIndex] = useState(0);
  const [sort, setSort] = useState<"tag" | "newest" | "oldest">("tag");
  const [reportDate, setReportDate] = useState(today());
  const [filteredCows, setFilteredCows] = useState<Cow[]>([]);
  const [filtering, setFiltering] = useState(false);
  const [filterLoadError, setFilterLoadError] = useState("");
  const filterError =
    validateFilters(filter) ||
    (!reportDate ? "Selectează data de referință." : "");
  const queryFilter = useMemo(
    () => toCowFilter(filter, search, reportDate, status),
    [filter, search, reportDate, status],
  );
  function resetFilters() {
    setFilter(emptyFilter);
    setSearch("");
    setStatus("present");
    setReportDate(today());
  }
  const writable = session?.user.role !== "Viewer";
  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const [c, b, i, h] = await Promise.all([
        command<Cow[]>("get_cows"),
        command<Birth[]>("get_births"),
        command<Insemination[]>("get_inseminations"),
        command<{ undo: boolean; redo: boolean }>("get_history_state"),
      ]);
      setCows(c);
      setBirths(b);
      setInseminations(i);
      setHistory(h);
    } finally {
      setLoading(false);
    }
  }, []);
  useEffect(() => {
    let active = true;
    auth
      .session()
      .then((value) => {
        if (active) setSession(value);
      })
      .catch((e) => {
        if (active) setError(romanianError(e));
      })
      .finally(() => {
        if (active) setStarting(false);
      });
    return () => {
      active = false;
    };
  }, []);
  useEffect(() => {
    if (session) refresh().catch((e) => setError(romanianError(e)));
  }, [session, refresh]);
  useEffect(() => {
    setPageIndex(0);
  }, [search, filter, status, sort, reportDate]);
  useEffect(() => {
    if (!notice) return;
    const timer = setTimeout(() => setNotice(""), 6000);
    return () => clearTimeout(timer);
  }, [notice]);
  useEffect(() => {
    setFilteredCows([]);
    setFilterLoadError("");
    if (!session || filterError) {
      setFiltering(false);
      return;
    }
    let active = true;
    setFiltering(true);
    const timer = setTimeout(() => {
      command<Cow[]>("get_cows_filtered", { filter: queryFilter })
        .then((rows) => {
          if (active) setFilteredCows(rows);
        })
        .catch((error) => {
          if (active) setFilterLoadError(romanianError(error));
        })
        .finally(() => {
          if (active) setFiltering(false);
        });
    }, 150);
    return () => {
      active = false;
      clearTimeout(timer);
    };
  }, [session, queryFilter, cows, filterError]);
  const visible = useMemo(
    () =>
      [...filteredCows].sort((a, b) =>
        sort === "tag"
          ? a.ear_tag.localeCompare(b.ear_tag)
          : sort === "newest"
            ? b.birth_date.localeCompare(a.birth_date)
            : a.birth_date.localeCompare(b.birth_date),
      ),
    [filteredCows, sort],
  );
  const currentPage = Math.min(
    pageIndex,
    Math.max(0, Math.ceil(visible.length / 12) - 1),
  );
  const displayed = visible.slice(currentPage * 12, currentPage * 12 + 12);
  const present = cows.filter((c) => isPresent(c));
  const selectedCow = cows.find((c) => c.id === selected);
  const tag = (id: number | null) =>
    cows.find((c) => c.id === id)?.ear_tag || "Nespecificat";
  const openEditor = (value: Editor) => {
    setModalError("");
    setEditor(value);
  };
  async function mutate(
    action: () => Promise<unknown>,
    success: string,
    inModal = false,
  ) {
    setBusy(true);
    setError("");
    setModalError("");
    try {
      const result = await action();
      if (result === false)
        throw new Error(
          "Înregistrarea nu a fost modificată. Reîncarcă datele și încearcă din nou.",
        );
      setEditor(null);
      setNotice(success);
      try {
        await refresh();
      } catch {
        setError(
          "Modificarea a fost salvată, dar lista nu a putut fi reîncărcată. Apasă Reîncarcă.",
        );
      }
    } catch (e) {
      (inModal ? setModalError : setError)(romanianError(e));
    } finally {
      setBusy(false);
    }
  }
  async function authenticate(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    const data = new FormData(e.currentTarget);
    setBusy(true);
    setError("");
    try {
      if (!String(data.get("username") || "").trim() || !data.get("password"))
        throw new Error("Completează numele de utilizator și parola.");
      if (authMode === "register") {
        if (
          !String(data.get("farm") || "").trim() ||
          !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(String(data.get("email")))
        )
          throw new Error(
            "Completează numele fermei și o adresă de e-mail validă.",
          );
        if (String(data.get("password")).length < 8)
          throw new Error("Parola trebuie să aibă cel puțin 8 caractere.");
        await auth.register(
          String(data.get("farm")).trim(),
          String(data.get("username")).trim(),
          String(data.get("email")).trim(),
          String(data.get("password")),
        );
      } else
        await auth.login(
          String(data.get("username")).trim(),
          String(data.get("password")),
        );
      setSession(await auth.session());
    } catch (e) {
      setError(romanianError(e));
    } finally {
      setBusy(false);
    }
  }
  async function saveEditor(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    if (!editor || !session) return;
    const form = e.currentTarget;
    if (!form.checkValidity()) {
      setModalError(
        "Completează toate câmpurile obligatorii cu valori valide.",
      );
      return;
    }
    const data = new FormData(form);
    const str = (key: string) => String(data.get(key) || "").trim();
    const num = (key: string) => (str(key) ? Number(str(key)) : null);
    if (editor.kind === "cow") {
      const birthDate = str("birth_date"),
        entryDate = str("entry_date"),
        exitDate = str("exit_date");
      if (!str("ear_tag") || !birthDate || !entryDate) {
        setModalError("Completează crotalia și datele obligatorii.");
        return;
      }
      if (
        birthDate > today() ||
        entryDate < birthDate ||
        (exitDate && exitDate < entryDate)
      ) {
        setModalError(
          "Data nașterii nu poate fi în viitor, intrarea trebuie să fie după naștere, iar ieșirea după intrare.",
        );
        return;
      }
      const birth = births.find((b) => b.id === editor.value?.birth_id);
      if (
        editor.value &&
        str("sex") !== editor.value.sex &&
        (births.some((b) => b.mother_id === editor.value?.id) ||
          inseminations.some(
            (i) =>
              i.dam_id === editor.value?.id || i.sire_id === editor.value?.id,
          ))
      ) {
        setModalError(
          "Nu poți schimba sexul unei bovine cu evenimente de reproducție. Corectează mai întâi evenimentele asociate.",
        );
        return;
      }
      if (
        birth &&
        (birth.date !== birthDate || birth.mother_id === editor.value?.id)
      ) {
        setModalError(
          "Data nașterii trebuie să coincidă cu fătarea asociată, iar bovina nu poate fi propria mamă.",
        );
        return;
      }
      const cow: Cow = {
        id: editor.value?.id ?? null,
        farm_id: session.user.farm_id,
        ear_tag: str("ear_tag").toUpperCase(),
        sex: str("sex") as Cow["sex"],
        breed: str("breed") as Cow["breed"],
        category: str("category") as Cow["category"],
        birth_date: birthDate,
        entry_date: entryDate,
        exit_date: exitDate || null,
        birth_id: editor.value?.birth_id ?? null,
        birth_count: editor.value?.birth_count ?? 0,
        insemination_count: editor.value?.insemination_count ?? 0,
      };
      await mutate(
        () => command(editor.value ? "update_cow" : "add_cow", { cow }),
        "Datele bovinei au fost salvate.",
        true,
      );
    } else if (editor.kind === "birth" || editor.kind === "insemination") {
      const mother = cows.find((c) => c.id === num("mother"));
      const sire = cows.find((c) => c.id === num("sire"));
      if (
        !mother ||
        !str("date") ||
        str("date") < mother.birth_date ||
        str("date") > today() ||
        (sire && str("date") < sire.birth_date)
      ) {
        setModalError(
          "Data evenimentului trebuie să fie după nașterea bovinelor și nu poate fi în viitor.",
        );
        return;
      }
      const common = {
        id: editor.value?.id ?? null,
        farm_id: session.user.farm_id,
        date: str("date"),
      };
      if (
        editor.kind === "birth" &&
        editor.value &&
        cows.some(
          (c) =>
            c.birth_id === editor.value?.id &&
            (c.birth_date !== str("date") || c.id === mother.id),
        )
      ) {
        setModalError(
          "Data fătării trebuie să coincidă cu nașterea vițeilor asociați, iar mama nu poate fi unul dintre viței.",
        );
        return;
      }
      const kind = editor.kind;
      await mutate(
        () =>
          command(
            `${editor.value ? "update" : "add"}_${kind}`,
            kind === "birth"
              ? { birth: { ...common, mother_id: mother.id } }
              : {
                  insemination: {
                    ...common,
                    dam_id: mother.id,
                    sire_id: num("sire"),
                  },
                },
          ),
        "Evenimentul a fost salvat.",
        true,
      );
    }
  }
  async function exportReport() {
    if (busy || filterError) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const path = await command<string | null>("export_report", {
        filter: queryFilter,
      });
      if (path) setNotice(`Raportul a fost salvat în ${path}.`);
    } catch (e) {
      setError(romanianError(e));
    } finally {
      setBusy(false);
    }
  }
  if (starting)
    return (
      <div className="startup">
        <span className="brand-symbol">
          <Icon name="herd" size={34} />
        </span>
        <h2>Se deschide ferma…</h2>
        <p>Se verifică sesiunea locală.</p>
      </div>
    );
  if (!session)
    return (
      <main className="auth-layout">
        <section className="auth-panel" aria-labelledby="auth-title">
          <div className="auth-card">
            <h1 id="auth-title">Manager de bovine</h1>
            <p>{authMode === "login" ? "Autentificare" : "Creează o fermă"}</p>
            {error && (
              <div className="alert error" role="alert">
                {error}
              </div>
            )}
            <form onSubmit={authenticate} noValidate>
              <fieldset disabled={busy}>
                {authMode === "register" && (
                  <>
                    <label>
                      Numele fermei
                      <input
                        name="farm"
                        required
                        placeholder="De exemplu, Ferma de la Deal"
                      />
                    </label>
                    <label>
                      Adresă de e-mail
                      <input
                        name="email"
                        type="email"
                        required
                        placeholder="nume@exemplu.ro"
                      />
                    </label>
                  </>
                )}
                <label>
                  Nume de utilizator
                  <input
                    name="username"
                    autoComplete="username"
                    required
                    placeholder="Numele tău de utilizator"
                  />
                </label>
                <label>
                  Parolă
                  <input
                    name="password"
                    type="password"
                    autoComplete={
                      authMode === "login" ? "current-password" : "new-password"
                    }
                    required
                    placeholder={
                      authMode === "register"
                        ? "Cel puțin 8 caractere"
                        : "Introdu parola"
                    }
                  />
                </label>
                <button className="primary full" disabled={busy || !isTauri()}>
                  {busy
                    ? "Se verifică…"
                    : authMode === "login"
                      ? "Autentifică-te"
                      : "Creează ferma"}
                </button>
              </fieldset>
            </form>
            <button
              className="auth-switch"
              disabled={busy}
              onClick={() => {
                setAuthMode(authMode === "login" ? "register" : "login");
                setError("");
              }}
            >
              {authMode === "login"
                ? "Prima utilizare? Creează o fermă"
                : "Înapoi la autentificare"}
            </button>
          </div>
        </section>
      </main>
    );

  return (
    <div className="app-shell">
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
              onClick={() => {
                setPage(item.page);
                setSelected(null);
              }}
            >
              <Icon name={item.icon} />
              <span>{item.label}</span>
              {item.page === "herd" && (
                <span className="nav-count">{cows.length}</span>
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
              onClick={async () => {
                setBusy(true);
                try {
                  await auth.logout();
                  setSession(null);
                  setCows([]);
                  setBirths([]);
                  setInseminations([]);
                  setSelected(null);
                  setEditor(null);
                  setError("");
                  setNotice("");
                } catch (e) {
                  setError(romanianError(e));
                } finally {
                  setBusy(false);
                }
              }}
            >
              <Icon name="logout" size={18} />
            </button>
          </div>
        </div>
      </aside>
      <div className="workspace">
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
        <main
          className={`main-content ${page === "extract" ? "extract-page" : ""}`}
        >
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
                      "Urmărește montările și însămânțările bovinelor.",
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
                  onClick={() => setPage("reports")}
                >
                  <Icon name="download" size={18} />
                  Exportă
                </button>
              )}
              {writable && page !== "reports" && page !== "extract" && page !== "exits" && (
                <button
                  className="primary"
                  disabled={busy}
                  onClick={() =>
                    openEditor({
                      kind:
                        page === "births"
                          ? "birth"
                          : page === "inseminations"
                            ? "insemination"
                            : "cow",
                    })
                  }
                >
                  <Icon name="plus" size={18} />
                  {page === "births"
                    ? "Adaugă o fătare"
                    : page === "inseminations"
                      ? "Adaugă o montare"
                      : "Adaugă o bovină"}
                </button>
              )}
            </div>
          </div>
          {error && (
            <div className="alert error" role="alert">
              {error}
              <button onClick={() => setError("")} aria-label="Închide mesajul">
                <Icon name="close" size={16} />
              </button>
            </div>
          )}
          {notice && (
            <div className="alert success" role="status">
              <Icon name="check" size={18} />
              {notice}
              <button
                onClick={() => setNotice("")}
                aria-label="Închide mesajul"
              >
                <Icon name="close" size={16} />
              </button>
            </div>
          )}
          {page !== "extract" && (
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
          )}

          {page === "exits" && <HerdExits writable={writable} onSaved={refresh} />}
          {page === "extract" && <HerdExtract />}
          {page === "herd" && (
            <section className="panel registry">
              <div className="panel-top">
                <div className="tabs">
                  {(
                    [
                      {
                        key: "all",
                        label: "Toate bovinele",
                        count: cows.length,
                      },
                      {
                        key: "present",
                        label: "În fermă",
                        count: cows.filter((c) => isPresent(c, reportDate))
                          .length,
                      },
                      {
                        key: "exited",
                        label: "Ieșite",
                        count: cows.filter(
                          (c) => c.exit_date && c.exit_date <= reportDate,
                        ).length,
                      },
                    ] as const
                  ).map((t) => (
                    <button
                      key={t.key}
                      className={status === t.key ? "active" : ""}
                      onClick={() => setStatus(t.key)}
                    >
                      {t.label}
                      <span>{t.count}</span>
                    </button>
                  ))}
                </div>
                <div className="history-controls">
                  <button
                    title="Anulează ultima modificare"
                    aria-label="Anulează ultima modificare"
                    disabled={busy || !history.undo || !writable}
                    onClick={() =>
                      mutate(
                        () => command("undo"),
                        "Ultima modificare a fost anulată.",
                      )
                    }
                  >
                    <Icon name="undo" size={17} />
                  </button>
                  <button
                    title="Refă modificarea"
                    aria-label="Refă modificarea"
                    disabled={busy || !history.redo || !writable}
                    onClick={() =>
                      mutate(
                        () => command("redo"),
                        "Modificarea a fost refăcută.",
                      )
                    }
                  >
                    <Icon name="redo" size={17} />
                  </button>
                </div>
              </div>
              <div className="toolbar">
                <div className="search">
                  <Icon name="search" size={18} />
                  <input
                    aria-label="Caută după crotalie"
                    placeholder="Caută după crotalie…"
                    value={search}
                    onChange={(e) => setSearch(e.target.value)}
                  />
                  {search && (
                    <button
                      aria-label="Șterge căutarea"
                      onClick={() => setSearch("")}
                    >
                      <Icon name="close" size={15} />
                    </button>
                  )}
                </div>
                <button
                  className={`secondary ${showFilters ? "selected" : ""}`}
                  aria-expanded={showFilters}
                  onClick={() => setShowFilters(!showFilters)}
                >
                  <Icon name="filter" size={18} />
                  Filtre
                  {Object.values(filter).filter(Boolean).length > 0 && (
                    <span className="filter-count">
                      {Object.values(filter).filter(Boolean).length}
                    </span>
                  )}
                </button>
                <select
                  aria-label="Sortează bovinele"
                  value={sort}
                  onChange={(e) => setSort(e.target.value as typeof sort)}
                >
                  <option value="tag">Crotalie: crescător</option>
                  <option value="newest">Cele mai tinere</option>
                  <option value="oldest">Cele mai în vârstă</option>
                </select>
                <button
                  className="icon-button"
                  title="Reîncarcă"
                  aria-label="Reîncarcă"
                  disabled={loading || busy}
                  onClick={() =>
                    refresh()
                      .then(() => setError(""))
                      .catch((e) => setError(romanianError(e)))
                  }
                >
                  <Icon name="refresh" size={18} />
                </button>
              </div>
              {showFilters && (
                <FilterFields
                  value={filter}
                  onChange={setFilter}
                  date={reportDate}
                  onDateChange={setReportDate}
                  status={status}
                  onStatusChange={setStatus}
                  onReset={resetFilters}
                />
              )}
              {(filterError || filterLoadError) && (
                <div className="alert error" role="alert">
                  {filterError || filterLoadError}
                </div>
              )}
              <p className="filter-context">
                Data de referință: {formatDate(reportDate || null)} ·{" "}
                {filtering ? "Se filtrează…" : `${visible.length} bovine`}
              </p>
              <div className="table-scroll">
                <table>
                  <thead>
                    <tr>
                      <th className="row-number">Nr.</th>
                      <th>Crotalie</th>
                      <th>Rasă</th>
                      <th>Sex</th>
                      <th>Data nașterii</th>
                      <th>Data intrării</th>
                      <th>Data ieșirii</th>
                      <th>Categorie</th>
                      <th>Montări</th>
                      <th>Fătări</th>
                      <th>Stare</th>
                      <th>
                        <span className="sr-only">Detalii</span>
                      </th>
                    </tr>
                  </thead>
                  <tbody>
                    {displayed.map((c, index) => (
                      <tr
                        key={c.id}
                        className={selected === c.id ? "selected-row" : ""}
                      >
                        <td className="row-number">
                          {currentPage * 12 + index + 1}
                        </td>
                        <td>
                          <button
                            className="tag-button"
                            onClick={() => setSelected(c.id)}
                          >
                            <span className="mini-cow">
                              <Icon name="herd" size={17} />
                            </span>
                            {c.ear_tag}
                          </button>
                        </td>
                        <td>{breeds[c.breed]}</td>
                        <td>
                          <span
                            className={`sex-badge ${c.sex === "Female" ? "female" : "male"}`}
                          >
                            {c.sex === "Female" ? "♀" : "♂"}
                          </span>
                          {sexes[c.sex]}
                        </td>
                        <td>{formatDate(c.birth_date)}</td>
                        <td>{formatDate(c.entry_date)}</td>
                        <td className={!c.exit_date ? "muted" : ""}>
                          {formatDate(c.exit_date)}
                        </td>
                        <td>
                          <span className="category-badge">{c.category}</span>
                        </td>
                        <td className="numeric">{c.insemination_count}</td>
                        <td className="numeric">{c.birth_count}</td>
                        <td>
                          <span
                            className={`state-badge ${isPresent(c, reportDate) ? "present" : "exited"}`}
                          >
                            <i />
                            {isPresent(c, reportDate)
                              ? "În fermă"
                              : c.entry_date > reportDate
                                ? "Intrare viitoare"
                                : "Ieșită"}
                          </span>
                        </td>
                        <td>
                          <button
                            className="icon-button"
                            aria-label={`Fișa bovinei ${c.ear_tag}`}
                            onClick={() => setSelected(c.id)}
                          >
                            <Icon name="chevron" size={16} />
                          </button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
                {!displayed.length && (
                  <div className="empty">
                    <span className="empty-icon">
                      <Icon
                        name={loading || filtering ? "refresh" : "herd"}
                        size={32}
                      />
                    </span>
                    <h3>
                      {loading || filtering
                        ? "Se încarcă registrul…"
                        : cows.length
                          ? "Nicio bovină nu corespunde căutării"
                          : "Registrul tău începe aici"}
                    </h3>
                    <p>
                      {cows.length
                        ? "Încearcă altă crotalie sau modifică filtrele."
                        : "Adaugă prima bovină pentru a construi evidența fermei."}
                    </p>
                    {!cows.length && writable && !loading && !filtering && (
                      <button
                        className="secondary"
                        onClick={() => openEditor({ kind: "cow" })}
                      >
                        <Icon name="plus" size={17} />
                        Adaugă prima bovină
                      </button>
                    )}
                  </div>
                )}
              </div>
              <div className="table-footer">
                <span>
                  {visible.length
                    ? `${currentPage * 12 + 1}–${Math.min((currentPage + 1) * 12, visible.length)} din ${visible.length} bovine`
                    : "0 bovine"}
                </span>
                <div>
                  <button
                    className="pagination"
                    disabled={currentPage === 0}
                    aria-label="Pagina precedentă"
                    onClick={() => setPageIndex(currentPage - 1)}
                  >
                    <Icon
                      name="chevron"
                      size={15}
                      style={{ transform: "rotate(180deg)" }}
                    />
                  </button>
                  <span className="page-number">{currentPage + 1}</span>
                  <button
                    className="pagination"
                    disabled={(currentPage + 1) * 12 >= visible.length}
                    aria-label="Pagina următoare"
                    onClick={() => setPageIndex(currentPage + 1)}
                  >
                    <Icon name="chevron" size={15} />
                  </button>
                </div>
              </div>
            </section>
          )}

          {(page === "births" || page === "inseminations") && (
            <section className="panel">
              <div className="section-heading">
                <h2>
                  {page === "births"
                    ? "Istoricul fătărilor"
                    : "Istoricul montărilor"}
                </h2>
                <span className="muted">
                  {(page === "births" ? births : inseminations).length}{" "}
                  înregistrări
                </span>
              </div>
              <div className="table-scroll">
                <table>
                  <thead>
                    <tr>
                      <th>Data evenimentului</th>
                      <th>
                        {page === "births"
                          ? "Crotalia mamei"
                          : "Crotalia femelei"}
                      </th>
                      <th>
                        {page === "births"
                          ? "Viței asociați"
                          : "Crotalia masculului"}
                      </th>
                      <th>Acțiuni</th>
                    </tr>
                  </thead>
                  <tbody>
                    {[...(page === "births" ? births : inseminations)]
                      .sort((a, b) => b.date.localeCompare(a.date))
                      .map((item) => {
                        const isBirth = "mother_id" in item;
                        return (
                          <tr key={item.id}>
                            <td>{formatDate(item.date)}</td>
                            <td>
                              <button
                                className="tag-button"
                                onClick={() =>
                                  setSelected(
                                    isBirth ? item.mother_id : item.dam_id,
                                  )
                                }
                              >
                                {tag(isBirth ? item.mother_id : item.dam_id)}
                              </button>
                            </td>
                            <td>
                              {isBirth
                                ? cows
                                    .filter((c) => c.birth_id === item.id)
                                    .map((c) => c.ear_tag)
                                    .join(", ") || "Niciun vițel asociat"
                                : tag(item.sire_id)}
                            </td>
                            <td>
                              <div className="row-actions">
                                {writable && (
                                  <>
                                    <button
                                      className="icon-button"
                                      aria-label="Editează evenimentul"
                                      onClick={() =>
                                        openEditor(
                                          isBirth
                                            ? { kind: "birth", value: item }
                                            : {
                                                kind: "insemination",
                                                value: item,
                                              },
                                        )
                                      }
                                    >
                                      <Icon name="edit" size={17} />
                                    </button>
                                    <button
                                      className="icon-button danger-text"
                                      aria-label="Șterge evenimentul"
                                      onClick={() =>
                                        openEditor({
                                          kind: "delete",
                                          command: isBirth
                                            ? "delete_birth"
                                            : "delete_insemination",
                                          args: isBirth
                                            ? {
                                                birthId: item.id,
                                                farmId: session.user.farm_id,
                                              }
                                            : { inseminationId: item.id },
                                          label: `evenimentul din ${formatDate(item.date)}`,
                                        })
                                      }
                                    >
                                      <Icon name="trash" size={17} />
                                    </button>
                                  </>
                                )}
                              </div>
                            </td>
                          </tr>
                        );
                      })}
                  </tbody>
                </table>
              </div>
              {!(page === "births" ? births : inseminations).length && (
                <div className="empty">
                  <Icon name={page === "births" ? "leaf" : "heart"} size={34} />
                  <h3>Niciun eveniment înregistrat</h3>
                  <p>
                    Evenimentele adăugate vor apărea aici și în fișa bovinei.
                  </p>
                </div>
              )}
            </section>
          )}

          {page === "overview" && (
            <div className="overview-grid">
              <section className="panel">
                <div className="section-heading">
                  <h2>Structura efectivului</h2>
                  <span className="muted">Bovine prezente</span>
                </div>
                <div className="breed-bars">
                  {Object.entries(breeds).map(([key, label]) => {
                    const count = present.filter((c) => c.breed === key).length;
                    return (
                      <div key={key}>
                        <div>
                          <span>{label}</span>
                          <strong>{count} bovine</strong>
                        </div>
                        <div className="bar-track">
                          <span
                            style={{
                              width: `${present.length ? (count / present.length) * 100 : 0}%`,
                            }}
                          />
                        </div>
                      </div>
                    );
                  })}
                </div>
              </section>
              <section className="panel">
                <div className="section-heading">
                  <h2>Evenimente recente</h2>
                  <Icon name="calendar" />
                </div>
                <div className="event-list">
                  {[
                    ...births.map((b) => ({
                      key: `b${b.id}`,
                      title: "Fătare înregistrată",
                      cow: tag(b.mother_id),
                      date: b.date,
                    })),
                    ...inseminations.map((i) => ({
                      key: `i${i.id}`,
                      title: "Montare înregistrată",
                      cow: tag(i.dam_id),
                      date: i.date,
                    })),
                  ]
                    .sort((a, b) => b.date.localeCompare(a.date))
                    .slice(0, 6)
                    .map((item) => (
                      <div className="event" key={item.key}>
                        <span className="stat-icon green">
                          <Icon name="calendar" size={18} />
                        </span>
                        <div>
                          <strong>{item.title}</strong>
                          <small>{item.cow}</small>
                        </div>
                        <span>{formatDate(item.date)}</span>
                      </div>
                    ))}
                  {!births.length && !inseminations.length && (
                    <p className="muted">
                      Evenimentele fermei vor apărea aici.
                    </p>
                  )}
                </div>
              </section>
            </div>
          )}

          {page === "reports" && (
            <section className="panel report-panel">
              <div className="report-icon">
                <Icon name="chart" size={32} />
              </div>
              <h2>Situația efectivului</h2>
              <p>
                Exportă un registru Excel cu crotalii, rase, sex, date și
                numărul de montări și fătări.
              </p>
              <FilterFields
                value={filter}
                onChange={setFilter}
                date={reportDate}
                onDateChange={setReportDate}
                search={search}
                onSearchChange={setSearch}
                status={status}
                onStatusChange={setStatus}
                onReset={resetFilters}
              />
              <label className="checkbox-label">
                <input
                  type="checkbox"
                  checked={status === "present"}
                  onChange={(event) =>
                    setStatus(event.target.checked ? "present" : "all")
                  }
                />
                Doar bovinele prezente în fermă la data de referință
              </label>
              <p className="form-hint">
                Debifează pentru a include și bovinele ieșite din fermă.
                Celelalte filtre rămân aplicate.
              </p>
              {(filterError || filterLoadError) && (
                <div className="alert error" role="alert">
                  {filterError || filterLoadError}
                </div>
              )}
              <div className="report-summary">
                <Icon name="folder" />
                <span>
                  Raportul folosește criteriile de mai sus și include toate
                  înregistrările corespunzătoare.
                </span>
              </div>
              <button
                className="primary"
                disabled={!!filterError || busy}
                onClick={exportReport}
              >
                <Icon name="download" size={18} />
                Exportă în Excel
              </button>
            </section>
          )}
          <footer className="workspace-footer">
            <span>
              <span className="status-dot" />
              {loading
                ? "Se actualizează datele…"
                : "Evidența fermei · Stocare locală"}
            </span>
            <span>Grijă pentru animale. Claritate pentru tine.</span>
          </footer>
        </main>
      </div>

      {selectedCow && !editor && (
        <Modal
          title={selectedCow.ear_tag}
          subtitle="Fișa bovinei"
          onClose={() => setSelected(null)}
        >
          <div className="detail-banner">
            <span className="detail-cow">
              <Icon name="herd" size={42} />
            </span>
            <div>
              <h3>{breeds[selectedCow.breed]}</h3>
              <p>
                {sexes[selectedCow.sex]} · {selectedCow.category}
              </p>
            </div>
            <span
              className={`state-badge ${isPresent(selectedCow) ? "present" : "exited"}`}
            >
              {isPresent(selectedCow)
                ? "În fermă"
                : selectedCow.entry_date > today()
                  ? "Intrare viitoare"
                  : "Ieșită"}
            </span>
          </div>
          <dl className="detail-grid">
            {[
              ["Data nașterii", formatDate(selectedCow.birth_date)],
              ["Data intrării", formatDate(selectedCow.entry_date)],
              ["Data ieșirii", formatDate(selectedCow.exit_date)],
              [
                "Fătarea de origine",
                selectedCow.birth_id
                  ? `#${selectedCow.birth_id} · ${formatDate(births.find((b) => b.id === selectedCow.birth_id)?.date || null)} · ${tag(births.find((b) => b.id === selectedCow.birth_id)?.mother_id ?? null)}`
                  : "Neasociată",
              ],
              ["Montări / însămânțări", selectedCow.insemination_count],
              ["Fătări", selectedCow.birth_count],
              ["Fermă", session.farm_name],
            ].map(([label, value]) => (
              <div key={label}>
                <dt>{label}</dt>
                <dd>{value}</dd>
              </div>
            ))}
          </dl>
          <h3 className="detail-section-title">Istoric de reproducție</h3>
          <div className="detail-events">
            {[
              ...births
                .filter((b) => b.mother_id === selectedCow.id)
                .map((b) => ({
                  key: `b${b.id}`,
                  text: "Fătare",
                  date: b.date,
                })),
              ...inseminations
                .filter(
                  (i) =>
                    i.dam_id === selectedCow.id || i.sire_id === selectedCow.id,
                )
                .map((i) => ({
                  key: `i${i.id}`,
                  text: "Montare / însămânțare",
                  date: i.date,
                })),
            ]
              .sort((a, b) => b.date.localeCompare(a.date))
              .map((event) => (
                <div key={event.key}>
                  <span>{event.text}</span>
                  <strong>{formatDate(event.date)}</strong>
                </div>
              ))}
            {!births.some((b) => b.mother_id === selectedCow.id) &&
              !inseminations.some(
                (i) =>
                  i.dam_id === selectedCow.id || i.sire_id === selectedCow.id,
              ) && <p className="muted">Nu sunt înregistrate evenimente.</p>}
          </div>
          {writable && (
            <div className="modal-actions">
              <button
                className="danger-button"
                onClick={() =>
                  openEditor({
                    kind: "delete",
                    command: "delete_cow",
                    args: {
                      cowId: selectedCow.id,
                      farmId: session.user.farm_id,
                    },
                    label: `bovina ${selectedCow.ear_tag}`,
                  })
                }
              >
                <Icon name="trash" size={17} />
                Șterge bovina
              </button>
              <button
                className="primary"
                onClick={() => openEditor({ kind: "cow", value: selectedCow })}
              >
                <Icon name="edit" size={17} />
                Editează datele
              </button>
            </div>
          )}
        </Modal>
      )}

      {editor && (
        <Modal
          title={
            editor.kind === "cow"
              ? editor.value
                ? "Editează bovina"
                : "Adaugă o bovină"
              : editor.kind === "birth"
                ? editor.value
                  ? "Editează fătarea"
                  : "Adaugă o fătare"
                : editor.kind === "insemination"
                  ? editor.value
                    ? "Editează montarea"
                    : "Adaugă o montare"
                  : "Șterge înregistrarea"
          }
          subtitle={
            editor.kind === "delete"
              ? "Verifică înregistrarea înainte de a continua."
              : "O evidență completă începe cu date corecte."
          }
          busy={busy}
          onClose={() => setEditor(null)}
        >
          {modalError && (
            <div className="alert error" role="alert">
              {modalError}
            </div>
          )}
          {editor.kind === "delete" ? (
            <>
              <p className="delete-description">
                Sigur dorești să ștergi {editor.label}?{" "}
                {editor.command === "delete_cow" &&
                  "Pentru femele se elimină montările și fătările asociate, iar vițeii rămân fără legătura cu fătarea. Pentru masculi se elimină doar asocierea lor din montări."}
                {editor.command === "delete_birth" &&
                  "Vițeii asociați vor rămâne în registru, fără legătura cu această fătare."}
              </p>
              <div className="modal-actions">
                <button
                  className="secondary"
                  disabled={busy}
                  onClick={() => setEditor(null)}
                >
                  Renunță
                </button>
                <button
                  className="danger-button solid"
                  disabled={busy}
                  onClick={() =>
                    mutate(
                      () => command(editor.command, editor.args),
                      "Înregistrarea a fost ștearsă.",
                      true,
                    )
                  }
                >
                  {busy ? "Se șterge…" : "Șterge înregistrarea"}
                </button>
              </div>
            </>
          ) : (
            <form onSubmit={saveEditor} noValidate>
              <fieldset disabled={busy}>
                {editor.kind === "cow" && (
                  <>
                    <div className="form-grid">
                      <label className="span-two">
                        Crotalie <span>*</span>
                        <input
                          name="ear_tag"
                          required
                          maxLength={30}
                          defaultValue={editor.value?.ear_tag}
                          placeholder="De exemplu, RO500010635363"
                          autoFocus
                        />
                      </label>
                      <label>
                        Sex <span>*</span>
                        <select
                          name="sex"
                          defaultValue={editor.value?.sex || "Female"}
                        >
                          <Options values={sexes} />
                        </select>
                      </label>
                      <label>
                        Rasă <span>*</span>
                        <select
                          name="breed"
                          defaultValue={
                            editor.value?.breed || "BaltataRomaneasca"
                          }
                        >
                          <Options values={breeds} />
                        </select>
                      </label>
                      <label>
                        Categorie <span>*</span>
                        <select
                          name="category"
                          defaultValue={editor.value?.category || "Carne"}
                        >
                          <Options values={categories} />
                        </select>
                      </label>
                      <label>
                        Data nașterii <span>*</span>
                        <DateInput
                          
                          name="birth_date"
                          required
                          max={today()}
                          defaultValue={editor.value?.birth_date}
                        />
                      </label>
                      <label>
                        Data intrării <span>*</span>
                        <DateInput
                          
                          name="entry_date"
                          required
                          defaultValue={editor.value?.entry_date || today()}
                        />
                      </label>
                      <label>
                        Data ieșirii
                        <DateInput
                          
                          name="exit_date"
                          defaultValue={editor.value?.exit_date || ""}
                        />
                      </label>
                    </div>
                    <p className="form-hint">
                      * Câmpuri obligatorii. Numărul de montări și fătări se
                      calculează automat din evenimente.
                    </p>
                  </>
                )}
                {(editor.kind === "birth" ||
                  editor.kind === "insemination") && (
                  <div className="form-grid">
                    <label className="span-two">
                      {editor.kind === "birth" ? "Mama" : "Femela"}{" "}
                      <span>*</span>
                      <select
                        name="mother"
                        required
                        defaultValue={
                          editor.value
                            ? "mother_id" in editor.value
                              ? editor.value.mother_id
                              : editor.value.dam_id
                            : ""
                        }
                      >
                        <option value="">Selectează o femelă</option>
                        {cows
                          .filter((c) => c.sex === "Female")
                          .map((c) => (
                            <option key={c.id} value={c.id!}>
                              {c.ear_tag} · {breeds[c.breed]}
                            </option>
                          ))}
                      </select>
                    </label>
                    {editor.kind === "insemination" && (
                      <label className="span-two">
                        Masculul (opțional)
                        <select
                          name="sire"
                          defaultValue={editor.value?.sire_id || ""}
                        >
                          <option value="">
                            Nespecificat / însămânțare artificială
                          </option>
                          {cows
                            .filter((c) => c.sex === "Male")
                            .map((c) => (
                              <option key={c.id} value={c.id!}>
                                {c.ear_tag}
                              </option>
                            ))}
                        </select>
                      </label>
                    )}
                    <label className="span-two">
                      Data evenimentului <span>*</span>
                      <DateInput
                        name="date"
                        
                        required
                        max={today()}
                        defaultValue={editor.value?.date || today()}
                      />
                    </label>
                  </div>
                )}
                <div className="modal-actions">
                  <button
                    type="button"
                    className="secondary"
                    onClick={() => setEditor(null)}
                    disabled={busy}
                  >
                    Renunță
                  </button>
                  <button className="primary" disabled={busy}>
                    {busy ? "Se salvează…" : "Salvează datele"}
                    <Icon name="check" size={17} />
                  </button>
                </div>
              </fieldset>
            </form>
          )}
        </Modal>
      )}
    </div>
  );
}
export default App;
