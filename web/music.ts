import catalogue from './audio-data.json';
import type { MusicBackendCommand, MusicEvent, MusicState } from './engine';
import urls from 'virtual:original-music';
interface Track { id: number; path: string; intro?: string; layer?: string; layerintro?: string; loop: boolean; }
type Start = Extract<MusicEvent, { type: 'start' }>;
interface Wanted { start: Start; gain: number; pitch: number; looping: boolean; }
interface Stream {
  generation: number; epoch: number; start: Start; gain: GainNode;
  intro: AudioBufferSourceNode; loop: AudioBufferSourceNode | null;
  loopWhen: number; introEnded: boolean; remainingIntro: number; lastRateTime: number; pitch: number;
}

// Music uses three independent streaming channels, outside the 16 SFX voices.
export class MusicRuntime {
  private readonly decoded = new Map<string, Promise<AudioBuffer>>();
  private readonly streams: (Stream | null)[] = [null, null, null];
  private readonly wanted: (Wanted | null)[] = [null, null, null];
  private epoch = 0;
  private unlocked = false;
  private traceEnabled = false;
  private history: Record<string, unknown>[] = [];
  private truncated = false;
  private failures: string[] = [];
  private feedback: (command: MusicBackendCommand, audioTime: number) => void = () => {};
  private constructor(private readonly context: AudioContext) {}

  static async load(context: AudioContext): Promise<MusicRuntime> {
    const music = new MusicRuntime(context);
    // Prefetching bytes cannot admit an event or start output. Keep initial
    // Title and the short run jingle ready for the first trusted gesture.
    await Promise.all([music.track(61), music.track(95)]);
    return music;
  }
  onBackend(callback: (command: MusicBackendCommand, audioTime: number) => void): void { this.feedback = callback; }
  async unlock(): Promise<void> {
    this.unlocked = true;
    await this.startBatch(this.wanted.filter((wanted): wanted is Wanted => wanted !== null).map(wanted => wanted.start), this.epoch);
  }

  private async decode(path: string): Promise<AudioBuffer> {
    let pending = this.decoded.get(path);
    if (!pending) {
      pending = (async () => {
        const metadata = catalogue.assets[path as keyof typeof catalogue.assets], url = urls[path];
        if (!metadata || !('codec' in metadata) || !url) throw new Error(`Missing authored music resource: ${path}`);
        // Preserve literal '#' and spaces in authored names regardless of how
        // the bundler spells a generated URL. Asset URLs have no query/fragment.
        const safeUrl = url.split('/').map(segment => encodeURIComponent(decodeURIComponent(segment))).join('/');
        const response = await fetch(safeUrl);
        if (!response.ok) throw new Error(`Music load failed: ${path} (${response.status})`);
        const bytes = await response.arrayBuffer();
        const hash = Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256', bytes)), byte => byte.toString(16).padStart(2, '0')).join('');
        if (bytes.byteLength !== metadata.bytes || hash !== metadata.sha256) throw new Error(`Music bytes differ from the pinned original: ${path}`);
        const decoded = await this.context.decodeAudioData(bytes);
        // Vorbis end trimming: the final Ogg granule position counts the
        // stream's audio frames, and the original's vorbisfile decoder stops
        // there. The pinned catalogue holds that granule for these exact
        // bytes. Some browser decoders (Chromium 141) return the whole last
        // block instead; trim at most one long Vorbis block (8192 frames).
        const frames = Math.round(metadata.frames * decoded.sampleRate / metadata.sampleRate);
        const excess = decoded.length - frames;
        if (decoded.numberOfChannels !== metadata.channels || excess < -1 ||
            excess > Math.ceil(8192 * decoded.sampleRate / metadata.sampleRate))
          throw new Error(`Decoded original music format differs: ${path}`);
        const buffer = excess > 0
          ? new AudioBuffer({ length: frames, numberOfChannels: decoded.numberOfChannels, sampleRate: decoded.sampleRate })
          : decoded;
        // The original streaming backend submits AL_FORMAT_STEREO16. Preserve
        // its clipped PCM16 representation before applying channel gain/pitch.
        for (let channel = 0; channel < buffer.numberOfChannels; channel++) {
          const source = decoded.getChannelData(channel), samples = buffer.getChannelData(channel);
          for (let frame = 0; frame < samples.length; frame++)
            samples[frame] = Math.max(-32768, Math.min(32767, Math.round(source[frame]! * 32768))) / 32768;
        }
        return buffer;
      })();
      this.decoded.set(path, pending);
    }
    return pending;
  }
  private async track(id: number): Promise<void> {
    const track = (catalogue.music as Track[]).find(track => track.id === id);
    if (!track) throw new Error(`Unconfigured original music ID: ${id}`);
    await Promise.all([track.path, track.intro, track.layer, track.layerintro].filter((path): path is string => !!path).map(path => this.decode(path)));
  }
  private record(event: Record<string, unknown>): void {
    if (!this.traceEnabled) return;
    if (this.history.length === 8192) { this.truncated = true; return; }
    this.history.push({ audioTime: this.context.currentTime, ...event });
  }
  private fail(error: unknown): void {
    if (this.failures.length < 16) this.failures.push(String(error));
    console.error(error);
  }

