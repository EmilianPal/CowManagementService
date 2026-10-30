import { CowDetailsModal } from "./features/herd/CowDetailsModal";
import { EditorModal } from "./features/herd/EditorModal";
import { ReproductionHistory } from "./features/herd/ReproductionHistory";
import { RegistryPage } from "./features/herd/RegistryPage";
import { OverviewPage } from "./features/herd/OverviewPage";
import { ReportsPage } from "./features/herd/ReportsPage";
import { prepareEditorSave } from "./features/herd/prepareEditorSave";
import type { Editor } from "./features/herd/editorTypes";
import { HerdExits } from "./features/herd/HerdExits";
import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type FormEvent,
} from "react";
import { command, romanianError } from "./lib/api";
import { auth } from "./features/auth/services";
import { AccountList } from "./features/auth/AccountList";
import { AuthScreen } from "./features/auth/AuthScreen";
import { Sidebar } from "./layout/Sidebar";
import { Topbar, PageHeading, FarmStats, WorkspaceFooter } from "./layout/WorkspaceChrome";
import type { Page } from "./layout/navigation";
import type { SavedAccount, Session } from "./features/auth/types";
import {
  today,
  formatDate,
  isPresent,
  type Cow,
  type Birth,
  type Insemination,
} from "./features/herd/types";
import { Icon } from "./components/Icon";
import { Modal } from "./components/Modal";
import { HerdExtract } from "./features/herd/HerdExtract";
import {
  emptyFilter,
  validateFilters,
  toCowFilter,
} from "./features/herd/filters";
import "./App.css";

