import catalogue from './audio-data.json';
import type { Ambience, SoundEvent } from './engine';
import { MusicRuntime } from './music';

const urls = import.meta.glob('../references/isaac/rebirth/resources/sfx/**/*.wav', {
  eager: true, query: '?url', import: 'default',
}) as Record<string, string>;
const RESOURCE_PREFIX = '../references/isaac/rebirth/resources/';
const TRACE_LIMIT = 8192;
type OutputEvent = SoundEvent & { path: string; voice: number | null; output: 'started' | 'waiting-for-unlock' | 'pool-full'; pcmSha256: string };
interface Sample { buffer: AudioBuffer; pcmSha256: string; }
interface Voice { source: AudioBufferSourceNode; gain: GainNode; path: string; endsAt: number; }

async function sha256(bytes: ArrayBuffer): Promise<string> {
  return Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256', bytes)), byte => byte.toString(16).padStart(2, '0')).join('');
}

// The pinned base samples are PCM16 WAV. Preserve their actual sample frames;
// browser compressed-audio decoders and output device rates do not select SFX.
async function loadSample(context: AudioContext, path: string): Promise<Sample> {
  const metadata = catalogue.assets[path as keyof typeof catalogue.assets];
  if (!metadata || !('sampleBytes' in metadata)) throw new Error(`Missing authored WAV metadata: ${path}`);
  const url = urls[RESOURCE_PREFIX + path];
  if (!url) throw new Error(`Missing authored SFX URL: ${path}`);
  const response = await fetch(url);
  if (!response.ok) throw new Error(`SFX load failed: ${path} (${response.status})`);
  const bytes = await response.arrayBuffer();
  if (bytes.byteLength !== metadata.bytes || await sha256(bytes) !== metadata.sha256)
    throw new Error(`SFX bytes differ from the pinned original: ${path}`);
  const view = new DataView(bytes);
  const tag = (at: number) => String.fromCharCode(...new Uint8Array(bytes, at, 4));
  if (bytes.byteLength < 12 || tag(0) !== 'RIFF' || tag(8) !== 'WAVE') throw new Error(`Invalid original WAV: ${path}`);
  let dataOffset = -1, dataBytes = 0, formatFound = false;
  for (let at = 12; at + 8 <= bytes.byteLength;) {
    const length = view.getUint32(at + 4, true), start = at + 8;
    if (start + length > bytes.byteLength) throw new Error(`Truncated original WAV: ${path}`);
    if (tag(at) === 'fmt ') {
      if (length < 16 || view.getUint16(start, true) !== 1 || view.getUint16(start + 2, true) !== metadata.channels ||
          view.getUint32(start + 4, true) !== metadata.sampleRate || view.getUint16(start + 14, true) !== 16)
        throw new Error(`Unsupported authored WAV format: ${path}`);
      formatFound = true;
    } else if (tag(at) === 'data') { dataOffset = start; dataBytes = length; }
    at = start + length + (length & 1);
  }
  if (!formatFound || dataOffset < 0 || metadata.sampleBytes !== 2 || dataBytes !== metadata.frames * metadata.channels * 2)
    throw new Error(`Original WAV frame metadata differs: ${path}`);
  const buffer = context.createBuffer(metadata.channels, metadata.frames, metadata.sampleRate);
  for (let channel = 0; channel < metadata.channels; channel++) {
    const output = buffer.getChannelData(channel);
    for (let frame = 0; frame < metadata.frames; frame++)
      output[frame] = view.getInt16(dataOffset + (frame * metadata.channels + channel) * 2, true) / 32768;
  }
  return { buffer, pcmSha256: await sha256(bytes.slice(dataOffset, dataOffset + dataBytes)) };
}

export class AudioRuntime {
  private readonly voices: (Voice | null)[] = Array.from({ length: 16 }, () => null);
  private lastSequence = -1;
  private traceEnabled = false;
  private trace: OutputEvent[] = [];
  private traceTruncated = false;
  private constructor(private readonly context: AudioContext, private readonly samples: Map<string, Sample>, readonly music: MusicRuntime) {}

  static async load(): Promise<AudioRuntime> {
    const context = new AudioContext({ sampleRate: 44100 });
    const paths = [...new Set(catalogue.sounds.flatMap(sound => sound.samples.map(sample => sample.path)))];
    const samples = new Map<string, Sample>();
    // Bound simultaneous resource requests while preserving all startup loads.
    let next = 0;
    try {
      await Promise.all(Array.from({ length: 8 }, async () => {
        while (next < paths.length) { const path = paths[next++]!; samples.set(path, await loadSample(context, path)); }
      }));
      return new AudioRuntime(context, samples, await MusicRuntime.load(context));
    } catch (error) { await context.close(); throw error; }
  }