  consume(state: MusicState): void {
    for (const request of state.pending) void this.track(request.id).catch(error => this.fail(error));
    if (state.queuedId) void this.track(state.queuedId).catch(error => this.fail(error));
    const starts: Start[] = [];
    for (const event of state.events) {
      const current = this.wanted[event.channel];
      if (event.type === 'start') {
        if (current?.start.generation === event.generation) continue;
        this.stop(event.channel);
        this.wanted[event.channel] = { start: event, gain: event.gain, pitch: event.pitch, looping: event.looping };
        starts.push(event);
      } else if (current?.start.generation === event.generation) {
        if (event.type === 'stop') { this.stop(event.channel); this.wanted[event.channel] = null; }
        else if (event.type === 'gain') {
          current.gain = event.gain;
          const stream = this.streams[event.channel]; if (stream) stream.gain.gain.value = event.gain;
          this.record(event);
        } else if (event.type === 'pitch') {
          current.pitch = event.pitch;
          const stream = this.streams[event.channel]; if (stream) this.pitch(stream, event.pitch);
          this.record(event);
        } else {
          const stream = this.streams[event.channel];
          if (stream) (stream.introEnded && stream.loop ? stream.loop : stream.intro).loop = event.looping;
          current.looping = event.looping;
          this.record(event);
        }
      }
    }
    if (starts.length) void this.startBatch(starts, this.epoch).catch(error => this.fail(error));
  }

