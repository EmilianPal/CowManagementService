import { command } from "../../lib/api";
import type { SavedAccount, Session, User } from "./types";

export const auth = {
  savedAccounts: () => command<SavedAccount[]>("saved_accounts"),
  remember: () => command<void>("remember_account"),
  switchAccount: (userId: number) => command<Session>("switch_account", { userId }),
  forget: (userId: number) => command<void>("forget_account", { userId }),
  session: () => command<Session | null>("get_session"),
  login: (username: string, password: string) =>
    command<User>("login_user", { username, password }),
  register: (
    farmName: string,
    username: string,
    email: string,
    password: string,
  ) => command<User>("register_admin", { farmName, username, email, password }),
  logout: (keepRemembered = false) => command<void>("logout", { keepRemembered }),
};
