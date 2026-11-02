import "@testing-library/jest-dom/vitest";
import { vi } from "vitest";

// jsdom has no layout engine. Radix Switch observes its control's size;
// these tests assert semantics and persistence, not measured geometry.
vi.stubGlobal(
  "ResizeObserver",
  class {
    observe() {}
    unobserve() {}
    disconnect() {}
  },
);
