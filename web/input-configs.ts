import type { InputConfigs, State } from './engine';

const KEY = 'isaac-rebirth.inputconfigs.dat.v1';
const LIMIT = 256;
//33 records × bounded Name escaping/all48 signed-i32 attributes fit128KiB.
const XML_BYTE_LIMIT = 128*1024;
const DEFAULT_PAIRS: readonly (readonly [number, number])[] = [
  [0,65],[1,68],[2,87],[3,83],[4,263],[5,262],[6,265],[7,264],
  [8,69],[9,32],[10,81],[11,341],[12,80],[13,258],[14,32],[14,257],
  [15,256],[16,82],[17,70],[18,77],[19,257],[20,263],[21,262],[22,265],[23,264],
];
type IOEvent = {operation: 'load' | 'save'; status: 'pass' | 'absent' | 'fail'; sequence?: number;
  configs?: InputConfigs; text?: string; error?: string};

export function validateInputConfigs(value: unknown): InputConfigs {
  if (!value || typeof value !== 'object' || Array.isArray(value) || Object.keys(value).join(',') !== 'devices')
    throw new Error('Input configurations require exactly the retained devices');
  const devices = (value as InputConfigs).devices;
  if (!Array.isArray(devices) || devices.length > 33) throw new Error('Input configuration device count is outside the admitted format');
  if (devices.filter(device => device?.name !== 'Keyboard').length > 32) throw new Error('Input configuration exceeds32 retained non-Keyboard devices');
  const names = new Set<string>();
  for (const device of devices) {
    if (!device || typeof device !== 'object' || Object.keys(device).sort().join(',') !== 'bindings,name' ||
        typeof device.name !== 'string' || !device.name || new TextEncoder().encode(device.name).length > 256 ||
        /[\u0000-\u001f\u007f-\u009f]/.test(device.name) || names.has(device.name))
      throw new Error('Input configuration has invalid or duplicate device names');
    names.add(device.name);
    if (!Array.isArray(device.bindings) || device.bindings.length !== 24 || device.bindings.some(pair =>
      !Array.isArray(pair) || pair.length !== 2 || pair.some(key => !Number.isInteger(key) || key < -2147483648 || key > 2147483647 ||
        (device.name === 'Keyboard' && (key < -1 || key > 348)))))
      throw new Error('Input configuration has invalid action24/column2 signed bindings');
  }
  return structuredClone(value as InputConfigs);
}

