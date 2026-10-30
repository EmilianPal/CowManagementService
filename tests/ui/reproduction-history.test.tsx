import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ReproductionHistory } from "../../src/features/herd/ReproductionHistory";
import type { Birth, Cow, Insemination } from "../../src/features/herd/types";

const cow = (id: number, ear_tag: string, sex: Cow["sex"], birth_date: string, birth_id: number | null = null): Cow => ({
  id, farm_id: 1, ear_tag, sex, birth_date, birth_id, breed: "Metis", category: "Carne",
  entry_date: birth_date, exit_date: null, birth_count: 0, insemination_count: 0,
});
const mother = cow(1, "RO-MOTHER", "Female", "2020-01-01");
const father = cow(2, "RO-FATHER", "Male", "2020-01-01");
const twinA = cow(3, "RO-CALF-A", "Female", "2024-01-07", 10);
const twinB = cow(4, "RO-CALF-B", "Male", "2024-01-07", 10);
const births: Birth[] = [
  { id: 10, farm_id: 1, mother_id: 1, date: "2024-01-07" },
  { id: 11, farm_id: 1, mother_id: 1, date: "2023-04-29" },
];
const inseminations: Insemination[] = [
  { id: 20, farm_id: 1, dam_id: 1, sire_id: 2, date: "2024-01-15" },
  { id: 21, farm_id: 1, dam_id: 1, sire_id: null, date: "2023-04-12" },
];

function setup(kind: "births" | "inseminations", writable = true) {
  const callbacks = { onSelect: vi.fn(), onEditBirth: vi.fn(), onDeleteBirth: vi.fn(), onEditInsemination: vi.fn(), onDeleteInsemination: vi.fn() };
  render(<ReproductionHistory kind={kind} births={births} inseminations={inseminations}
    cows={[mother, father, twinA, twinB]} writable={writable} {...callbacks} />);
  return callbacks;
}

describe("reproduction history interface", () => {
  it("shows twins on separate rows with their identification data and filters by event month/year", () => {
    const callbacks = setup("births");
    const table = screen.getByRole("table");
    expect(within(table).getAllByRole("row")).toHaveLength(4);
    expect(within(table).getByText("RO-CALF-A").closest("tr")?.textContent).toContain("RO-MOTHER");
    expect(within(table).getByText("RO-CALF-B").closest("tr")?.textContent).toContain("07.01.2024");
    fireEvent.change(screen.getByLabelText("Anul evenimentului"), { target: { value: "2024" } });
    fireEvent.change(screen.getByLabelText("Luna evenimentului"), { target: { value: "01" } });
    expect(within(table).getAllByRole("row")).toHaveLength(3);
    expect(screen.getByRole("status").textContent).toContain("1 din 2");
    fireEvent.click(screen.getAllByRole("button", { name: /Editează fătarea/ })[0]);
    expect(callbacks.onEditBirth).toHaveBeenCalledWith(births[0]);
    fireEvent.click(screen.getByRole("button", { name: /Resetează filtrele/ }));
    expect(within(table).getAllByRole("row")).toHaveLength(4);
  });

  it("filters inseminations by sire association and keeps actions unavailable to viewers", () => {
    setup("inseminations", false);
    const table = screen.getByRole("table");
    fireEvent.change(screen.getByLabelText("Mascul asociat"), { target: { value: "without" } });
    expect(within(table).getAllByRole("row")).toHaveLength(2);
    expect(within(table).getByText("Nespecificat")).toBeTruthy();
    expect(within(table).queryByText("RO-FATHER")).toBeNull();
    expect(within(table).queryByRole("button", { name: /Editează/ })).toBeNull();
  });
});
