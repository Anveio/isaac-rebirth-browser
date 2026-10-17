import type { State } from './engine';

type Request = State['fullscreen']['requests'][number];
type Job = Request & {generation: number; deferred: boolean};
type EventRow = Record<string, unknown>;

/** Actual host operations/observations. Canonical intent remains in the engine. */
export class FullscreenRuntime {
  private readonly events: EventRow[] = [];
  private readonly queue: Job[] = [];
  private readonly seen = new Map<number, Request>();
  private nextSequence = 0;
  private generation = 0;
  private running = false;
  private truncated = false;

  constructor(private readonly root: HTMLElement, private readonly failure: (message: string) => void) {
    document.addEventListener('fullscreenchange', event => this.record({
      type: 'fullscreenchange', trusted: event.isTrusted, actual: this.actual(),
    }));
    document.addEventListener('fullscreenerror', event => this.record({
      type: 'fullscreenerror', trusted: event.isTrusted, actual: this.actual(),
    }));
  }

  consume(requests: State['fullscreen']['requests']): void {
    for (const request of requests) {
      if (!Number.isSafeInteger(request.sequence) || request.sequence < 0 ||
          typeof request.requested !== 'boolean' || !['startup', 'options'].includes(request.origin))
        throw new Error('Invalid source fullscreen request');
      const previous = this.seen.get(request.sequence);
      if (previous) {
        if (previous.requested !== request.requested || previous.origin !== request.origin)
          throw new Error('Fullscreen request identity changed');
        continue;
      }
      if (request.sequence !== this.nextSequence || this.queue.length >= 512)
        throw new Error('Fullscreen request sequence/capacity is unavailable');
      this.nextSequence++;
      this.seen.set(request.sequence, {...request});
      //A single command is bounded to512 effects below; retain that complete
      //deduplication window without growing for the lifetime of the page.
      if (this.seen.size > 512) this.seen.delete(this.seen.keys().next().value!);
      this.queue.push({...request, generation: this.generation, deferred: false});
      this.record({type: 'source-request', request: {...request}, actual: this.actual()});
    }
    void this.drain();
  }

  gesture(event: Event): void {
    if (!event.isTrusted) return;
    this.record({type: 'trusted-gesture', event: event.type, activation: this.activation()});
    void this.drain();
  }

  assertIdleForReset(): void {
    if (this.running || this.queue.length !== 0)
      throw new Error('A source fullscreen request is still pending; reset cannot discard its outcome');
  }

  reset(requests: State['fullscreen']['requests']): void {
    this.assertIdleForReset();
    this.generation++;
    this.nextSequence = 0;
    this.seen.clear();
    this.record({type: 'engine-reset', actual: this.actual()});
    this.consume(requests);
  }

  snapshot() {
    return {schema: 1, supported: typeof this.root.requestFullscreen === 'function' &&
      typeof document.exitFullscreen === 'function', enabled: document.fullscreenEnabled,
      actual: this.actual(), generation: this.generation, running: this.running,
      pending: this.queue.map(({sequence, requested, origin, generation, deferred}) =>
        ({sequence, requested, origin, generation, deferred})),
      events: structuredClone(this.events), truncated: this.truncated,
      scope: 'Actual browser fullscreen API/promises/change/error/DOM observations. Source requested intent and desktop window geometry remain separate.'};
  }

  private activation() {
    return {active: navigator.userActivation?.isActive ?? false,
      everActive: navigator.userActivation?.hasBeenActive ?? false};
  }

  private actual() {
    const element = document.fullscreenElement;
    return {rootActive: element === this.root, anyActive: element !== null,
      element: element ? {tag: element.tagName, id: element.id} : null,
      viewport: [innerWidth, innerHeight], rootConnected: this.root.isConnected};
  }

  private record(row: EventRow): void {
    if (this.events.length === 4096) this.truncated = true;
    else this.events.push({generation: this.generation, at: performance.now(), ...structuredClone(row)});
  }

  private async drain(): Promise<void> {
    if (this.running) return;
    this.running = true;
    try {
      while (this.queue.length) {
        const job = this.queue[0]!;
        const before = this.actual();
        //A loaded startup request has no user activation of its own. Keep it
        //pending until a real gesture; do not invent automatic host success.
        if (job.origin === 'startup' && job.requested && !before.rootActive && !this.activation().active) {
          if (!job.deferred) this.record({type: 'deferred-activation', request: job, actual: before});
          job.deferred = true;
          return;
        }
        try {
          if (!before.rootConnected) throw new Error('Fullscreen game root is disconnected');
          if (before.anyActive && !before.rootActive) throw new Error('Another element owns browser fullscreen');
          if (job.requested === before.rootActive) {
            this.record({type: 'already-matching', request: job, actual: before});
          } else {
            const method = job.requested ? 'requestFullscreen' : 'exitFullscreen';
            this.record({type: 'api-call', request: job, method, activation: this.activation(), actual: before});
            //Forward the real operation even when an Options caller lacks
            //activation. Its genuine API rejection is required evidence.
            if (job.requested) await this.root.requestFullscreen();
            else await document.exitFullscreen();
            const actual = this.actual();
            this.record({type: 'api-resolved', request: job, actual,
              matchesRequested: actual.rootActive === job.requested && (!actual.anyActive || actual.rootActive)});
            if (actual.rootActive !== job.requested || (actual.anyActive && !actual.rootActive))
              throw new Error('Fullscreen promise resolved without the requested actual element state');
          }
        } catch (error) {
          const detail = error instanceof Error ? {name: error.name, message: error.message} : {name: 'Error', message: String(error)};
          this.record({type: 'request-failed', request: job, error: detail, actual: this.actual()});
          this.failure(`Fullscreen request failed: ${detail.message}`);
        }
        this.queue.shift();
      }
    } finally {
      this.running = false;
    }
  }
}
