# Ferma · Manager de bovine

Aplicație desktop Tauri, cu interfață React și TypeScript în limba română și bază de date SQLite administrată de backendul Rust.

## Pornire

```powershell
npm.cmd install
npm.cmd run tauri dev
```

Sunt necesare Node.js, Rust și instrumentele de compilare pentru Tauri pe Windows. Comanda `npm.cmd run dev` pornește doar serverul pentru interfață; accesul la date necesită fereastra Tauri.

La prima pornire, alege „Creează o fermă”. Sesiunea se păstrează între porniri până la deconectare. Datele fermei sunt salvate în directorul local al aplicației, prin backendul existent.

## Generarea aplicației Windows

Rulează `npm.cmd run build:desktop` pentru compilarea versiunii finale și a instalatorului în română. Instalatorul se găsește în `src-tauri/target/release/bundle/nsis/`, iar executabilul aplicației în `src-tauri/target/release/cowmanagementservice.exe`.

Pentru a genera doar executabilul, rulează `npm.cmd run build:exe`. Interfața este inclusă în executabil; nu trebuie pornit Vite și nu sunt necesare Node.js sau Rust pe calculatorul familiei. WebView2 trebuie să fie instalat; instalatorul îl descarcă dacă lipsește, caz în care este necesară conexiunea la internet.

Identificatorul aplicației rămâne `com.lenovo.cowmanagementservice`, astfel încât baza de date existentă continuă să fie folosită. Datele nu sunt incluse în instalator și nu sunt transferate automat pe alt calculator.

## Funcționalități

- „Extras efectiv”: tabel cu masculi, femele și totaluri pe trei grupe de vârstă, pentru întregul efectiv prezent la data aleasă. La exact 6 luni sau 2 ani, bovina se află în grupa intermediară. Listele de crotalii pe grupe pot fi deschise sub tabel.
- Registru cu crotalie, sex, rasă, categorie, datele nașterii/intrării/ieșirii, numărul de montări și fătări și starea bovinei.
- Căutare după orice parte din crotalie, filtre după rasă, sex, categorie, anul nașterii, „Vârsta sub” și „Vârsta peste” în luni împlinite, data intrării și data ieșirii. „Vârsta peste” include limita introdusă (>=), iar „Vârsta sub” o exclude (<). Data de referință se aplică vârstei și stării bovinei. Registrul și exportul folosesc aceleași filtre, cu sortare și paginare în registru.
- Fișa bovinei cu toate atributele modelului Rust, inclusiv identificatorii, fătarea de origine și istoricul de reproducție.
- Adăugare, editare și ștergere a bovinelor, montărilor și fătărilor; asocierea vițeilor prin fătarea de origine.
- Anularea și refacerea modificărilor în sesiunea curentă.
- Statistici și export Excel cu filtre și dată de referință. La export se deschide fereastra nativă „Salvare ca”, unde alegi dosarul și numele fișierului `.xlsx`. Închiderea dialogului fără salvare anulează exportul.
- Mesaje de validare și erori în română, navigare cu tastatura și formulare modale.

Interfața apelează comenzile Rust prin `@tauri-apps/api/core`. Nu conține date demonstrative și nu folosește un server HTTP pentru operațiile fermei.

## Verificări

```powershell
npm.cmd run build
cargo test --manifest-path src-tauri/Cargo.toml --test frontend_workflows
```

Testele folosesc o bază de date temporară și verifică operațiile de reproducție cu identificatori diferiți de identificatorul fermei, păstrarea relațiilor la anulare/refacere și generarea unui raport Excel filtrat.

## Manual updates

Automatic updates are not configured. Keep the source and installer private and
send the installer directly to family members when releasing a new version.

1. Increase the version in `package.json`, `src-tauri/Cargo.toml`, and
   `src-tauri/tauri.conf.json` together (for example, `0.1.0` to `0.1.1`).
   Run `npm.cmd install --package-lock-only` to refresh the npm lockfile.
2. Run `npm.cmd run build:desktop` from the project directory. This builds both
   the frontend and Rust backend and generates the Romanian Windows installer.
3. Send the new `-setup.exe` from `src-tauri/target/release/bundle/nsis/` privately
   to the target computer.
4. Close Ferma and run the installer using the same Windows account as the
   existing installation. Reopen Ferma after installation and check the version's
   changes and existing farm records.

Keep the identifier `com.lenovo.cowmanagementservice` unchanged. The database is
stored separately from the installed application and is reused during updates;
do not delete the application data folder. Existing versions without an updater
can be updated using this same installer process.