function App() {
  const [session, setSession] = useState<Session | null>(null);
  const [savedAccounts, setSavedAccounts] = useState<SavedAccount[]>([]);
  const [accountPicker, setAccountPicker] = useState(false);
  const [showLoginForm, setShowLoginForm] = useState(false);
  const sessionGeneration = useRef(0);
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
  const [eventDate, setEventDate] = useState(today());
  const [eventMotherId, setEventMotherId] = useState<number | null>(null);
  const [selectedCalfIds, setSelectedCalfIds] = useState<number[]>([]);
  const [modalError, setModalError] = useState("");
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
    const generation = sessionGeneration.current;
    setLoading(true);
    try {
      const [c, b, i, h] = await Promise.all([
        command<Cow[]>("get_cows"),
        command<Birth[]>("get_births"),
        command<Insemination[]>("get_inseminations"),
        command<{ undo: boolean; redo: boolean }>("get_history_state"),
      ]);
      if (generation !== sessionGeneration.current) return;
      setCows(c);
      setBirths(b);
      setInseminations(i);
      setHistory(h);
    } finally {
      if (generation === sessionGeneration.current) setLoading(false);
    }
  }, []);
  useEffect(() => {
    let active = true;
    auth.savedAccounts().then((accounts) => { if (active) setSavedAccounts(accounts); }).catch((e) => { if (active) setError(romanianError(e)); });
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
    if (!notice) return;
    const timer = setTimeout(() => setNotice(""), 6000);
    return () => clearTimeout(timer);
  }, [notice]);
  useEffect(() => {
    setFilteredCows([]);
    setFilterLoadError("");
    if (!session || page !== "herd" || filterError) {
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
  }, [session, page, queryFilter, cows, filterError]);
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
  const present = cows.filter((c) => isPresent(c));
  const selectedCow = cows.find((c) => c.id === selected);
  const tag = (id: number | null) =>
    cows.find((c) => c.id === id)?.ear_tag || "Nespecificat";
  const openEditor = (value: Editor) => {
    setSelectedCalfIds(value.kind === "birth" && value.value
      ? cows.filter(c => c.birth_id === value.value?.id).flatMap(c => c.id === null ? [] : [c.id]) : []);
    if (value.kind === "birth" || value.kind === "insemination") {
      setEventDate(value.value?.date ?? today());
      setEventMotherId(value.cow?.sex === "Female" ? value.cow.id : value.value ? "mother_id" in value.value ? value.value.mother_id : value.value.dam_id : null);
    }
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
      return false;
    } finally {
      setBusy(false);
    }
    return true;
  }
  function changeSession(next: Session | null) {
    sessionGeneration.current += 1;
    setCows([]);
    setBirths([]);
    setInseminations([]);
    setFilteredCows([]);
    setHistory({ undo: false, redo: false });
    setSelected(null);
    setEditor(null);
    setError("");
    setModalError("");
    setNotice("");
    setFilterLoadError("");
    setLoading(false);
    resetFilters();
    setSort("tag");
    setShowFilters(false);
    setPage("herd");
    setAccountPicker(false);
    setSession(next);
  }
  async function chooseAccount(id: number) {
    setBusy(true);
    setError("");
    try { changeSession(await auth.switchAccount(id)); }
    catch (e) { setError(romanianError(e)); }
    finally { setBusy(false); }
  }
  async function forgetAccount(id: number) {
    setBusy(true);
    setError("");
    try {
      await auth.forget(id);
      setSavedAccounts(await auth.savedAccounts());
    } catch (e) { setError(romanianError(e)); }
    finally { setBusy(false); }
  }
  async function openAccounts() {
    setBusy(true);
    try {
      await auth.remember();
      setSavedAccounts(await auth.savedAccounts());
      setError("");
      setAccountPicker(true);
    } catch (e) { setError(romanianError(e)); }
    finally { setBusy(false); }
  }
  const accountList = <AccountList accounts={savedAccounts} activeUserId={session?.user.id}
    busy={busy} onChoose={chooseAccount} onForget={forgetAccount} />;
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
      const next = await auth.session();
      await auth.remember();
      changeSession(next);
      setSavedAccounts(await auth.savedAccounts());
    } catch (e) {
      setError(romanianError(e));
    } finally {
      setBusy(false);
    }
  }
  async function saveEditor(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    if (!editor || !session) return;
    if (!e.currentTarget.checkValidity()) {
      setModalError("Completează toate câmpurile obligatorii cu valori valide.");
      return;
    }
    const plan = prepareEditorSave({
      editor,
      data: new FormData(e.currentTarget),
      cows,
      births,
      inseminations,
      selectedCalfIds,
      farmId: session.user.farm_id,
    });
    if ("error" in plan) {
      setModalError(plan.error);
      return;
    }
    await mutate(() => command(plan.commandName, plan.args), plan.notice, true);
  }
  async function logout() {
    setBusy(true);
    try {
      await auth.logout();
      changeSession(null);
      setSavedAccounts(await auth.savedAccounts());
      setShowLoginForm(false);
      setAuthMode("login");
    } catch (e) {
      setError(romanianError(e));
    } finally {
      setBusy(false);
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
  if (!session) return <AuthScreen mode={authMode} error={error} busy={busy}
    savedAccountsCount={savedAccounts.length} showLoginForm={showLoginForm} accounts={accountList}
    onSubmit={authenticate} onToggleLoginForm={() => setShowLoginForm(!showLoginForm)}
    onToggleMode={() => { setAuthMode(authMode === "login" ? "register" : "login"); setError(""); }} />;

  return (
    <div className="app-shell">
      <Sidebar session={session} page={page} presentCount={present.length}
        busy={busy} loading={loading} onNavigate={next => { setPage(next); setSelected(null); }}
        onAccounts={openAccounts} onLogout={logout} />
      <div className="workspace">
        <Topbar page={page} />
        <main
          className={`main-content ${page === "extract" ? "extract-page" : ""}`}
        >
          <PageHeading page={page} writable={writable} busy={busy}
            onExport={() => setPage("reports")}
            onAdd={() => openEditor({ kind: page === "births" ? "birth" : page === "inseminations" ? "insemination" : "cow" })} />
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
          {page !== "extract" && <FarmStats present={present} births={births} loading={loading} />}

          {page === "exits" && <HerdExits writable={writable} onSaved={refresh} />}
          {page === "extract" && <HerdExtract />}
          {page === "herd" && <RegistryPage cows={cows} visible={visible} selected={selected}
            setSelected={setSelected} writable={writable} busy={busy} loading={loading} filtering={filtering}
            history={history} status={status} setStatus={setStatus} reportDate={reportDate}
            setReportDate={setReportDate} filter={filter} setFilter={setFilter} search={search}
            setSearch={setSearch} showFilters={showFilters} setShowFilters={setShowFilters}
            sort={sort} setSort={setSort} filterError={filterError} filterLoadError={filterLoadError}
            resetFilters={resetFilters} openEditor={openEditor}
            onUndo={() => { void mutate(() => command("undo"), "Ultima modificare a fost anulată."); }}
            onRedo={() => { void mutate(() => command("redo"), "Modificarea a fost refăcută."); }}
            onRefresh={() => { void refresh().then(() => setError("")).catch(e => setError(romanianError(e))); }} />}

          {(page === "births" || page === "inseminations") && (
            <ReproductionHistory kind={page} births={births} inseminations={inseminations}
              cows={cows} writable={writable} onSelect={setSelected}
              onEditBirth={value => openEditor({ kind: "birth", value })}
              onEditInsemination={value => openEditor({ kind: "insemination", value })}
              onDeleteBirth={birth => openEditor({ kind: "delete", command: "delete_birth",
                args: { birthId: birth.id, farmId: session.user.farm_id }, label: `fătarea din ${formatDate(birth.date)}` })}
              onDeleteInsemination={event => openEditor({ kind: "delete", command: "delete_insemination",
                args: { inseminationId: event.id }, label: `monta din ${formatDate(event.date)}` })}
            />
          )}

          {page === "overview" && <OverviewPage present={present} births={births}
            inseminations={inseminations} tag={tag} />}

          {page === "reports" && <ReportsPage filter={filter} setFilter={setFilter}
            reportDate={reportDate} setReportDate={setReportDate} search={search} setSearch={setSearch}
            status={status} setStatus={setStatus} resetFilters={resetFilters} filterError={filterError}
            filterLoadError={filterLoadError} busy={busy} onExport={exportReport} />}
          <WorkspaceFooter loading={loading} />
        </main>
      </div>

      {accountPicker && (
        <Modal title="Schimbă contul" subtitle="Alege contul pe care vrei să îl folosești." busy={busy} onClose={() => setAccountPicker(false)}>
          {error && <div className="alert error" role="alert">{error}</div>}
          {accountList}
          <p className="form-hint account-hint">Autentifică-te o dată în fiecare cont, apoi comută fără parolă, inclusiv după repornirea aplicației. Eliminarea din listă nu șterge datele fermei.</p>
          <div className="modal-actions">
            <button className="secondary" disabled={busy} onClick={() => setAccountPicker(false)}>Renunță</button>
            <button className="primary" disabled={busy} onClick={async () => {
              setBusy(true);
              try {
                await auth.logout(true);
                changeSession(null);
                setShowLoginForm(true);
                setAuthMode("login");
              } catch (e) { setError(romanianError(e)); }
              finally { setBusy(false); }
            }}>Adaugă un cont</button>
          </div>
        </Modal>
      )}
      {selectedCow && !editor && <CowDetailsModal key={selectedCow.id} cow={selectedCow}
        cows={cows} births={births} inseminations={inseminations}
        farmName={session.farm_name} farmId={session.user.farm_id}
        writable={writable} onClose={() => setSelected(null)} onOpenEditor={openEditor} />}
      {editor && <EditorModal editor={editor} busy={busy} modalError={modalError} cows={cows}
        eventDate={eventDate} eventMotherId={eventMotherId} selectedCalfIds={selectedCalfIds}
        setEventDate={setEventDate} setEventMotherId={setEventMotherId}
        setSelectedCalfIds={setSelectedCalfIds} setModalError={setModalError} tag={tag}
        onClose={() => setEditor(null)} onSave={saveEditor}
        onDelete={() => editor.kind === "delete" && mutate(
          () => command(editor.command, editor.args), "Înregistrarea a fost ștearsă.", true,
        )} />}
    </div>
  );
}
export default App;
