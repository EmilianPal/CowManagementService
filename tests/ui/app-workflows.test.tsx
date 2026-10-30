import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../../src/App";
import type { Cow } from "../../src/features/herd/types";
import type { Session } from "../../src/features/auth/types";
import { today } from "../../src/features/herd/types";

const mocks = vi.hoisted(() => ({
  command: vi.fn(),
  auth: {
    savedAccounts: vi.fn(), session: vi.fn(), remember: vi.fn(), login: vi.fn(), register: vi.fn(),
    switchAccount: vi.fn(), forget: vi.fn(), logout: vi.fn(),
  },
}));
vi.mock("../../src/lib/api", () => ({ command: mocks.command, romanianError: (error: unknown) => String(error) }));
vi.mock("../../src/features/auth/services", () => ({ auth: mocks.auth }));
vi.mock("@tauri-apps/api/core", () => ({ isTauri: () => true }));

const session = (role: Session["user"]["role"] = "Editor", farm = "Emisadi", id = 1): Session => ({
  farm_name: farm,
  user: { id, farm_id: id, username: `user${id}`, email: `user${id}@example.com`, role },
});
const cow = (id: number, tag: string, birth_date = "2024-01-01"): Cow => ({
  id, farm_id: 1, ear_tag: tag, sex: "Female", breed: "Metis", category: "Carne",
  birth_date, entry_date: birth_date, exit_date: null, birth_id: null,
  birth_count: 0, insemination_count: 0,
});
const herd = [cow(1, "RO-Z", "2023-01-01"), cow(2, "RO-A", "2024-01-01")];

beforeEach(() => {
  Object.values(mocks.auth).forEach(mock => mock.mockReset());
  mocks.command.mockReset();
  mocks.auth.savedAccounts.mockResolvedValue([]);
  mocks.auth.session.mockResolvedValue(session());
  mocks.auth.remember.mockResolvedValue(undefined);
  mocks.command.mockImplementation(async (name: string) => {
    if (name === "get_cows" || name === "get_cows_filtered") return herd;
    if (name === "get_births" || name === "get_inseminations") return [];
    if (name === "get_history_state") return { undo: false, redo: false };
    if (name === "export_report") return "C:/Reports/herd.xlsx";
    return true;
  });
});