function decodeName(text: string): string {
  if (/<|[\u0000-\u001f]/.test(text) || /&(?!(?:amp|lt|gt|quot|apos|#\d+|#x[\da-fA-F]+);)/.test(text))
    throw new Error('Input device Name contains unsupported XML text');
  return text.replace(/&(amp|lt|gt|quot|apos|#\d+|#x[\da-fA-F]+);/g, (_all, entity: string) => {
    const names: Record<string, string> = {amp:'&',lt:'<',gt:'>',quot:'"',apos:"'"};
    if (entity in names) return names[entity]!;
    const code = entity.startsWith('#x') ? parseInt(entity.slice(2),16) : Number(entity.slice(1));
    if (!Number.isInteger(code) || code <= 0 || code > 0x10ffff || (code >= 0xd800 && code <= 0xdfff))
      throw new Error('Input device Name contains invalid XML character references');
    return String.fromCodePoint(code);
  });
}

/** TinyXML's native decimal attributes such as0-0 are outside the browser
 * DOM XML name grammar. Parse this bounded native save language directly;
 * DOMParser would reject a real native file before any setters could apply.
 * Complete48-column records are admitted. Corrupt/incomplete native lifetime,
 * duplicate names, arbitrary XML extensions and hotplug remain unqualified. */
export function parseInputConfigs(text: string): InputConfigs {
  if (new TextEncoder().encode(text).length > XML_BYTE_LIMIT) throw new Error('Input configuration exceeds128KiB');
  let body = text.replace(/^\uFEFF/, '').trim().replace(/^<\?xml\s+[^?]*\?>\s*/, '');
  const root = /^<KeyConfigs\s*>([\s\S]*)<\/KeyConfigs\s*>$/.exec(body);
  if (!root) throw new Error('Input configuration requires the native KeyConfigs root');
  body = root[1]!.trim();
  const configs: InputConfigs = {devices: []};
  while (body) {
    const child = /^<Controller\b((?:[^"'<>]|"[^"]*"|'[^']*')*)(?:\/>|>\s*<\/Controller\s*>)/.exec(body);
    if (!child) throw new Error('Input configuration contains an unsupported Controller record');
    const attributes = new Map<string,string>();
    let remaining = child[1]!;
    while (remaining.trim()) {
      const attribute = /^\s+(Name|\d+-[01])\s*=\s*(?:"([^"]*)"|'([^']*)')/.exec(remaining);
      if (!attribute || attributes.has(attribute[1]!)) throw new Error('Input configuration has invalid or duplicate attributes');
      attributes.set(attribute[1]!, attribute[2] ?? attribute[3]!); remaining = remaining.slice(attribute[0].length);
    }
    const nameText = attributes.get('Name');
    if (nameText === undefined) throw new Error('Input configuration device is missing Name');
    const name = decodeName(nameText);
    const bindings = Array.from({length:24}, () => [-1,-1]);
    //43BC70's Name=Keyboard path presets BOTH columns for EVERY pair. The
    //duplicate action14 table entry therefore writes Return to both columns;
    //all explicit attributes then apply through the actual startup setter.
    if (name === 'Keyboard') for (const [action,key] of DEFAULT_PAIRS) bindings[action] = [key,key];
    for (let action=0; action<24; action++) for (let column=0; column<2; column++) {
      const value = attributes.get(`${action}-${column}`);
      if (value === undefined || !/^-?\d+$/.test(value)) throw new Error('Input configuration requires all48 signed decimal attributes');
      bindings[action]![column] = Number(value);
    }
    if (attributes.size !== 49) throw new Error('Input configuration has unsupported attributes');
    configs.devices.push({name,bindings});
    if (configs.devices.length > 33) throw new Error('Input configuration exceeds32 retained non-Keyboard devices plus Keyboard');
    body = body.slice(child[0].length).trim();
  }
  return validateInputConfigs(configs);
}

export function serializeInputConfigs(input: InputConfigs): string {
  const configs = validateInputConfigs(input);
  const escape = (value: string) => value.replace(/[&<>"']/g, character => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&apos;'}[character]!));
  const text = '<?xml version="1.0"?>\n<KeyConfigs>\n' + configs.devices.map(device => {
    const attributes = device.bindings.flatMap((pair,action) => pair.map((key,column) => `${action}-${column}="${key}"`));
    return `  <Controller Name="${escape(device.name)}" ${attributes.join(' ')} />`;
  }).join('\n') + '\n</KeyConfigs>\n';
  if (new TextEncoder().encode(text).length > XML_BYTE_LIMIT) throw new Error('Serialized input configuration exceeds128KiB');
  return text;
}

/**Actual origin-local storage IO, separate from the Options preference key.
 * XML transport equivalence never claims a browser owns the native file. */
export class InputConfigIO {
  private storage: Storage | null = null;
  private lastSave = -1;
  private events: IOEvent[] = [];
  private truncated = false;
  constructor() { try { this.storage = window.localStorage; } catch (error) { this.failure('load', error); } }
  load(): InputConfigs | null {
    try {
      if (!this.storage) throw new Error('Input configuration storage is unavailable');
      const text = this.storage.getItem(KEY);
      if (text === null) { this.record({operation:'load', status:'absent'}); return null; }
      const configs = parseInputConfigs(text);
      this.record({operation:'load', status:'pass', configs, text});
      return structuredClone(configs);
    } catch (error) { this.failure('load',error); return null; }
  }
  consume(requests: State['controlsMenu']['saveRequests']): boolean {
    let success = true;
    for (const request of requests) {
      if (!Number.isSafeInteger(request.sequence) || request.sequence < 0) { success=false; this.failure('save','Invalid input save sequence'); continue; }
      if (request.sequence <= this.lastSave) continue;
      this.lastSave = request.sequence;
      try {
        if (!this.storage) throw new Error('Input configuration storage is unavailable');
        const configs = validateInputConfigs(request.configs);
        const text = serializeInputConfigs(configs);
        this.storage.setItem(KEY,text);
        const readback = this.storage.getItem(KEY);
        if (readback !== text || JSON.stringify(parseInputConfigs(readback)) !== JSON.stringify(configs))
          throw new Error('Input configuration readback differs from its actual serialized bindings');
        this.record({operation:'save',status:'pass',sequence:request.sequence,configs,text});
      } catch (error) { success=false; this.failure('save',error,request.sequence); }
    }
    return success;
  }
  resetDispatch(): void { this.lastSave=-1; }
  snapshot() { return {key:KEY, storageAvailable:this.storage !== null, events:structuredClone(this.events), truncated:this.truncated,
    scope:'Actual origin-local inputconfigs.dat-language IO; native filesystem/startup/hotplug/incomplete XML parity is unqualified'}; }
  private record(event: IOEvent): void { if (this.events.length === LIMIT) this.truncated=true; else this.events.push(event); }
  private failure(operation: IOEvent['operation'], error: unknown, sequence?: number): void {
    this.record({operation,status:'fail',...(sequence === undefined ? {} : {sequence}),error:String(error)});
    console.error(`Input configuration ${operation} failed`,error);
  }
}