  async unlock(): Promise<void> {
    if (this.context.state !== 'running') await this.context.resume();
    await this.music.unlock();
  }

  consume(events: readonly SoundEvent[]): void {
    for (const event of events) {
      if (event.sequence <= this.lastSequence) continue;
      this.lastSequence = event.sequence;
      // A suspended context retains the same bounded voice graph. The trusted
      // gesture resumes that graph; no second admission or burst queue is used.
      this.play(event);
    }
  }

  /** Room ambience loops (SFX 269 fire, SFX 4 insect swarm): the engine
   * starts each loop through an ordinary looping admission; afterwards the
   * live voice follows the loop's eased volume and pitch until it stops. */
  syncAmbience(ambience: Ambience, sfxVolume: number): void {
    for (const [id, loop] of [[269, ambience.fire], [4, ambience.swarm]] as const) {
      const paths = new Set(catalogue.sounds.find(sound => sound.id === id)?.samples.map(sample => sample.path) ?? []);
      for (let i = 0; i < this.voices.length; i++) {
        const voice = this.voices[i];
        if (!voice || !paths.has(voice.path) || voice.endsAt !== Infinity) continue;
        if (!loop.playing) { this.release(i); continue; }
        voice.gain.gain.value = Math.min(sfxVolume * loop.volume, 1);
        voice.source.playbackRate.value = loop.pitch;
      }
    }
  }

  private record(event: SoundEvent, voice: number | null, output: OutputEvent['output']): void {
    if (!this.traceEnabled) return;
    const path = catalogue.sounds.find(sound => sound.id === event.id)?.samples[event.sampleIndex]?.path;
    const sample = path && this.samples.get(path);
    if (!path || !sample) throw new Error('Admitted SFX does not exist in the authored catalogue');
    if (this.trace.length === TRACE_LIMIT) { this.traceTruncated = true; return; }
    this.trace.push({ ...event, path, voice, output, pcmSha256: sample.pcmSha256 });
  }

  private release(index: number): void {
    const voice = this.voices[index];
    if (!voice) return;
    this.voices[index] = null;
    voice.source.onended = null;
    voice.source.stop(); voice.source.disconnect(); voice.gain.disconnect();
  }

  private play(event: SoundEvent): void {
    const path = catalogue.sounds.find(sound => sound.id === event.id)?.samples[event.sampleIndex]?.path;
    const sample = path && this.samples.get(path);
    if (!path || !sample) throw new Error('Admitted SFX does not exist in the authored catalogue');
    // Source cache keys are authored paths. Alias IDs share the same backend.
    // Configured native SoundSource::Play stops its existing voice first.
    for (let i = 0; i < this.voices.length; i++) {
      const voice = this.voices[i];
      if (voice && (voice.path === path || voice.endsAt <= this.context.currentTime)) this.release(i);
    }
    const index = this.voices.findIndex(voice => voice === null);
    if (index < 0) { this.record(event, null, 'pool-full'); return; }
    const source = this.context.createBufferSource(), gain = this.context.createGain();
    source.buffer = sample.buffer; source.loop = event.looping;
    source.playbackRate.value = event.pitch; gain.gain.value = event.gain;
    source.connect(gain); gain.connect(this.context.destination);
    const voice = { source, gain, path, endsAt: event.looping ? Infinity : this.context.currentTime + sample.buffer.duration / event.pitch };
    this.voices[index] = voice;
    source.onended = () => { if (this.voices[index] === voice) { this.voices[index] = null; source.disconnect(); gain.disconnect(); } };
    source.start();
    this.record(event, index, this.context.state === 'running' ? 'started' : 'waiting-for-unlock');
  }

  reset(trace = false): void {
    for (let i = 0; i < this.voices.length; i++) this.release(i);
    this.lastSequence = -1;
    this.traceEnabled = trace; this.trace = []; this.traceTruncated = false;
    this.music.reset(trace);
  }

  snapshot() {
    return { contextState: this.context.state, contextSampleRate: this.context.sampleRate,
      loadedSamples: this.samples.size, voiceCapacity: this.voices.length,
      waitingForUnlock: this.context.state === 'running' ? 0 : this.voices.filter(Boolean).length,
      activeVoices: this.voices.map((voice, index) => voice && { voice: index, path: voice.path }).filter(Boolean),
      events: structuredClone(this.trace), truncated: this.traceTruncated, music: this.music.snapshot() };
  }
}
