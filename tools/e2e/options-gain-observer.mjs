// Bounded forwarding observer. No engine objects or expected gains.
// Install after the existing public Web Audio observer, before app execution.
export function installOptionsGainObserver() {
  const LIMIT = 8192, NODE_LIMIT = 1024;
  const parameters = new WeakMap(), gainIds = new WeakMap(), sourceIds = new WeakMap();
  const gainNodes = [], sourceNodes = [], events = [];
  let truncated = false, nextSource = 0;
  const record = row => { if (events.length === LIMIT) truncated = true; else events.push(row); };
  const identifyGain = node => {
    if (!gainIds.has(node)) {
      if (gainNodes.length === NODE_LIMIT) { truncated = true; throw new Error('Actual Options gain-node observation capacity exceeded'); }
      gainIds.set(node, gainNodes.length); gainNodes.push(node); parameters.set(node.gain, node);
    }
    return gainIds.get(node);
  };
  const createGain = AudioContext.prototype.createGain;
  AudioContext.prototype.createGain = function (...args) {
    const node = createGain.apply(this, args); identifyGain(node); return node;
  };
  const descriptor = Object.getOwnPropertyDescriptor(AudioParam.prototype, 'value');
  if (!descriptor?.set || !descriptor.get || !descriptor.configurable) throw new Error('Actual AudioParam value descriptor unavailable');
  Object.defineProperty(AudioParam.prototype, 'value', {...descriptor,
    set(value) {
      const node = parameters.get(this), before = node ? descriptor.get.call(this) : null;
      descriptor.set.call(this, value);
      if (node) record({type: 'gain-write', gainNode: identifyGain(node), before,
        requested: value, actual: descriptor.get.call(this), audioTime: node.context.currentTime, contextState: node.context.state,
        sources: sourceNodes.filter(row => row.gain === node).map(row => ({source: row.source, ended: row.ended,
          pitch: row.node.playbackRate.value, looping: row.node.loop}))});
    },
  });
  const destinations = new WeakMap(), connect = AudioNode.prototype.connect;
  AudioNode.prototype.connect = function (...args) {
    const result = connect.apply(this, args);
    if (this instanceof AudioBufferSourceNode && args[0] instanceof GainNode) destinations.set(this, args[0]);
    return result;
  };
  const start = AudioBufferSourceNode.prototype.start;
  AudioBufferSourceNode.prototype.start = function (...args) {
    const result = start.apply(this, args), gain = destinations.get(this);
    if (sourceNodes.length === NODE_LIMIT) { truncated = true; throw new Error('Actual Options source observation capacity exceeded'); }
    const id = nextSource++, row = {source: id, node: this, gain, ended: false};
    sourceIds.set(this, id); sourceNodes.push(row);
    //The unchanged forwarding observer records synchronously inside start.
    //Join actualAPI identities, never an engine/sample-slot identifier.
    const api = window.__audioApi?.events.at(-1);
    if (api?.type !== 'start' || api.frames !== this.buffer?.length || api.sampleRate !== this.buffer?.sampleRate)
      throw new Error('Actual forwarded Source.start identity is missing');
    record({type: 'source-start', source: id, gainNode: gain ? identifyGain(gain) : null,
      apiNode: api.node, apiGainNode: api.gainNode, reachesDestination: api.reachesDestination,
      pitch: this.playbackRate.value, looping: this.loop,
      actualGain: gain?.gain.value ?? null, frames: this.buffer?.length ?? null,
      channels: this.buffer?.numberOfChannels ?? null, sampleRate: this.buffer?.sampleRate ?? null,
      when: args[0] ?? 0, audioTime: this.context.currentTime, contextState: this.context.state});
    this.addEventListener('ended', () => {row.ended = true; record({type: 'source-ended', source: id,
      gainNode: gain ? identifyGain(gain) : null, audioTime: this.context.currentTime});});
    return result;
  };
  const stop = AudioBufferSourceNode.prototype.stop;
  AudioBufferSourceNode.prototype.stop = function (...args) {
    const result = stop.apply(this, args);
    record({type: 'source-stop', source: sourceIds.get(this) ?? null, when: args[0] ?? 0,
      audioTime: this.context.currentTime, contextState: this.context.state});
    return result;
  };
  window.__optionsGainObservation = {
    snapshot: () => ({schema: 1, events: structuredClone(events), truncated,
      nodes: gainNodes.map((node, gainNode) => ({gainNode, actual: node.gain.value, audioTime: node.context.currentTime, contextState: node.context.state})),
      sources: sourceNodes.map(({source, node, gain, ended}) => ({source, gainNode: gain ? identifyGain(gain) : null,
        ended, actualGain: gain?.gain.value ?? null, frames: node.buffer?.length ?? null})),
      scope: 'Actual forwarded GainNode AudioParam.value writes/readbacks and Source starts/ended; no engine-state-derived gains, PCM or audible parity'}),
  };
}
