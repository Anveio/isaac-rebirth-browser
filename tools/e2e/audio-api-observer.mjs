// Independent forwarding observation of actual public Web Audio calls.
export function installBrowserAudioObserver() {
  const nodes = new WeakMap(), gains = new WeakMap(), connections = new WeakMap(), decodedInputs = new WeakMap(), decodedBuffers = [], contexts = new Set(); let next = 0;
  const trace = { events: [], pending: [], truncated: false };
  trace.contextTimes = () => [...contexts].map(context => ({ state: context.state, sampleRate: context.sampleRate, currentTime: context.currentTime }));
  const identify = node => { if (!nodes.has(node)) nodes.set(node, ++next); return nodes.get(node); };
  const record = event => { if (trace.events.length >= 8192) trace.truncated = true; else trace.events.push(event); };
  const digest = async bytes => Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256', bytes)), byte => byte.toString(16).padStart(2, '0')).join('');
  const decode = AudioContext.prototype.decodeAudioData;
  AudioContext.prototype.decodeAudioData = function (...args) {
    contexts.add(this);
    const input = args[0].slice(0), identity = digest(input).then(sha256 => ({ sha256, bytes: input.byteLength }));
    const result = decode.apply(this, args);
    trace.pending.push(result.then(buffer => { decodedInputs.set(buffer, identity); decodedBuffers.push(buffer); }));
    return result;
  };
  // A producer may trim a decoder's output to the stream's end position.
  // Such a buffer names the decode input whose leading frames it exactly
  // reproduces after PCM16 quantization; nothing else is attributed.
  const quantized = value => Math.max(-32768, Math.min(32767, Math.round(value * 32768)));
  const trimmedInput = buffer => {
    for (const decoded of decodedBuffers) {
      if (decoded === buffer || decoded.length <= buffer.length || decoded.sampleRate !== buffer.sampleRate ||
          decoded.numberOfChannels !== buffer.numberOfChannels) continue;
      let equal = true;
      for (let channel = 0; equal && channel < buffer.numberOfChannels; channel++) {
        const source = decoded.getChannelData(channel), samples = buffer.getChannelData(channel);
        for (let frame = 0; frame < buffer.length; frame++)
          if (quantized(source[frame]) !== quantized(samples[frame])) { equal = false; break; }
      }
      if (equal) return decodedInputs.get(decoded).then(input => ({ ...input, trimmedFromFrames: decoded.length }));
    }
    return undefined;
  };
  const loop = Object.getOwnPropertyDescriptor(AudioBufferSourceNode.prototype, 'loop');
  Object.defineProperty(AudioBufferSourceNode.prototype, 'loop', { ...loop,
    set(value) { loop.set.call(this, value); record({ type: 'loop', node: identify(this), looping: loop.get.call(this), audioTime: this.context.currentTime }); },
  });
  const connect = AudioNode.prototype.connect;
  AudioNode.prototype.connect = function (...args) {
    const result = connect.apply(this, args);
    if (args[0] instanceof AudioNode) {
      if (!connections.has(this)) connections.set(this, new Set());
      connections.get(this).add(args[0]);
    }
    if (this instanceof AudioBufferSourceNode && args[0] instanceof GainNode) gains.set(this, args[0]);
    return result;
  };
  const disconnect = AudioNode.prototype.disconnect;
  AudioNode.prototype.disconnect = function (...args) {
    const result = disconnect.apply(this, args);
    if (args[0] instanceof AudioNode) connections.get(this)?.delete(args[0]);
    else connections.delete(this);
    return result;
  };
  const reachesDestination = source => {
    const pending = [source], seen = new Set();
    while (pending.length) {
      const node = pending.pop();
      if (node === source.context.destination) return true;
      if (seen.has(node)) continue;
      seen.add(node); pending.push(...(connections.get(node) || []));
    }
    return false;
  };
  const start = AudioBufferSourceNode.prototype.start, stop = AudioBufferSourceNode.prototype.stop;
  AudioBufferSourceNode.prototype.start = function (...args) {
    contexts.add(this.context);
    const result = start.apply(this, args), buffer = this.buffer;
    if (!buffer) throw new Error('Actual Web Audio start has no sample buffer');
    const event = { type: 'start', node: identify(this), frames: buffer.length, channels: buffer.numberOfChannels, sampleRate: buffer.sampleRate,
      pitch: this.playbackRate.value, gain: gains.get(this)?.gain.value, looping: this.loop,
      gainNode: gains.has(this) ? identify(gains.get(this)) : null,
      reachesDestination: reachesDestination(this), pcmSha256: null,
      when: args[0] ?? 0, audioTime: this.context.currentTime,
      pcmPrefix: buffer.numberOfChannels === 2 ? Array.from({ length: Math.min(buffer.length, 2048) * 2 }, (_, index) =>
        Math.max(-32768, Math.min(32767, Math.round(buffer.getChannelData(index % 2)[Math.floor(index / 2)] * 32768)))) : null };
    record(event);
    this.addEventListener('ended', () => record({ type: 'ended', node: identify(this), audioTime: this.context.currentTime }));
    // Defer complete buffer reads until the producer's synchronous start batch
    // has finished. Observation must not delay the next scheduled channel call.
    trace.pending.push(Promise.resolve().then(async () => {
      const bytes = new ArrayBuffer(buffer.length * buffer.numberOfChannels * 2), view = new DataView(bytes);
      for (let channel = 0; channel < buffer.numberOfChannels; channel++) {
        const samples = buffer.getChannelData(channel);
        for (let frame = 0; frame < buffer.length; frame++) view.setInt16((frame * buffer.numberOfChannels + channel) * 2,
          Math.max(-32768, Math.min(32767, Math.round(samples[frame] * 32768))), true);
      }
      event.pcmSha256 = await digest(bytes);
      if (buffer.numberOfChannels === 2) {
        event.encodedInput = await (decodedInputs.get(buffer) ?? trimmedInput(buffer));
        const data = new Uint8Array(bytes), pieces = [];
        for (let i = 0; i < data.length; i += 32768) pieces.push(String.fromCharCode(...data.subarray(i, i + 32768)));
        event.pcmBase64 = btoa(pieces.join(''));
      }
    }));
    return result;
  };
  AudioBufferSourceNode.prototype.stop = function (...args) { const result = stop.apply(this, args); record({ type: 'stop', node: identify(this) }); return result; };
  window.__audioApi = trace;
}
