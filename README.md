# Ferma · Manager de bovine

Aplicație desktop Tauri, cu interfață React și TypeScript în limba română și bază de date SQLite administrată de backendul Rust.

## Pornire

```powershell
npm.cmd install
npm.cmd run tauri dev
```

Sunt necesare Node.js, Rust și instrumentele de compilare pentru Tauri pe Windows. Comanda `npm.cmd run dev` pornește doar serverul pentru interfață; accesul la date necesită fereastra Tauri.

La prima pornire, alege „Creează o fermă”. Sesiunea se păstrează între porniri până la deconectare. Datele fermei sunt salvate în directorul local al aplicației, prin backendul existent.

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
