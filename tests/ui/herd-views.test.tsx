import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { HerdExits } from "../../src/features/herd/HerdExits";
import { HerdExtract } from "../../src/features/herd/HerdExtract";
import type { Cow } from "../../src/features/herd/types";

const mocks = vi.hoisted(() => ({ command: vi.fn(), romanianError: (error: unknown) => String(error) }));
vi.mock("../../src/lib/api", () => mocks);

const cow = (id: number, ear_tag: string, sex: Cow["sex"]): Cow => ({
  id, farm_id: 1, ear_tag, sex, breed: "Metis", category: "Carne",
  birth_date: "2024-01-01", entry_date: "2024-01-01", exit_date: null,
  birth_id: null, birth_count: 0, insemination_count: 0,
});

beforeEach(() => mocks.command.mockReset());

describe("herd exit screen", () => {
  it("keeps selected cows when searching and submits the full selection", async () => {
    const onSaved = vi.fn(async () => {});
    mocks.command.mockImplementation(async (name: string, args?: { filter?: { ear_tag_contains?: string } }) => {
      if (name === "get_cows_filtered") return args?.filter?.ear_tag_contains === "B" ? [cow(2, "RO-B", "Male")] : [cow(1, "RO-A", "Female"), cow(2, "RO-B", "Male")];
      if (name === "exit_cows") return true;
      return [];
    });
    render(<HerdExits writable onSaved={onSaved} />);
    const first = await screen.findByRole("checkbox", { name: "Selectează RO-A" });
    fireEvent.click(first);
    fireEvent.change(screen.getByPlaceholderText("Orice parte din crotalie"), { target: { value: "B" } });
    await waitFor(() => expect(mocks.command).toHaveBeenCalledWith("get_cows_filtered", expect.objectContaining({ filter: expect.objectContaining({ ear_tag_contains: "B" }) })));
    await waitFor(() => expect(screen.getByRole("checkbox", { name: "Selectează RO-A" })).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: /Înregistrează ieșirea \(1\)/ }));
    await waitFor(() => expect(mocks.command).toHaveBeenCalledWith("exit_cows", expect.objectContaining({ cowIds: [1] })));
    expect(onSaved).toHaveBeenCalledTimes(1);
  });

  it("disables write actions for a viewer", async () => {
    mocks.command.mockResolvedValue([cow(1, "RO-A", "Female")]);
    render(<HerdExits writable={false} onSaved={vi.fn()} />);
    const checkbox = await screen.findByRole("checkbox", { name: "Selectează RO-A" });
    expect(checkbox).toHaveProperty("disabled", true);
    expect(screen.getByRole("button", { name: /Înregistrează ieșirea/ })).toHaveProperty("disabled", true);
  });

  it("keeps the selection but prevents submitting after a filter query fails", async () => {
    mocks.command.mockImplementation(async (name: string, args?: { filter?: { ear_tag_contains?: string } }) => {
      if (name === "get_cows_filtered" && args?.filter?.ear_tag_contains) throw new Error("Query failed");
      if (name === "get_cows_filtered") return [cow(1, "RO-A", "Female")];
      return true;
    });
    render(<HerdExits writable onSaved={vi.fn()} />);
    fireEvent.click(await screen.findByRole("checkbox", { name: "Selectează RO-A" }));
    fireEvent.change(screen.getByPlaceholderText("Orice parte din crotalie"), { target: { value: "missing" } });
    expect(await screen.findByRole("alert")).toHaveProperty("textContent", "Error: Query failed");
    expect(screen.getByRole("checkbox", { name: "Selectează RO-A" })).toHaveProperty("checked", true);
    expect(screen.getByRole("button", { name: /Înregistrează ieșirea \(1\)/ })).toHaveProperty("disabled", true);
    expect(mocks.command.mock.calls.some(call => call[0] === "exit_cows")).toBe(false);
  });
});

describe("herd extract screen", () => {
  it("renders the returned census in age groups and the ear-tag details", async () => {
    mocks.command.mockResolvedValue([cow(1, "RO-A", "Female"), cow(2, "RO-B", "Male")]);
    render(<HerdExtract />);
    const summary = await screen.findByRole("table", { name: /Efectiv la/ });
    expect(within(summary).getByRole("row", { name: /Total efectiv/ }).textContent).toMatch(/1.*1.*2/);
    fireEvent.click(screen.getByText("Vezi crotaliile pe grupe"));
    expect(screen.getByText("RO-A")).toBeTruthy();
    expect(screen.getByText("RO-B")).toBeTruthy();
    expect(mocks.command).toHaveBeenCalledWith("get_cows_in_the_plantation", expect.objectContaining({ date: expect.any(String) }));
  });
});
