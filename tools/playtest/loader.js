// Inline loader for the playtest bundle: fetch the asset packs, serve their
// original bytes through fetch() and <img>.src, then start the game module.
//
// Two counted browser-compatibility shims (window.__playtestShims) cover
// decoder behavior observed in some Chromium builds; the self-playtest reports
// their counts:
// - HTMLImageElement.decode() rejects with EncodingError when a few hundred
//   decodes are in flight; fall back to the element's load event.
// - decodeAudioData() ignores Vorbis end trimming; trim the buffer to the
//   final Ogg granule position, as the Vorbis spec requires.
const shims = window.__playtestShims = { imageDecodeFallbacks: 0, vorbisEndTrims: 0 };
{
  const decode = HTMLImageElement.prototype.decode;
  HTMLImageElement.prototype.decode = function () {
    return decode.call(this).catch((error) => new Promise((resolve, reject) => {
      const settle = () => (this.naturalWidth > 0 ? (shims.imageDecodeFallbacks++, resolve()) : reject(error));
      if (this.complete) settle();
      else { this.addEventListener('load', settle, { once: true }); this.addEventListener('error', () => reject(error), { once: true }); }
    }));
  };
  const vorbisFrames = (buffer) => {
    const bytes = new Uint8Array(buffer);
    if (bytes.length < 64 || String.fromCharCode(...bytes.subarray(0, 4)) !== 'OggS') return null;
    const packet = 27 + bytes[26];
    if (bytes[packet] !== 1 || String.fromCharCode(...bytes.subarray(packet + 1, packet + 7)) !== 'vorbis') return null;
    const rate = new DataView(buffer).getUint32(packet + 12, true);
    for (let at = bytes.length - 27; at >= 0; at--) {
      if (bytes[at] === 0x4f && bytes[at + 1] === 0x67 && bytes[at + 2] === 0x67 && bytes[at + 3] === 0x53 && bytes[at + 5] & 4)
        return { granule: Number(new DataView(buffer).getBigInt64(at + 6, true)), rate };
    }
    return null;
  };
  const decodeAudio = BaseAudioContext.prototype.decodeAudioData;
  BaseAudioContext.prototype.decodeAudioData = function (buffer, ...callbacks) {
    const end = buffer instanceof ArrayBuffer ? vorbisFrames(buffer) : null;
    return decodeAudio.call(this, buffer).then((decoded) => {
      const frames = end && Math.round(end.granule * decoded.sampleRate / end.rate);
      if (!frames || decoded.length <= frames) return decoded;
      const trimmed = new AudioBuffer({ length: frames, numberOfChannels: decoded.numberOfChannels, sampleRate: decoded.sampleRate });
      for (let channel = 0; channel < decoded.numberOfChannels; channel++)
        trimmed.copyToChannel(decoded.getChannelData(channel).subarray(0, frames), channel);
      shims.vorbisEndTrims++;
      return trimmed;
    }).then((decoded) => { callbacks[0]?.(decoded); return decoded; }, (error) => { callbacks[1]?.(error); throw error; });
  };
}
(async () => {
  const config = window.__PLAYTEST__;
  const status = document.getElementById('status');
  const start = document.getElementById('playtest-start');
  const canvas = document.getElementById('game');
  const total = config.packs.length;
  let loaded = 0;
  status.textContent = `Loading assets 0/${total}…`;
  const packs = await Promise.all(config.packs.map(async (url) => {
    const response = await fetch(url);
    if (!response.ok) throw new Error(`Asset pack ${url} failed: ${response.status}`);
    const bytes = await response.arrayBuffer();
    status.textContent = `Loading assets ${++loaded}/${total}…`;
    return bytes;
  }));
  // Copy each asset into its own Blob: Chromium fails some concurrent image
  // decodes of slices taken from one large Blob.
  const blobs = new Map();
  for (const [name, [pack, offset, length, type]] of Object.entries(config.index))
    blobs.set(name, new Blob([new Uint8Array(packs[pack], offset, length)], { type }));
  const objectUrls = new Map();
  const packedName = (input) => {
    try {
      const href = typeof input === 'string' ? input : input instanceof URL ? input.href : input?.url;
      const url = new URL(href, location.href);
      if (url.protocol === 'blob:' || url.protocol === 'data:') return null;
      const path = decodeURIComponent(url.pathname);
      const name = path.slice(path.lastIndexOf('/') + 1);
      return blobs.has(name) ? name : null;
    } catch { return null; }
  };
  const nativeFetch = window.fetch.bind(window);
  window.fetch = (input, init) => {
    const name = packedName(input);
    if (!name) return nativeFetch(input, init);
    const blob = blobs.get(name);
    return Promise.resolve(new Response(blob, { status: 200, headers: { 'Content-Type': blob.type, 'Content-Length': String(blob.size) } }));
  };
  const source = Object.getOwnPropertyDescriptor(HTMLImageElement.prototype, 'src');
  Object.defineProperty(HTMLImageElement.prototype, 'src', {
    configurable: true, enumerable: source.enumerable, get: source.get,
    set(value) {
      const name = packedName(String(value));
      if (name && !objectUrls.has(name)) objectUrls.set(name, URL.createObjectURL(blobs.get(name)));
      source.set.call(this, name ? objectUrls.get(name) : value);
    },
  });
  status.textContent = 'Starting…';
  const ready = new MutationObserver(() => {
    if (document.body.dataset.ready !== 'true') return;
    ready.disconnect();
    if (!document.hasFocus()) start.hidden = false;
  });
  ready.observe(document.body, { attributes: true, attributeFilter: ['data-ready'] });
  start.addEventListener('click', () => { start.hidden = true; canvas.focus(); });
  window.addEventListener('focus', () => { start.hidden = true; });
  const script = document.createElement('script');
  script.type = 'module';
  script.src = config.entry;
  document.body.append(script);
})().catch((error) => {
  const status = document.getElementById('status');
  status.hidden = false;
  status.textContent = String(error);
  console.error(error);
});
