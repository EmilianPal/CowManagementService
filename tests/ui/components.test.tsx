import { useState } from "react";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { DateInput } from "../../src/components/DateInput";
import { CowSelect } from "../../src/components/CowSelect";
import { CalfMultiSelect } from "../../src/components/CalfMultiSelect";
import { Modal } from "../../src/components/Modal";
import { MotherDetails } from "../../src/features/herd/MotherDetails";
import type { Cow } from "../../src/features/herd/types";

const cow = (id: number, tag: string, birthDate = "2025-01-01"): Cow => ({
  id, farm_id: 1, ear_tag: tag, sex: "Female", breed: "Metis", category: "Carne",
  birth_date: birthDate, entry_date: birthDate, exit_date: null, birth_id: null,
  birth_count: 0, insemination_count: 0,
});

describe("date and cattle controls", () => {
  it("keeps the prior date when typed input is invalid and accepts a leap day", () => {
    const onChange = vi.fn();
    const { container } = render(<DateInput name="birth_date" defaultValue="2024-01-01" onChange={onChange} />);
    const input = screen.getByPlaceholderText("zz.ll.aaaa") as HTMLInputElement;
    fireEvent.change(input, { target: { value: "31.02.2024" } });
    expect(screen.getByRole("alert").textContent).toContain("dată validă");
    expect((container.querySelector('[name="birth_date"]') as HTMLInputElement).value).toBe("2024-01-01");
    expect(onChange).not.toHaveBeenCalled();
    fireEvent.change(input, { target: { value: "29.02.2024" } });
    expect((container.querySelector('[name="birth_date"]') as HTMLInputElement).value).toBe("2024-02-29");
    expect(onChange).toHaveBeenCalledWith({ target: { value: "2024-02-29" } });
  });

  it("rejects incomplete and post-limit dates without changing the submitted ISO value", () => {
    const onChange = vi.fn();
    const { container } = render(<DateInput name="event_date" defaultValue="2024-02-29" max="2024-03-01" onChange={onChange} />);
    const visible = screen.getByPlaceholderText("zz.ll.aaaa") as HTMLInputElement;
    const hidden = container.querySelector('[name="event_date"]') as HTMLInputElement;
    for (const value of ["29.02", "02.03.2024", "29.02.2023"]) {
      fireEvent.change(visible, { target: { value } });
      expect(visible.getAttribute("aria-invalid")).toBe("true");
      expect(hidden.value).toBe("2024-02-29");
    }
    expect(onChange).not.toHaveBeenCalled();
    fireEvent.change(visible, { target: { value: "01.03.2024" } });
    expect(visible.getAttribute("aria-invalid")).toBe("false");
    expect(hidden.value).toBe("2024-03-01");
  });

  it("updates the submitted date from the calendar control", () => {
    const onChange = vi.fn();
    const { container } = render(<DateInput name="event_date" defaultValue="2024-01-01" onChange={onChange} />);
    fireEvent.change(screen.getByLabelText("Alege data din calendar"), { target: { value: "2024-02-29" } });
    expect((container.querySelector('[name="event_date"]') as HTMLInputElement).value).toBe("2024-02-29");
    expect(screen.getByPlaceholderText("zz.ll.aaaa")).toHaveProperty("value", "29.02.2024");
    expect(onChange).toHaveBeenCalledWith({ target: { value: "2024-02-29" } });
  });

  it("searches any ear-tag fragment and selects with the keyboard", () => {
    const onChange = vi.fn();
    const { container } = render(<CowSelect name="mother" cows={[cow(1, "RO1234567"), cow(2, "RO9999000")]} onChange={onChange} />);
    const input = screen.getByRole("combobox") as HTMLInputElement;
    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: "67" } });
    expect(screen.getAllByRole("option")).toHaveLength(1);
    fireEvent.keyDown(input, { key: "Enter" });
    expect((container.querySelector('[name="mother"]') as HTMLInputElement).value).toBe("1");
    expect(onChange).toHaveBeenLastCalledWith(expect.objectContaining({ ear_tag: "RO1234567" }));
  });

  it("clears a selected cow if the list no longer contains it", async () => {
    const onChange = vi.fn();
    const { container, rerender } = render(<CowSelect name="mother" cows={[cow(1, "RO-ONE"), cow(2, "RO-TWO")]} defaultValue={2} onChange={onChange} />);
    expect((container.querySelector('[name="mother"]') as HTMLInputElement).value).toBe("2");
    rerender(<CowSelect name="mother" cows={[cow(1, "RO-ONE")]} defaultValue={2} onChange={onChange} />);
    await waitFor(() => expect((container.querySelector('[name="mother"]') as HTMLInputElement).value).toBe(""));
    expect(onChange).toHaveBeenCalledWith(null);
  });

  it("toggles only calves born on the selected date", () => {
    function Harness() {
      const [selected, setSelected] = useState<number[]>([2]);
      return <CalfMultiSelect cows={[cow(1, "RO111", "2025-01-01"), cow(2, "RO222", "2025-01-01"), cow(3, "RO333", "2024-01-01")]}
        birthDate="2025-01-01" selectedIds={selected} onChange={setSelected} />;
    }
    render(<Harness />);
    fireEvent.focus(screen.getByRole("combobox"));
    expect(screen.getAllByRole("option").map(option => option.textContent)).toEqual([
      expect.stringContaining("RO222"), expect.stringContaining("RO111"),
    ]);
    expect(screen.queryByText(/RO333/)).toBeNull();
    fireEvent.click(screen.getByRole("option", { name: /RO111/ }));
    expect(screen.getByText("RO111 · RO222")).toBeTruthy();
    fireEvent.click(screen.getByRole("option", { name: /RO222/ }));
    expect(screen.getByText("RO111")).toBeTruthy();
  });

  it("searches calf tags and toggles a selected match with Enter", () => {
    function Harness() {
      const [selected, setSelected] = useState<number[]>([2]);
      return <CalfMultiSelect cows={[cow(1, "RO-1234567"), cow(2, "RO-9990067"), cow(3, "RO-OTHER")]}
        birthDate="2025-01-01" selectedIds={selected} onChange={setSelected} />;
    }
    render(<Harness />);
    const search = screen.getByRole("combobox");
    fireEvent.focus(search);
    fireEvent.change(search, { target: { value: "67" } });
    expect(screen.getAllByRole("option").map(option => option.textContent)).toEqual([
      expect.stringContaining("RO-9990067"), expect.stringContaining("RO-1234567"),
    ]);
    fireEvent.keyDown(search, { key: "Enter" });
    expect(screen.queryByText("RO-9990067")).toBeNull();
  });
});

