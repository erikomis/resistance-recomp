// Controls: the controller test. The port reads the controller through SDL2
// (DualSense, DualShock 4/3 or Xbox); this page shows what the system sees
// through the Gamepad API, button by button, so the player can check the pad
// is connected and every button registers before playing. There is nothing
// to remap: the toolkit's keyboard and mouse input is Windows-only so far,
// and the controller layout is the PS3's own.
import { t } from "./i18n";

const $ = <T extends HTMLElement = HTMLElement>(id: string) => document.getElementById(id) as T;

/** The standard Gamepad mapping, by index, as PS3 buttons. */
const BUTTONS: { i: number; ps: string; xbox: string }[] = [
  { i: 0, ps: "✕ Cross", xbox: "A" },
  { i: 1, ps: "○ Circle", xbox: "B" },
  { i: 2, ps: "□ Square", xbox: "X" },
  { i: 3, ps: "△ Triangle", xbox: "Y" },
  { i: 4, ps: "L1", xbox: "LB" },
  { i: 5, ps: "R1", xbox: "RB" },
  { i: 6, ps: "L2", xbox: "LT" },
  { i: 7, ps: "R2", xbox: "RT" },
  { i: 8, ps: "Select", xbox: "View" },
  { i: 9, ps: "Start", xbox: "Menu" },
  { i: 10, ps: "L3", xbox: "LS" },
  { i: 11, ps: "R3", xbox: "RS" },
  { i: 12, ps: "↑", xbox: "↑" },
  { i: 13, ps: "↓", xbox: "↓" },
  { i: 14, ps: "←", xbox: "←" },
  { i: 15, ps: "→", xbox: "→" },
  { i: 16, ps: "PS", xbox: "Xbox" },
];

let active = false;
let raf = 0;
const cells: HTMLElement[] = [];

function draw() {
  raf = 0;
  if (!active) return;
  const pads = ("getGamepads" in navigator ? Array.from(navigator.getGamepads()) : []).filter((p): p is Gamepad => !!p);
  const pad = pads[0];
  $("pad-name").textContent = pad ? t("controls.connected", { name: pad.id }) : "getGamepads" in navigator ? t("controls.noPad") : t("controls.noApi");
  for (const b of BUTTONS) {
    const on = !!pad?.buttons[b.i] && (pad.buttons[b.i].pressed || pad.buttons[b.i].value > 0.4);
    cells[b.i]?.classList.toggle("held", on);
  }
  const stick = (id: string, x = 0, y = 0) => {
    const dot = $(id).querySelector("i") as HTMLElement;
    dot.style.left = `${50 + x * 42}%`;
    dot.style.top = `${50 + y * 42}%`;
  };
  stick("stick-l", pad?.axes[0], pad?.axes[1]);
  stick("stick-r", pad?.axes[2], pad?.axes[3]);
  raf = requestAnimationFrame(draw);
}

export function renderControls() {
  const grid = $("pad-grid");
  grid.replaceChildren();
  cells.length = 0;
  for (const b of BUTTONS) {
    const d = document.createElement("div");
    d.className = "pad-key";
    const n = document.createElement("b");
    n.textContent = b.ps;
    const s = document.createElement("span");
    s.className = "small muted";
    s.textContent = `Xbox: ${b.xbox}`;
    d.append(n, s);
    grid.append(d);
    cells[b.i] = d;
  }
}

export function showControls(on: boolean) {
  active = on;
  if (on && !raf) raf = requestAnimationFrame(draw);
}

export function initControls() {
  renderControls();
  window.addEventListener("gamepadconnected", () => active && !raf && (raf = requestAnimationFrame(draw)));
}
