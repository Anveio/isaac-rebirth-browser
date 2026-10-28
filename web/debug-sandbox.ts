// Debug sandbox floor (port only). Built only by `npm run dev:sandbox`, which
// sets __DEBUG_SANDBOX__ and the engine's `debug-sandbox` cargo feature; the
// release bundle never contains this module (tools/e2e/policy.mjs checks).
//
// Base Rebirth has no item room. This panel lays out every imported
// collectible on pedestals in the current room, one page at a time and
// filtered by pool, spawns a chosen enemy, and prints the command that records
// the held set (plus a chosen trinket) as a live original-vs-port case. Loose
// trinket pickups are not ported, so a trinket joins only the recorded case,
// where the sandbox Challenge grants it.
import catalogue from "./generated/item-assets/catalogue.json";
import type { DebugSandbox, State } from "./engine";

type Entry = { id: number; name: string; pools?: string[] };

export const MARKER = "debug-sandbox-floor";

export function mountDebugSandbox(sandbox: DebugSandbox, state: () => State, refresh: () => void) {
  const collectibles: Entry[] = catalogue.collectibles;
  const trinkets: Entry[] = catalogue.trinkets;
  const pools = [...new Set(collectibles.flatMap(item => item.pools ?? []))].sort();
  let lastEnemy = "";

  const panel = document.createElement("aside");
  panel.dataset.debugSandbox = MARKER;
  panel.style.cssText = "position:fixed;top:8px;right:8px;z-index:10;width:260px;max-height:calc(100vh - 16px);overflow:auto;"
    + "background:rgba(20,16,12,.92);color:#eee;font:12px/1.4 system-ui,sans-serif;padding:8px;border-radius:6px";
  panel.innerHTML = `
    <strong>Debug sandbox floor</strong>
    <label style="display:block">Pool <select data-pool><option value="">All</option>${pools.map(pool => `<option>${pool}</option>`).join("")}</select></label>
    <div style="display:flex;gap:4px;align-items:center;margin-top:4px">
      <button data-prev>&lt;</button><span data-page style="flex:1;text-align:center"></span><button data-next>&gt;</button>
    </div>
    <button data-layout style="width:100%;margin-top:4px">Lay out this page</button>
    <ol data-list style="margin:6px 0;padding-left:28px"></ol>
    <label style="display:block">Enemy (type.variant) <input data-enemy size="8" placeholder="10.0"></label>
    <button data-spawn style="width:100%;margin-top:4px">Spawn enemy</button>
    <label style="display:block;margin-top:8px">Trinket for the case <select data-trinket style="width:100%"><option value="0">None</option>${trinkets.map(trinket => `<option value="${trinket.id}">${trinket.id} ${trinket.name}</option>`).join("")}</select></label>
    <button data-record style="width:100%;margin-top:8px">Record case</button>
    <code data-command style="display:block;white-space:pre-wrap;word-break:break-all;margin-top:4px"></code>
    <div data-error style="color:#f88"></div>`;
  document.body.append(panel);
  const $ = <T extends HTMLElement>(name: string) => panel.querySelector<T>(`[data-${name}]`)!;
  const pool = $<HTMLSelectElement>("pool"), list = $<HTMLOListElement>("list");
  const error = $("error");
  let page = 0;

  const entries = () => collectibles.filter(item => !pool.value || item.pools?.includes(pool.value));
  const pageSize = () => Math.max(1, sandbox.slots());
  const pageCount = () => Math.max(1, Math.ceil(entries().length / pageSize()));
  const current = () => entries().slice(page * pageSize(), (page + 1) * pageSize());
  const show = () => {
    page = Math.min(page, pageCount() - 1);
    $("page").textContent = `${page + 1} / ${pageCount()}`;
    list.start = page * pageSize() + 1;
    list.replaceChildren(...current().map(item => Object.assign(document.createElement("li"), { textContent: `${item.id} ${item.name}` })));
  };
  const act = (action: () => void) => {
    try { action(); error.textContent = ""; } catch (failure) { error.textContent = String(failure instanceof Error ? failure.message : failure); }
    refresh();
  };
  pool.onchange = () => { page = 0; show(); };
  $("prev").onclick = () => { page = (page + pageCount() - 1) % pageCount(); show(); };
  $("next").onclick = () => { page = (page + 1) % pageCount(); show(); };
  $("layout").onclick = () => act(() => {
    sandbox.clear();
    current().forEach((item, slot) => sandbox.place(item.id, slot));
  });
  $("spawn").onclick = () => act(() => {
    const [type, variant = "0"] = $<HTMLInputElement>("enemy").value.trim().split(".");
    sandbox.enemy(Number(type), Number(variant));
    lastEnemy = `${Number(type)}.${Number(variant)}`;
  });
  $("record").onclick = () => {
    const items = state().playerItems as { ownedCounts?: number[] } | null | undefined;
    const owned = (items?.ownedCounts ?? []).flatMap((count, id) => Array.from({ length: Math.max(0, count) }, () => id));
    const trinket = Number($<HTMLSelectElement>("trinket").value);
    $("command").textContent = owned.length === 0 ? "Pick up at least one collectible first."
      : `npm run sandbox:case -- --items ${owned.join(",")}${trinket ? ` --trinket ${trinket}` : ""}${lastEnemy ? ` --enemy ${lastEnemy}` : ""}`;
  };
  // Keys typed into the panel stay out of the game.
  panel.addEventListener("keydown", event => event.stopPropagation());
  show();
}