describe("details and modal", () => {
  it("opens mother's card on click and keeps it open while the pointer is over the card", async () => {
    const { container } = render(<MotherDetails mother={cow(1, "RO-MOTHER")} farmName="Ferma" />);
    const trigger = screen.getByRole("button", { name: /Arată datele mamei/ });
    fireEvent.pointerEnter(trigger);
    expect(trigger.getAttribute("aria-expanded")).toBe("false");
    fireEvent.click(trigger);
    expect(trigger.getAttribute("aria-expanded")).toBe("true");
    const card = container.querySelector(".mother-card")!;
    expect(card.hasAttribute("hidden")).toBe(false);
    fireEvent.pointerLeave(trigger);
    fireEvent.pointerEnter(card);
    await new Promise(resolve => setTimeout(resolve, 180));
    expect(trigger.getAttribute("aria-expanded")).toBe("true");
    fireEvent.pointerLeave(card);
    await waitFor(() => expect(trigger.getAttribute("aria-expanded")).toBe("false"));
  });

  it("allows Escape to close an idle modal but not a busy one", () => {
    const close = vi.fn();
    const { rerender, container } = render(<Modal title="Fișa bovinei" onClose={close}>Detalii</Modal>);
    const dialog = container.querySelector("dialog")!;
    fireEvent(dialog, new Event("cancel", { bubbles: true, cancelable: true }));
    expect(close).toHaveBeenCalledTimes(1);
    rerender(<Modal title="Fișa bovinei" busy onClose={close}>Detalii</Modal>);
    fireEvent(dialog, new Event("cancel", { bubbles: true, cancelable: true }));
    expect(close).toHaveBeenCalledTimes(1);
  });
});
