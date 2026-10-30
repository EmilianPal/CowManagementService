import { isTauri } from "@tauri-apps/api/core";
import type { FormEventHandler, ReactNode } from "react";

interface Props {
  mode: "login" | "register";
  error: string;
  busy: boolean;
  savedAccountsCount: number;
  showLoginForm: boolean;
  accounts: ReactNode;
  onSubmit: FormEventHandler<HTMLFormElement>;
  onToggleLoginForm: () => void;
  onToggleMode: () => void;
}

export function AuthScreen({ mode, error, busy, savedAccountsCount, showLoginForm, accounts,
  onSubmit, onToggleLoginForm, onToggleMode }: Props) {
  return (
    <main className="auth-layout">
      <section className="auth-panel" aria-labelledby="auth-title">
        <div className="auth-card">
          <h1 id="auth-title">Manager de bovine</h1>
          <p>{mode === "login" ? "Autentificare" : "Creează o fermă"}</p>
          {error && (
            <div className="alert error" role="alert">
              {error}
            </div>
          )}
          {mode === "login" && savedAccountsCount > 0 && (
            <>
              <p>Alege un cont pentru acces rapid</p>
              {accounts}
              <button className="secondary full" disabled={busy} onClick={() => onToggleLoginForm()}>
                {showLoginForm ? "Ascunde autentificarea" : "Adaugă un cont"}
              </button>
            </>
          )}
          <p className="account-hint">Conturile rămân disponibile până la deconectare sau eliminarea din accesul rapid.</p>
          {(mode === "register" || showLoginForm || savedAccountsCount === 0) && <form onSubmit={onSubmit} noValidate>
            <fieldset disabled={busy}>
              {mode === "register" && (
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
                    mode === "login" ? "current-password" : "new-password"
                  }
                  required
                  placeholder={
                    mode === "register"
                      ? "Cel puțin 8 caractere"
                      : "Introdu parola"
                  }
                />
              </label>
              <button className="primary full" disabled={busy || !isTauri()}>
                {busy
                  ? "Se verifică…"
                  : mode === "login"
                    ? "Autentifică-te"
                    : "Creează ferma"}
              </button>
            </fieldset>
          </form>}
          <button
            className="auth-switch"
            disabled={busy}
            onClick={onToggleMode}
          >
            {mode === "login"
              ? "Prima utilizare? Creează o fermă"
              : "Înapoi la autentificare"}
          </button>
        </div>
      </section>
    </main>
  );
}
