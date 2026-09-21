import { invoke, isTauri } from "@tauri-apps/api/core";

class LocalizedError extends Error {}

export function romanianError(error: unknown): string {
  if (error instanceof LocalizedError) return error.message;
  const message = String(error);
  if (/UNIQUE constraint failed: farms\.name/i.test(message))
    return "Există deja o fermă cu acest nume. Autentifică-te sau alege alt nume.";
  if (/UNIQUE constraint failed: users\.username/i.test(message))
    return "Acest nume de utilizator este deja folosit. Alege alt nume.";
  if (/UNIQUE constraint failed: users\.email/i.test(message))
    return "Această adresă de e-mail este deja folosită. Autentifică-te sau folosește altă adresă.";
  if (/no such table|no such column/i.test(message))
    return "Baza de date nu are structura necesară. Repornește aplicația după actualizare.";
  if (/database is locked|database is busy/i.test(message))
    return "Baza de date este ocupată. Așteaptă câteva secunde și încearcă din nou.";
  if (/readonly database|unable to open database/i.test(message))
    return "Aplicația nu poate accesa baza de date pentru salvare. Verifică permisiunile dosarului aplicației.";
  if (/Invalid username or password/i.test(message))
    return "Numele de utilizator sau parola este incorectă.";
  if (/UNIQUE constraint/i.test(message))
    return "Există deja o înregistrare cu aceste date. Verifică crotalia, utilizatorul sau data evenimentului.";
  if (/No session/i.test(message))
    return "Sesiunea a expirat. Autentifică-te din nou.";
  if (/FOREIGN KEY/i.test(message))
    return "Înregistrarea este asociată cu alte date. Verifică bovina și evenimentele selectate.";
  if (
    /nu |Nu |Selectează|Data |Crotalia|Parola|Completează|Aplicația|Înregistrarea|Există deja|Acest nume|Această adresă|Baza de date/.test(
      message,
    )
  )
    return message.replace(/^Error: /, "");
  return "Operația nu a putut fi finalizată. Verifică datele introduse și încearcă din nou.";
}

export async function command<T>(
  name: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (!isTauri())
    throw new Error(
      "Aplicația trebuie pornită în fereastra desktop Tauri pentru a accesa datele fermei.",
    );
  try {
    return await invoke<T>(name, args);
  } catch (error) {
    throw new LocalizedError(romanianError(error));
  }
}
