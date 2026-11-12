// The Controls page's "Modern layout": the controller bindings of a modern
// shooter, laid over the game's own default controls (Options > Controls >
// Save/Load, "Default Controls": R1 primary fire, L1 alt. fire, L2 crouch,
// R2 weapon select, ○ use grenade, ✕ jump, □ reload, △ action/melee, R3
// toggle zoom, L3 voice chat). PS3 control -> the physical inputs (SDL
// names) that press it. The game has no sprint and no hold-to-zoom: its zoom
// is a toggle, so L2 toggles it. No imports, so it can be checked on its own.

export type PadBindings = Record<string, string[]>;

export const MODERN: PadBindings = {
  r1: ["righttrigger"], // R2 fires
  r3: ["lefttrigger"], // L2 zooms (press again to zoom out)
  l1: ["leftshoulder"], // L1 alt. fire, as before
  r2: ["rightshoulder"], // R1 switches weapons
  l2: ["leftstick"], // L3 crouches
  triangle: ["y", "rightstick"], // R3 melee / action; △ still does it
  l3: [], // voice chat (online only) leaves the stick click
};

/** `controller` with `layout` applied: the layout's controls get its inputs,
 * and those inputs leave every other control, so no input presses two
 * controls. Applying it twice changes nothing. */
export function applyLayout(controller: PadBindings, layout: PadBindings): PadBindings {
  const used = new Set(Object.values(layout).flat());
  const out: PadBindings = {};
  for (const [k, v] of Object.entries(controller)) out[k] = v.filter((n) => !used.has(n));
  for (const [k, v] of Object.entries(layout)) out[k] = [...v];
  return out;
}