  private async startBatch(starts: Start[], epoch: number): Promise<void> {
    const prepared = await Promise.all(starts.map(async start => ({ start, intro: await this.decode(start.path),
      loop: start.queuedPath ? await this.decode(start.queuedPath) : null })));
    if (epoch !== this.epoch || !this.unlocked) return;
    const when = this.context.currentTime;
    for (const { start, intro, loop } of prepared) {
      const wanted = this.wanted[start.channel];
      if (wanted?.start.generation !== start.generation || this.streams[start.channel]?.generation === start.generation) continue;
      const gain = this.context.createGain(); gain.gain.value = start.gain; gain.connect(this.context.destination);
      const source = this.context.createBufferSource(); source.buffer = intro; source.loop = start.looping;
      source.playbackRate.value = start.pitch; source.connect(gain);
      const stream: Stream = { generation: start.generation, epoch, start, gain, intro: source, loop: null,
        loopWhen: loop ? when + intro.duration / start.pitch : Infinity, introEnded: false,
        remainingIntro: intro.duration, lastRateTime: when, pitch: start.pitch };
      this.streams[start.channel] = stream;
      if (loop) {
        stream.loop = this.loopSource(stream, loop, stream.loopWhen);
        source.onended = () => {
          if (this.streams[start.channel] !== stream || epoch !== this.epoch) return;
          stream.introEnded = true; source.disconnect();
          const command: MusicBackendCommand = { type: 'musicBackend', channel: start.channel, generation: start.generation, event: 'loop' };
          this.record({ type: 'backend', command }); this.feedback(command, this.context.currentTime);
        };
      } else {
        source.onended = () => this.ended(stream, source);
      }
      source.start(when);
      this.record({ type: 'start', channel: start.channel, generation: start.generation, path: start.path,
        looping: start.looping, queuedPath: start.queuedPath, queuedLooping: start.queuedLooping,
        gain: start.gain, pitch: start.pitch, scheduledWhen: when });
      // Preserve source Play parameters even if asynchronous resource loading
      // completed after later engine fade updates. Apply the latest settings
      // after the actual start without inventing a past device start time.
      gain.gain.value = wanted.gain;
      source.loop = wanted.looping;
      if (wanted.pitch !== stream.pitch) this.pitch(stream, wanted.pitch);
    }
  }
  private loopSource(stream: Stream, buffer: AudioBuffer, when: number): AudioBufferSourceNode {
    const source = this.context.createBufferSource(); source.buffer = buffer; source.loop = stream.start.queuedLooping;
    source.playbackRate.value = stream.pitch; source.connect(stream.gain);
    source.onended = () => this.ended(stream, source); source.start(when);
    return source;
  }
  private ended(stream: Stream, source: AudioBufferSourceNode): void {
    // Every terminal decoder can finish after SetLoop(false). Old callbacks
    // from reset, explicit stop or a replaced pitch schedule cannot retire it.
    if (this.streams[stream.start.channel] !== stream || stream.epoch !== this.epoch ||
        source !== (stream.loop ?? stream.intro)) return;
    source.disconnect(); stream.gain.disconnect(); this.streams[stream.start.channel] = null;
    const command: MusicBackendCommand = { type: 'musicBackend', channel: stream.start.channel, generation: stream.generation, event: 'ended' };
    this.record({ type: 'backend', command }); this.feedback(command, this.context.currentTime);
  }
  private pitch(stream: Stream, pitch: number): void {
    const now = this.context.currentTime;
    if (stream.loop && !stream.introEnded && now < stream.loopWhen) {
      stream.remainingIntro = Math.max(0, stream.remainingIntro - (now - stream.lastRateTime) * stream.pitch);
      const buffer = stream.loop.buffer!; stream.loop.stop(); stream.loop.disconnect();
      stream.pitch = pitch; stream.loopWhen = now + stream.remainingIntro / pitch;
      stream.loop = this.loopSource(stream, buffer, stream.loopWhen);
    } else { stream.pitch = pitch; if (stream.loop) stream.loop.playbackRate.value = pitch; }
    stream.lastRateTime = now; stream.intro.playbackRate.value = pitch;
  }
  private stop(index: number): void {
    const stream = this.streams[index]; if (!stream) return;
    this.streams[index] = null;
    stream.intro.onended = null; stream.intro.stop(); stream.intro.disconnect();
    if (stream.loop) { stream.loop.onended = null; stream.loop.stop(); stream.loop.disconnect(); }
    stream.gain.disconnect(); this.record({ type: 'stop', channel: index, generation: stream.generation });
  }
  reset(trace = false): void {
    this.epoch++;
    for (let i = 0; i < this.streams.length; i++) this.stop(i);
    this.wanted.fill(null); this.traceEnabled = trace; this.history = []; this.truncated = false; this.failures = [];
  }
  snapshot() {
    return { decodedResources: this.decoded.size, waitingForUnlock: this.unlocked ? 0 : this.wanted.filter(Boolean).length,
      errors: [...this.failures], truncated: this.truncated,
      channels: this.streams.map((stream, channel) => stream && { channel, generation: stream.generation,
        path: stream.introEnded ? stream.start.queuedPath : stream.start.path,
        queuedPath: stream.introEnded ? null : stream.start.queuedPath,
        looping: stream.introEnded && stream.loop ? stream.loop.loop : stream.intro.loop,
        gain: stream.gain.gain.value, pitch: stream.pitch, loopWhen: Number.isFinite(stream.loopWhen) ? stream.loopWhen : null }),
      events: structuredClone(this.history) };
  }
}