describe("application workflows", () => {
  it("authenticates from the login form and opens the registry", async () => {
    mocks.auth.session.mockResolvedValueOnce(null).mockResolvedValue(session());
    mocks.auth.login.mockResolvedValue(session().user);
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Autentifică-te" }));
    expect(screen.getByRole("alert").textContent).toContain("Completează");
    fireEvent.change(screen.getByPlaceholderText("Numele tău de utilizator"), { target: { value: "  user1  " } });
    fireEvent.change(screen.getByPlaceholderText("Introdu parola"), { target: { value: "secret123" } });
    fireEvent.click(screen.getByRole("button", { name: "Autentifică-te" }));
    await waitFor(() => expect(mocks.auth.login).toHaveBeenCalledWith("user1", "secret123"));
    expect(await screen.findByRole("heading", { name: "Registrul bovinelor" })).toBeTruthy();
  });

  it("validates farm registration before calling the authentication service", async () => {
    mocks.auth.session.mockResolvedValueOnce(null).mockResolvedValue(session());
    mocks.auth.register.mockResolvedValue(session().user);
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: /Prima utilizare/ }));
    fireEvent.change(screen.getByPlaceholderText("De exemplu, Ferma de la Deal"), { target: { value: "Ferma Test" } });
    fireEvent.change(screen.getByPlaceholderText("nume@exemplu.ro"), { target: { value: "invalid" } });
    fireEvent.change(screen.getByPlaceholderText("Numele tău de utilizator"), { target: { value: "owner" } });
    fireEvent.change(screen.getByPlaceholderText("Cel puțin 8 caractere"), { target: { value: "long-password" } });
    fireEvent.click(screen.getByRole("button", { name: "Creează ferma" }));
    expect(mocks.auth.register).not.toHaveBeenCalled();
    fireEvent.change(screen.getByPlaceholderText("nume@exemplu.ro"), { target: { value: "owner@example.com" } });
    fireEvent.click(screen.getByRole("button", { name: "Creează ferma" }));
    await waitFor(() => expect(mocks.auth.register).toHaveBeenCalledWith("Ferma Test", "owner", "owner@example.com", "long-password"));
  });

  it("sorts the registry, opens cattle details, and hides editing for viewers", async () => {
    mocks.auth.session.mockResolvedValue(session("Viewer"));
    render(<App />);
    const table = await screen.findByRole("table");
    await waitFor(() => expect(within(table).getAllByRole("row")).toHaveLength(3));
    expect(within(table).getAllByRole("button", { name: /^RO-/ }).map(button => button.textContent)).toEqual(["RO-A", "RO-Z"]);
    fireEvent.change(screen.getByLabelText("Sortează bovinele"), { target: { value: "oldest" } });
    expect(within(table).getAllByRole("button", { name: /^RO-/ }).map(button => button.textContent)).toEqual(["RO-Z", "RO-A"]);
    fireEvent.click(within(table).getByRole("button", { name: "RO-Z" }));
    expect(screen.getByRole("tab", { name: "Date generale" })).toBeTruthy();
    fireEvent.click(screen.getByRole("tab", { name: "Fătări" }));
    expect(screen.getByRole("tabpanel")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Editează datele" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Adaugă o fătare" })).toBeNull();
  });

  it("passes the active report filters to Excel export and shows the saved path", async () => {
    render(<App />);
    await screen.findByRole("heading", { name: "Registrul bovinelor" });
    fireEvent.click(screen.getByRole("button", { name: "Rapoarte și export" }));
    fireEvent.change(screen.getByPlaceholderText("Orice parte din crotalie"), { target: { value: "RO-A" } });
    fireEvent.click(screen.getByRole("button", { name: "Exportă în Excel" }));
    await waitFor(() => expect(mocks.command).toHaveBeenCalledWith("export_report", {
      filter: expect.objectContaining({ ear_tag_contains: "RO-A", show_only_entered: true, show_only_exited: false }),
    }));
    expect(await screen.findByRole("status")).toHaveProperty("textContent", expect.stringContaining("C:/Reports/herd.xlsx"));
  });

  it("shows current herd structure and recent reproduction events on the overview", async () => {
    mocks.command.mockImplementation(async (name: string) => {
      if (name === "get_cows" || name === "get_cows_filtered") return herd;
      if (name === "get_births") return [{ id: 10, farm_id: 1, mother_id: 1, date: "2024-01-07" }];
      if (name === "get_inseminations") return [{ id: 20, farm_id: 1, dam_id: 1, sire_id: null, date: "2024-03-01" }];
      if (name === "get_history_state") return { undo: false, redo: false };
      return true;
    });
    render(<App />);
    await screen.findByRole("heading", { name: "Registrul bovinelor" });
    fireEvent.click(screen.getByRole("button", { name: "Privire de ansamblu" }));
    expect(screen.getByRole("heading", { name: "Structura efectivului" })).toBeTruthy();
    expect(screen.getByText("2 bovine")).toBeTruthy();
    expect(screen.getByText("Fătare înregistrată")).toBeTruthy();
    expect(screen.getByText("Montă înregistrată")).toBeTruthy();
  });

  it("switches accounts and reloads cattle in the new session", async () => {
    mocks.auth.savedAccounts.mockResolvedValue([{ id: 1, username: "user1", farm_name: "Emisadi" }, { id: 2, username: "user2", farm_name: "Ferma nouă" }]);
    mocks.auth.switchAccount.mockResolvedValue(session("Editor", "Ferma nouă", 2));
    render(<App />);
    await screen.findByRole("heading", { name: "Registrul bovinelor" });
    fireEvent.click(screen.getByRole("button", { name: "Schimbă contul" }));
    fireEvent.click(await screen.findByRole("button", { name: /user2.*Ferma nouă/ }));
    await waitFor(() => expect(mocks.auth.switchAccount).toHaveBeenCalledWith(2));
    expect(screen.getByText("Ferma nouă")).toBeTruthy();
    await waitFor(() => expect(mocks.command.mock.calls.filter(call => call[0] === "get_cows")).toHaveLength(2));
  });

  it("ignores a late registry filter response from the previous account", async () => {
    let resolveOldFilter!: (value: Cow[]) => void;
    const oldFilter = new Promise<Cow[]>(resolve => { resolveOldFilter = resolve; });
    const newCow = { ...cow(9, "RO-NEW-FARM"), farm_id: 2 };
    let cowLoads = 0;
    let filterLoads = 0;
    mocks.auth.savedAccounts.mockResolvedValue([
      { id: 1, username: "user1", farm_name: "Emisadi" },
      { id: 2, username: "user2", farm_name: "Second farm" },
    ]);
    mocks.auth.switchAccount.mockResolvedValue(session("Editor", "Second farm", 2));
    mocks.command.mockImplementation(async (name: string) => {
      if (name === "get_cows") return ++cowLoads === 1 ? herd : [newCow];
      if (name === "get_cows_filtered") return ++filterLoads === 1 ? oldFilter : [newCow];
      if (name === "get_births" || name === "get_inseminations") return [];
      if (name === "get_history_state") return { undo: false, redo: false };
      return true;
    });
    render(<App />);
    await waitFor(() => expect(filterLoads).toBe(1));
    fireEvent.click(screen.getByRole("button", { name: "Schimbă contul" }));
    fireEvent.click(await screen.findByRole("button", { name: /user2.*Second farm/ }));
    await waitFor(() => expect(screen.getByRole("button", { name: "RO-NEW-FARM" })).toBeTruthy());
    resolveOldFilter(herd);
    await waitFor(() => expect(screen.queryByRole("button", { name: "RO-Z" })).toBeNull());
  });

  it("fixes the mother when adding a birth from her card and sends selected calves", async () => {
    const newborn = cow(3, "RO-NEWBORN", today());
    mocks.command.mockImplementation(async (name: string) => {
      if (name === "get_cows" || name === "get_cows_filtered") return [...herd, newborn];
      if (name === "get_births" || name === "get_inseminations") return [];
      if (name === "get_history_state") return { undo: false, redo: false };
      return true;
    });
    render(<App />);
    const table = await screen.findByRole("table");
    fireEvent.click(await within(table).findByRole("button", { name: "RO-Z" }));
    fireEvent.click(screen.getByRole("tab", { name: "Fătări" }));
    fireEvent.click(screen.getByRole("button", { name: "Adaugă o fătare" }));
    const dialog = screen.getByRole("dialog");
    expect(within(dialog).queryByLabelText(/Mama/)).toBeNull();
    fireEvent.focus(within(dialog).getByRole("combobox"));
    expect(within(dialog).queryByRole("option", { name: /RO-Z/ })).toBeNull();
    fireEvent.click(within(dialog).getByRole("option", { name: /RO-NEWBORN/ }));
    fireEvent.click(within(dialog).getByRole("button", { name: "Salvează datele" }));
    await waitFor(() => expect(mocks.command).toHaveBeenCalledWith("save_birth_with_calves", {
      birth: expect.objectContaining({ mother_id: 1, date: today() }), calfIds: [3],
    }));
  });

  it("updates a birth and removes calf selections when its date changes", async () => {
    const date = "2024-01-01";
    const linked = { ...cow(3, "RO-CALF", date), birth_id: 10 };
    mocks.command.mockImplementation(async (name: string) => {
      if (name === "get_cows" || name === "get_cows_filtered") return [...herd, linked];
      if (name === "get_births") return [{ id: 10, farm_id: 1, mother_id: 1, date }];
      if (name === "get_inseminations") return [];
      if (name === "get_history_state") return { undo: false, redo: false };
      return true;
    });
    render(<App />);
    await screen.findByRole("heading", { name: "Registrul bovinelor" });
    fireEvent.click(screen.getByRole("button", { name: "Fătări" }));
    fireEvent.click(await screen.findByRole("button", { name: /Editează fătarea/ }));
    const dialog = screen.getByRole("dialog");
    expect(within(dialog).getByText("RO-CALF")).toBeTruthy();
    fireEvent.change(within(dialog).getByPlaceholderText("zz.ll.aaaa"), { target: { value: "02.01.2024" } });
    fireEvent.click(within(dialog).getByRole("button", { name: "Salvează datele" }));
    await waitFor(() => expect(mocks.command).toHaveBeenCalledWith("save_birth_with_calves", {
      birth: expect.objectContaining({ id: 10, mother_id: 1, date: "2024-01-02" }), calfIds: [],
    }));
  });

  it("keeps the birth editor open after a failed save and does not refresh", async () => {
    mocks.command.mockImplementation(async (name: string) => {
      if (name === "get_cows" || name === "get_cows_filtered") return herd;
      if (name === "get_births" || name === "get_inseminations") return [];
      if (name === "get_history_state") return { undo: false, redo: false };
      if (name === "save_birth_with_calves") throw new Error("Duplicate birth");
      return true;
    });
    render(<App />);
    const table = await screen.findByRole("table");
    fireEvent.click(await within(table).findByRole("button", { name: "RO-Z" }));
    fireEvent.click(screen.getByRole("tab", { name: "Fătări" }));
    fireEvent.click(screen.getByRole("button", { name: "Adaugă o fătare" }));
    fireEvent.click(within(screen.getByRole("dialog")).getByRole("button", { name: "Salvează datele" }));
    expect(await within(screen.getByRole("dialog")).findByRole("alert")).toHaveProperty("textContent", "Error: Duplicate birth");
    expect(screen.getByRole("dialog")).toBeTruthy();
    expect(mocks.command.mock.calls.filter(call => call[0] === "get_cows")).toHaveLength(1);
  });

  it("refuses to change a mother's sex while she has a birth", async () => {
    mocks.command.mockImplementation(async (name: string) => {
      if (name === "get_cows" || name === "get_cows_filtered") return herd;
      if (name === "get_births") return [{ id: 10, farm_id: 1, mother_id: 1, date: "2024-01-07" }];
      if (name === "get_inseminations") return [];
      if (name === "get_history_state") return { undo: false, redo: false };
      return true;
    });
    render(<App />);
    const table = await screen.findByRole("table");
    fireEvent.click(await within(table).findByRole("button", { name: "RO-Z" }));
    fireEvent.click(screen.getByRole("button", { name: "Editează datele" }));
    const dialog = screen.getByRole("dialog");
    fireEvent.change(within(dialog).getByLabelText(/Sex/), { target: { value: "Male" } });
    fireEvent.click(within(dialog).getByRole("button", { name: "Salvează datele" }));
    expect(within(dialog).getByRole("alert").textContent).toContain("Nu poți schimba sexul");
    expect(mocks.command.mock.calls.some(call => call[0] === "update_cow")).toBe(false);
  });

  it("creates a calf with its mother through the combined cow and birth command", async () => {
    render(<App />);
    await screen.findByRole("heading", { name: "Registrul bovinelor" });
    fireEvent.click(screen.getByRole("button", { name: "Adaugă o bovină" }));
    const dialog = screen.getByRole("dialog");
    fireEvent.change(within(dialog).getByPlaceholderText("De exemplu, RO500010635363"), { target: { value: "ro-new-calf" } });
    fireEvent.change(within(dialog).getAllByPlaceholderText("zz.ll.aaaa")[0], { target: { value: "01.01.2024" } });
    const mother = within(dialog).getByPlaceholderText("Caută mama după crotalie");
    fireEvent.change(mother, { target: { value: "RO-Z" } });
    fireEvent.click(within(dialog).getByRole("option", { name: /RO-Z/ }));
    fireEvent.click(within(dialog).getByRole("button", { name: "Salvează datele" }));
    await waitFor(() => expect(mocks.command).toHaveBeenCalledWith("add_cow_with_birth", {
      cow: expect.objectContaining({ ear_tag: "RO-NEW-CALF", birth_date: "2024-01-01" }), motherId: 1,
    }));
  });

  it("invokes Undo from the registry and refreshes the records", async () => {
    mocks.command.mockImplementation(async (name: string) => {
      if (name === "get_cows" || name === "get_cows_filtered") return herd;
      if (name === "get_births" || name === "get_inseminations") return [];
      if (name === "get_history_state") return { undo: true, redo: false };
      return true;
    });
    render(<App />);
    const undo = await screen.findByRole("button", { name: "Anulează ultima modificare" });
    await waitFor(() => expect(undo).toHaveProperty("disabled", false));
    fireEvent.click(undo);
    await waitFor(() => expect(mocks.command).toHaveBeenCalledWith("undo"));
    await waitFor(() => expect(mocks.command.mock.calls.filter(call => call[0] === "get_cows")).toHaveLength(2));
  });

  it("opens and saves an existing insemination through the Monte screen", async () => {
    const sire = { ...cow(3, "RO-SIRE"), sex: "Male" as const };
    mocks.command.mockImplementation(async (name: string) => {
      if (name === "get_cows" || name === "get_cows_filtered") return [...herd, sire];
      if (name === "get_births") return [];
      if (name === "get_inseminations") return [{ id: 20, farm_id: 1, dam_id: 1, sire_id: 3, date: "2024-03-01" }];
      if (name === "get_history_state") return { undo: false, redo: false };
      return true;
    });
    render(<App />);
    await screen.findByRole("heading", { name: "Registrul bovinelor" });
    fireEvent.click(screen.getByRole("button", { name: "Monte" }));
    fireEvent.click(await screen.findByRole("button", { name: /Editează monta/ }));
    const dialog = screen.getByRole("dialog");
    expect(within(dialog).getByPlaceholderText("Caută un mascul sau lasă necompletat")).toHaveProperty("value", "RO-SIRE");
    fireEvent.click(within(dialog).getByRole("button", { name: "Salvează datele" }));
    await waitFor(() => expect(mocks.command).toHaveBeenCalledWith("update_insemination", {
      insemination: expect.objectContaining({ id: 20, dam_id: 1, sire_id: 3, date: "2024-03-01" }),
    }));
  });
});
