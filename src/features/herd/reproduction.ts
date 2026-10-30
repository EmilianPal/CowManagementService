import { formatDate, type Cow } from "./types";

export function availableCalves(cows: Cow[], motherId: number | null, currentCalfId?: number | null) {
  return cows.filter(cow => (cow.birth_id === null || cow.id === currentCalfId) && cow.id !== motherId);
}

export function calfAssociationError(calf: Cow, motherId: number | null, date: string, birthId?: number) {
  if (calf.id === motherId) return "Mama nu poate fi selectată ca vițel.";
  if (calf.birth_id !== null && calf.birth_id !== birthId) return "Vițelul este deja asociat unei alte fătări.";
  if (calf.birth_date !== date) {
    return `Vițelul ${calf.ear_tag} este născut la ${formatDate(calf.birth_date)}, iar data fătării este ${formatDate(date)}. Cele două date trebuie să coincidă.`;
  }
  return "";
}
