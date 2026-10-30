import { afterEach } from "vitest";
import { cleanup } from "@testing-library/react";

HTMLDialogElement.prototype.showModal = function () { this.setAttribute("open", ""); };
HTMLDialogElement.prototype.close = function () { this.removeAttribute("open"); };
HTMLElement.prototype.showPopover = function () { this.removeAttribute("hidden"); };
HTMLElement.prototype.hidePopover = function () { this.setAttribute("hidden", ""); };

afterEach(() => {
  cleanup();
  document.querySelectorAll("[popover]").forEach(node => node.remove());
});
