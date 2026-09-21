import { command } from "../../lib/api";
import type { Session, User } from "./types";

export const auth = {
  session: () => command<Session | null>("get_session"),
  login: (username: string, password: string) =>
    command<User>("login_user", { username, password }),
  register: (
    farmName: string,
    username: string,
    email: string,
    password: string,
  ) => command<User>("register_admin", { farmName, username, email, password }),
  logout: () => command<void>("logout"),
};
