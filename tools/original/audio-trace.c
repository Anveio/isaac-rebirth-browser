/* Public OpenAL forwarding observer. No game instructions or memory
 * are patched. All OpenAL arguments and return behavior pass through unchanged.
 * Main-thread monotonic readings pass through the existing public clock gate;
 * worker timestamps are explicitly real clocks, not presentation phases. */
#define _GNU_SOURCE
#include <dlfcn.h>
#include <errno.h>
#include <fcntl.h>
#include <limits.h>
#include <math.h>
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <sys/uio.h>
#include <time.h>
#include <unistd.h>

typedef unsigned int ALuint;
typedef int ALenum;
typedef int ALint;
typedef int ALsizei;
typedef float ALfloat;
static pthread_mutex_t trace_lock = PTHREAD_MUTEX_INITIALIZER;
static FILE *events;
static pid_t owner_pid, main_tid;
static uint64_t sequence;
static int source_owners;
static int options_owners;
static void options_sfx237_owner(ALuint source, uintptr_t owner, uintptr_t caller, uintptr_t parent, uintptr_t pool_index);
static ALuint owned_sources[4096];
static size_t owned_source_count;
static int source_allocation_overflow;
static char output_dir[PATH_MAX];
static const size_t max_pcm_bytes = 16u * 1024u * 1024u;

__attribute__((constructor)) static void initialize_trace(void) {
    const char *directory = getenv("ISAAC_AUDIO_TRACE_DIR");
    const char *gate = getenv("ISAAC_CLOCK_GATE_SOCKET");
    if (!directory || !*directory || !gate || !*gate) return;
    char executable[PATH_MAX];
    ssize_t count = readlink("/proc/self/exe", executable, sizeof(executable) - 1);
    if (count <= 0) return;
    executable[count] = '\0';
    const char *name = strrchr(executable, '/');
    if (!name || strcmp(name + 1, "isaac.x64")) return;
    if (strlen(directory) >= sizeof(output_dir) - 64) return;
    strcpy(output_dir, directory);
    mkdir(output_dir, 0700);
    char filename[PATH_MAX + 64];
    snprintf(filename, sizeof(filename), "%s/calls.jsonl", output_dir);
    events = fopen(filename, "a");
    if (!events) return;
    source_owners = getenv("ISAAC_AUDIO_SFX_OWNERS") && !strcmp(getenv("ISAAC_AUDIO_SFX_OWNERS"), "1");
    options_owners = getenv("ISAAC_AUDIO_OPTIONS_OWNERS") && !strcmp(getenv("ISAAC_AUDIO_OPTIONS_OWNERS"), "1");
    owner_pid = getpid();
    main_tid = (pid_t)syscall(SYS_gettid);
    fprintf(events, "{\"schema\":1,\"api\":\"trace-start\",\"pid\":%d,\"mainTid\":%d,\"scope\":\"Forwarded public OpenAL calls and PCM arguments; original hash/direct-child/gate identity validated by paired source observer\"}\n", owner_pid, main_tid);
    fflush(events);
}

static int begin_event(const char *api) {
    if (!events || getpid() != owner_pid) return 0;
    pthread_mutex_lock(&trace_lock);
    struct timespec stamp = {0};
    clock_gettime(CLOCK_MONOTONIC, &stamp);
    uint64_t ns = (uint64_t)stamp.tv_sec * UINT64_C(1000000000) + (uint64_t)stamp.tv_nsec;
    /* Observer-only kernel time: do not change the game's gated clock. */
    struct timespec host_stamp = {0};
    int host_ok = syscall(SYS_clock_gettime, CLOCK_MONOTONIC, &host_stamp) == 0;
    uint64_t host_ns = (uint64_t)host_stamp.tv_sec * UINT64_C(1000000000) + (uint64_t)host_stamp.tv_nsec;
    pid_t tid = (pid_t)syscall(SYS_gettid);
    fprintf(events, "{\"schema\":1,\"seq\":%llu,\"api\":\"%s\",\"pid\":%d,\"tid\":%d,\"monotonicNs\":%llu,\"hostMonotonicNs\":",
            (unsigned long long)++sequence, api, owner_pid, tid, (unsigned long long)ns);
    if (host_ok) fprintf(events, "%llu", (unsigned long long)host_ns); else fputs("null", events);
    fprintf(events, ",\"clockDomain\":\"%s\",\"virtualElapsedNs\":",
            tid == main_tid ? "gate-main-thread" : "real-worker-thread");
    if (tid == main_tid && ns >= UINT64_C(1000000000000))
        fprintf(events, "%llu", (unsigned long long)(ns - UINT64_C(1000000000000)));
    else fputs("null", events);
    return 1;
}

static void end_event(void) {
    fputs("}\n", events);
    fflush(events);
    pthread_mutex_unlock(&trace_lock);
}

static void ids_event(const char *api, ALsizei count, const ALuint *ids) {
    if (!begin_event(api)) return;
    if (source_owners && ids && count >= 0) {
        if (!strcmp(api, "alGenSources")) {
            for (int i = 0; i < count; i++) {
                if (owned_source_count == 4096) { source_allocation_overflow = 1; break; }
                owned_sources[owned_source_count++] = ids[i];
            }
        } else if (!strcmp(api, "alDeleteSources")) {
            for (int i = 0; i < count && i < 4096; i++) {
                for (size_t j = 0; j < owned_source_count; j++) {
                    if (owned_sources[j] == ids[i]) { owned_sources[j] = owned_sources[--owned_source_count]; break; }
                }
            }
        }
    }
    fprintf(events, ",\"count\":%d,\"ids\":[", count);
    int limit = count > 4096 ? 4096 : count;
    for (int i = 0; ids && i < limit; i++) fprintf(events, "%s%u", i ? "," : "", ids[i]);
    fprintf(events, "],\"truncated\":%s", count > limit ? "true" : "false");
    end_event();
}

static void source_ids_event(const char *api, ALuint source, ALsizei count, const ALuint *ids) {
    if (!begin_event(api)) return;
    fprintf(events, ",\"source\":%u,\"count\":%d,\"buffers\":[", source, count);
    int limit = count > 4096 ? 4096 : count;
    for (int i = 0; ids && i < limit; i++) fprintf(events, "%s%u", i ? "," : "", ids[i]);
    fprintf(events, "],\"truncated\":%s", count > limit ? "true" : "false");
    end_event();
}

#define RESOLVE(name, signature) signature = dlsym(RTLD_NEXT, #name)
#define IDS_WRAPPER(name, qualifier) \
void name(ALsizei count, qualifier ALuint *ids) { \
    RESOLVE(name, void (*real)(ALsizei, qualifier ALuint *)); \
    real(count, ids); ids_event(#name, count, ids); \
}
IDS_WRAPPER(alGenSources, )
IDS_WRAPPER(alGenBuffers, )
IDS_WRAPPER(alDeleteSources, const)
IDS_WRAPPER(alDeleteBuffers, const)

void alBufferData(ALuint buffer, ALenum format, const void *data, ALsizei size, ALsizei rate) {
    RESOLVE(alBufferData, void (*real)(ALuint, ALenum, const void *, ALsizei, ALsizei));
    real(buffer, format, data, size, rate);
    if (!begin_event("alBufferData")) return;
    char relative[96], path[PATH_MAX + 128];
    snprintf(relative, sizeof(relative), "buffer-%u-upload-%llu.pcm", buffer, (unsigned long long)sequence);
    snprintf(path, sizeof(path), "%s/%s", output_dir, relative);
    size_t wanted = size > 0 ? (size_t)size : 0;
    size_t stored = wanted > max_pcm_bytes ? max_pcm_bytes : wanted;
    long written = -1;
    FILE *pcm = fopen(path, "wb");
    if (pcm) {
        written = data || !stored ? (long)fwrite(data, 1, stored, pcm) : -1;
        fclose(pcm);
    }
    fprintf(events, ",\"buffer\":%u,\"format\":%d,\"rate\":%d,\"bytes\":%d,\"pcmArtifact\":\"%s\",\"artifactBytes\":%ld,\"truncated\":%s",
            buffer, format, rate, size, relative, written, stored < wanted ? "true" : "false");
    end_event();
}

void alSourcei(ALuint source, ALenum parameter, ALint value) {
    RESOLVE(alSourcei, void (*real)(ALuint, ALenum, ALint));
    real(source, parameter, value);
    if (begin_event("alSourcei")) {
        fprintf(events, ",\"source\":%u,\"parameter\":%d,\"value\":%d", source, parameter, value);
        end_event();
    }
}

__attribute__((used, noinline)) static void observe_al_source_f(ALuint source, ALenum parameter, ALfloat value, uintptr_t caller, uintptr_t wrapper, uintptr_t parent) {
    RESOLVE(alSourcef, void (*real)(ALuint, ALenum, ALfloat));
    real(source, parameter, value);
    if (begin_event("alSourcef")) {
        uint32_t bits; memcpy(&bits, &value, sizeof(bits));
        fprintf(events, ",\"source\":%u,\"parameter\":%d,\"value\":", source, parameter);
        if (isfinite(value)) fprintf(events, "%.9g", (double)value); else fputs("null", events);
        fprintf(events, ",\"valueBits\":%u", bits);
        if (options_owners && (caller == 0x5e1e37UL || caller == 0x5e1a76UL ||
            (caller == 0x610dcfUL && (parent == 0x5a81f4UL || parent == 0x5a8337UL))) && parameter == 0x100a)
            options_sfx237_owner(source, wrapper, caller, parent, 0);
        end_event();
    }
}

/* Gain612eb0 -> backend610c00 tailcalls alSourcef; the real caller remains
 * 5e1e37 and callee-saved R12 remains the configured wrapper. XMM0 and all
 * actual API arguments are forwarded unchanged. Configured request5e1980
 * uses RBX for its chosen wrapper and reaches gain with return5e1a76.
 * A first stored gain may emit no API at handle0; BasePlay later applies it
 * at610dcf, with RBX=backend and the actual Options return32bytes above RSP. */
__attribute__((naked)) void alSourcef(ALuint source __attribute__((unused)), ALenum parameter __attribute__((unused)), ALfloat value __attribute__((unused))) {
    __asm__ __volatile__("movq (%rsp), %rdx\n\t"
                         "movq %r12, %rcx\n\t"
                         "xorl %r8d, %r8d\n\t"
                         "cmpq $0x5e1a76, %rdx\n\t"
                         "je 1f\n\t"
                         "cmpq $0x610dcf, %rdx\n\t"
                         "jne 2f\n\t"
                         "movq 32(%rsp), %r8\n\t"
                         "1: movq %rbx, %rcx\n\t"
                         "2: jmp observe_al_source_f\n\t");
}

#define SOURCE_WRAPPER(name) \
void name(ALuint source) { \
    RESOLVE(name, void (*real)(ALuint)); real(source); \
    if (begin_event(#name)) { fprintf(events, ",\"source\":%u", source); end_event(); } \
}

/* Opt-in source ownership at the actual configured-SFX OpenAL call.
 * Original610d30 keeps its actual backend object in callee-saved RBX and
 * calls alSourcePlay at610de6 (return610deb). The trampoline captures RBX
 * before any C prologue may reuse it. No executable/game memory is patched.
 * Exact table-reference ambiguity is retained; PCM never selects an ID. */
static uintptr_t last_owner_read_address;
static size_t last_owner_read_requested;
static ssize_t last_owner_read_returned;
static int last_owner_read_errno;
static int last_owner_read_attempted;
static int last_owner_open_attempted;
static int last_owner_open_errno;
static int own_read(uintptr_t address, void *target, size_t length) {
    int saved_errno = errno;
    ssize_t returned = 0;
    int read_errno = 0;
    last_owner_read_attempted = 0;
    last_owner_open_attempted = address != 0;
    last_owner_open_errno = 0;
    if (last_owner_open_attempted) {
        errno = 0;
        int fd = open("/proc/self/mem", O_RDONLY | O_CLOEXEC);
        last_owner_open_errno = errno;
        if (fd >= 0) {
            last_owner_read_attempted = 1;
            errno = 0;
            returned = pread(fd, target, length, (off_t)address);
            read_errno = errno;
            close(fd);
        }
    }
    last_owner_read_address = address;
    last_owner_read_requested = length;
    last_owner_read_returned = returned;
    last_owner_read_errno = read_errno;
    errno = saved_errno;
    return last_owner_read_attempted && returned == (ssize_t)length;
}
static uint32_t word32(const unsigned char *at) {
    uint32_t value; memcpy(&value, at, sizeof(value)); return value;
}
static uintptr_t pointer64(const unsigned char *at) {
    uintptr_t value; memcpy(&value, at, sizeof(value)); return value;
}
static void quoted_bytes(const unsigned char *text, size_t length) {
    fputc('"', events);
    for (size_t i = 0; i < length; i++) {
        unsigned char ch = text[i];
        if (ch == '"' || ch == '\\') { fputc('\\', events); fputc(ch, events); }
        else if (ch < 0x20 || ch >= 0x7f) fprintf(events, "\\u%04x", ch);
        else fputc(ch, events);
    }
    fputc('"', events);
}
/* This leaf observes one Source-located configured wrapper, not all aliases
 * or an inferred ID selected from PCM. The prior real start must independently
 * have the existing UNIQUE_ACTUAL_TABLE_REFERENCE owner for237. */
static int options_read(uintptr_t address, void *target, size_t length, size_t *bytes) {
    if (*bytes > 1024 || length > 1024 - *bytes) return 0;
    *bytes += length;
    return own_read(address, target, length);
}
static void options_sfx237_owner(ALuint source, uintptr_t owner, uintptr_t caller, uintptr_t parent, uintptr_t pool_index) {
    uintptr_t system = 0, backend = 0, vtable = 0, path = 0, input_backend = owner;
    uint32_t actual_source = 0, counter = 0, volume = 0, backend_gain = 0, pool_source = 0;
    unsigned char slot[0x158], text[512]; uint64_t length = 0; size_t bytes = 0;
    int materialized = caller == 0x610dcfUL, restart_stop = caller == 0x62a2c9UL, pool_reset = caller == 0x610a87UL;
    int backend_input = materialized || restart_stop || pool_reset;
    fprintf(events, ",\"optionsOwner\":{\"schema\":\"actual-options-sfx237-owner-v2\",\"callSite\":\"0x%lx\",\"parentCallSite\":\"0x%lx\",\"actualInputOwnerAddress\":\"0x%lx\",\"ownerInputRole\":\"%s\"",
            (unsigned long)caller, (unsigned long)parent, (unsigned long)owner,
            pool_reset ? "actual-pool-manager-rbx" : backend_input ? "actual-backend-rbx" : "actual-wrapper-r12-or-rbx");
    if ((pid_t)syscall(SYS_gettid) != main_tid ||
        !((caller == 0x610d14UL && parent == 0x5e1e14UL) ||
          ((caller == 0x5e1e37UL || caller == 0x5e1a76UL || restart_stop || pool_reset) && !parent) ||
          (materialized && (parent == 0x5a81f4UL || parent == 0x5a8337UL)))) {
        fputs(",\"status\":\"UNKNOWN_UNSELECTED_CALLER_OR_THREAD\"}", events); return;
    }
    const char *operation = caller == 0x610d14UL ? "global-volume-is-playing" :
        caller == 0x5e1e37UL ? "global-volume-live-gain" : caller == 0x5e1a76UL ? "configured-feedback-admitted-gain" :
        materialized ? "backend-play-materialized-gain" : restart_stop ? "sample-restart-stop" : "pool-reset-before-binding";
    fprintf(events, ",\"operation\":\"%s\"", operation);
    if (!options_read(0x8ed560UL, &system, sizeof(system), &bytes) || !system) goto unknown;
    uintptr_t slot_at = system + 0x12c20UL + 237UL * 0x158UL;
    if (!backend_input && owner != slot_at + 0x10) {
        fprintf(events, ",\"bytesRead\":%zu,\"status\":\"UNSELECTED_CONFIGURED_WRAPPER\"}", bytes); return;
    }
    if (pool_reset) {
        if (owner != 0x934940UL || pool_index >= 16 ||
            !options_read(owner + 0x88 + pool_index * 8, &input_backend, sizeof(input_backend), &bytes) ||
            !options_read(owner + 0x108 + pool_index * 4, &pool_source, sizeof(pool_source), &bytes) || pool_source != source) goto unknown;
        fprintf(events, ",\"actualPoolIndex\":%lu,\"actualPoolSource\":%u,\"sourceHandleBinding\":\"pending-pool-claim\"", (unsigned long)pool_index, pool_source);
    }
    if (!options_read(slot_at, slot, sizeof(slot), &bytes)) goto unknown;
    backend = pointer64(slot + 0x10); path = pointer64(slot + 8);
    if (backend_input && backend != input_backend) {
        fprintf(events, ",\"bytesRead\":%zu,\"status\":\"UNSELECTED_CONFIGURED_BACKEND\"}", bytes); return;
    }
    if (word32(slot + 0x148) != 1 || slot[0x150] != 1 || !backend ||
        !options_read(backend, &vtable, sizeof(vtable), &bytes) || vtable != 0x67cf10UL ||
        !options_read(backend + 0x78, &actual_source, sizeof(actual_source), &bytes) ||
        (!pool_reset && actual_source != source) ||
        !options_read(backend + 0x7c, &backend_gain, sizeof(backend_gain), &bytes) ||
        !options_read(system + 0x2efa0, &counter, sizeof(counter), &bytes) ||
        !options_read(system + 0x2df94, &volume, sizeof(volume), &bytes) || path < 24 ||
        !options_read(path - 24, &length, sizeof(length), &bytes) || !length || length > sizeof(text) ||
        !options_read(path, text, (size_t)length, &bytes)) goto unknown;
    fprintf(events, ",\"id\":237,\"sampleIndex\":0,\"wrapperAddress\":\"0x%lx\",\"backendObject\":\"0x%lx\",\"backendClassVtable\":\"0x%lx\",\"actualBackendSource\":%u,\"systemCounter\":%u,\"globalVolumeBits\":%u,\"requestedVolumeBits\":%u,\"wrapperGainBits\":%u,\"backendGainBits\":%u,\"path\":",
            (unsigned long)(slot_at + 0x10), (unsigned long)backend, (unsigned long)vtable, actual_source, counter, volume,
            word32(slot + 0x14c), word32(slot + 0x10 + 0x18), backend_gain);
    quoted_bytes(text, (size_t)length);
    fprintf(events, ",\"bytesRead\":%zu,\"maxBytes\":1024,\"status\":\"ACTUAL_CONFIGURED_237_WRAPPER\"}", bytes); return;
unknown:
    fprintf(events, ",\"bytesRead\":%zu,\"maxBytes\":1024,\"status\":\"UNKNOWN_REQUIRED_READ_OR_SOURCE_IDENTITY\"}", bytes);
}

static void configured_sfx_owner(ALuint source, uintptr_t backend, uintptr_t caller) {
    fprintf(events, ",\"sfxOwner\":{\"schema\":\"actual-openal-sfx-owner-v1\",\"callSite\":\"0x%lx\",\"backendObject\":\"0x%lx\"",
            (unsigned long)caller, (unsigned long)backend);
    if ((pid_t)syscall(SYS_gettid) != main_tid || caller != 0x610debUL) {
        fputs(",\"status\":\"UNKNOWN_UNSELECTED_CALLER_OR_THREAD\"}", events); return;
    }
    uintptr_t system = 0, game = 0;
    uint32_t backend_source = 0, counter = 0, game_counter = 0, volume_bits = 0;
    unsigned char table[324 * 0x158];
    const char *failed_stage = NULL;
    int system_read = 0, backend_source_read = 0;
    if (!own_read(0x8ed560UL, &system, sizeof(system))) failed_stage = "system-singleton-read";
    else {
        system_read = 1;
        if (!system) failed_stage = "system-singleton-null";
        else if (!own_read(backend + 0x78, &backend_source, sizeof(backend_source))) failed_stage = "backend-source78-read";
        else {
            backend_source_read = 1;
            if (backend_source != source) failed_stage = "actual-source78-handle-mismatch";
            else if (!own_read(system + 0x2efa0, &counter, sizeof(counter))) failed_stage = "system-counter-read";
            else if (!own_read(system + 0x2df94, &volume_bits, sizeof(volume_bits))) failed_stage = "global-volume-read";
            else if (!own_read(system + 0x12c20, table, sizeof(table))) failed_stage = "configured-table-read";
        }
    }
    if (failed_stage) {
        fprintf(events, ",\"failedStage\":\"%s\",\"readResult\":{\"api\":\"pread-proc-self-mem\",\"attempted\":%s,\"address\":\"0x%lx\",\"requestedBytes\":%zu,\"returnedBytes\":",
                failed_stage, last_owner_read_attempted ? "true" : "false",
                (unsigned long)last_owner_read_address, last_owner_read_requested);
        if (last_owner_read_attempted) fprintf(events, "%zd,\"errno\":%d}", last_owner_read_returned, last_owner_read_errno);
        else fputs("null,\"errno\":null}", events);
        fprintf(events, ",\"openResult\":{\"api\":\"open-proc-self-mem\",\"attempted\":%s,\"errno\":", last_owner_open_attempted ? "true" : "false");
        if (last_owner_open_attempted) fprintf(events, "%d", last_owner_open_errno); else fputs("null", events);
        fprintf(events, "},\"actualInputSource\":%u,\"observedSystemPointer\":", source);
        if (system_read) fprintf(events, "\"0x%lx\"", (unsigned long)system); else fputs("null", events);
        fputs(",\"observedBackendSource78\":", events);
        if (backend_source_read) fprintf(events, "%u", backend_source); else fputs("null", events);
        fputs(",\"status\":\"UNKNOWN_REQUIRED_READ_OR_ACTUAL_SOURCE_HANDLE\"}", events); return;
    }
    fprintf(events, ",\"systemCounter\":%u,\"globalVolumeBits\":%u,\"gameUpdateCount\":", counter, volume_bits);
    if (own_read(0x8ecd30UL, &game, sizeof(game)) && game && own_read(game + 0xce9f4, &game_counter, sizeof(game_counter)))
        fprintf(events, "%u", game_counter);
    else fputs("null", events);
    unsigned matches = 0; int complete = 1;
    fputs(",\"references\":[", events);
    for (unsigned id = 0; id < 324; id++) {
        const unsigned char *slot = table + id * 0x158;
        uint32_t count = word32(slot + 0x148);
        if (count > 5) { complete = 0; continue; }
        if (!count) continue;
        if (slot[0x150] > 1) complete = 0;
        for (unsigned sample = 0; sample < count; sample++) {
            const unsigned char *wrapper = slot + 0x10 + sample * 64;
            if (pointer64(wrapper) != backend) continue;
            uintptr_t path = pointer64(slot + 8 + sample * 64);
            uint64_t length = 0; unsigned char text[512];
            int path_known = path >= 24 && own_read(path - 24, &length, sizeof(length)) && length > 0 && length <= sizeof(text) && own_read(path, text, (size_t)length);
            if (!path_known || wrapper[0x14] > 1) complete = 0;
            uint32_t delay_bits = word32(slot + 4) - word32(slot);
            fprintf(events, "%s{\"id\":%u,\"sampleIndex\":%u,\"wrapperAddress\":\"0x%lx\",\"lastTickBits\":%u,\"nextAllowedBits\":%u,\"delayBits\":%u,\"requestedVolumeBits\":%u,\"loaded\":%u,\"loopRaw\":%u,\"gainBits\":%u,\"pitchBits\":%u,\"path\":",
                    matches ? "," : "", id, sample, (unsigned long)(system + 0x12c20 + id * 0x158 + 0x10 + sample * 64),
                    word32(slot), word32(slot + 4), delay_bits, word32(slot + 0x14c), slot[0x150], wrapper[0x14], word32(wrapper + 0x18), word32(wrapper + 0x20));
            if (path_known) quoted_bytes(text, (size_t)length); else fputs("null", events);
            fputc('}', events); matches++;
        }
    }
    fprintf(events, "],\"referenceCount\":%u,\"completeTableAndPathRead\":%s,\"status\":\"%s\"}", matches,
            complete ? "true" : "false", !complete ? "UNKNOWN_INCOMPLETE_REFERENCE_READ" : matches == 1 ? "UNIQUE_ACTUAL_TABLE_REFERENCE" : matches ? "AMBIGUOUS_ACTUAL_TABLE_REFERENCES" : "UNKNOWN_NO_TABLE_REFERENCE");
}

/* SysVAMD64 callee-saved owner+actual returnPC capture only. Other architectures
 * are not this pinned Original executable and must fail compilation. */
#if !defined(__x86_64__)
#error "Pinned Original audio ownership requires SysVAMD64"
#endif
__attribute__((used, noinline)) static void observe_al_source_play(ALuint source, uintptr_t backend, uintptr_t caller) {
    RESOLVE(alSourcePlay, void (*real)(ALuint)); real(source);
    if (begin_event("alSourcePlay")) {
        fprintf(events, ",\"source\":%u", source);
        if (source_owners) {
            int allocated = 0;
            for (size_t i = 0; i < owned_source_count; i++) if (owned_sources[i] == source) { allocated = 1; break; }
            fprintf(events, ",\"actualSourceAllocationKnown\":%s,\"sourceAllocationOverflow\":%s,\"postPlayState\":", allocated ? "true" : "false", source_allocation_overflow ? "true" : "false");
            if (allocated && !source_allocation_overflow && caller == 0x610debUL && (pid_t)syscall(SYS_gettid) == main_tid) {
                RESOLVE(alGetSourcei, void (*query)(ALuint, ALenum, ALint *));
                RESOLVE(alGetSourcef, void (*query_float)(ALuint, ALenum, ALfloat *));
                ALint state = 0, buffer = 0, loop = -1;
                ALfloat gain = 0, pitch = 0;
                if (query && query_float) {
                    query(source, 0x1010, &state);
                    query(source, 0x1009, &buffer);
                    query(source, 0x1007, &loop);
                    query_float(source, 0x100a, &gain);
                    query_float(source, 0x1003, &pitch);
                }
                if (state >= 0x1011 && state <= 0x1014) fprintf(events, "%d", state); else fputs("null", events);
                uint32_t gain_bits, pitch_bits;
                memcpy(&gain_bits, &gain, sizeof(gain_bits)); memcpy(&pitch_bits, &pitch, sizeof(pitch_bits));
                fprintf(events, ",\"actualMixer\":{\"state\":%d,\"buffer\":%u,\"loopRaw\":%d,\"gainBits\":%u,\"pitchBits\":%u,\"complete\":%s}",
                        state, (ALuint)buffer, loop, gain_bits, pitch_bits,
                        query && query_float && state >= 0x1011 && state <= 0x1014 && buffer && (loop == 0 || loop == 1) && isfinite(gain) && isfinite(pitch) ? "true" : "false");
            } else fputs("null,\"actualMixer\":null", events);
            configured_sfx_owner(source, backend, caller);
        }
        end_event();
    }
}
__attribute__((naked)) void alSourcePlay(ALuint source __attribute__((unused))) {
    __asm__ __volatile__("movq %rbx, %rsi\n\t"
                         "movq (%rsp), %rdx\n\t"
                         "jmp observe_al_source_play\n\t");
}

/* Global-volume scan5e1e0f calls IsPlaying612e20, which tailcalls610cf0.
 * That backend calls real alGetSourcei once (return610d14), then compares its
 * actual output with AL_PLAYING. Its untouched caller is32bytes above the API
 * entry stack (5e1e14); R12 is the actual sample wrapper. No extra query. */
__attribute__((used, noinline)) static void observe_al_get_source_i(ALuint source, ALenum parameter, ALint *value, uintptr_t caller, uintptr_t parent, uintptr_t wrapper) {
    RESOLVE(alGetSourcei, void (*real)(ALuint, ALenum, ALint *));
    real(source, parameter, value);
    if (!options_owners || caller != 0x610d14UL || parent != 0x5e1e14UL || parameter != 0x1010) return;
    if (begin_event("alGetSourcei")) {
        fprintf(events, ",\"source\":%u,\"parameter\":%d,\"value\":", source, parameter);
        if (value) fprintf(events, "%d", *value); else fputs("null", events);
        options_sfx237_owner(source, wrapper, caller, parent, 0);
        end_event();
    }
}
__attribute__((naked)) void alGetSourcei(ALuint source __attribute__((unused)), ALenum parameter __attribute__((unused)), ALint *value __attribute__((unused))) {
    __asm__ __volatile__("movq (%rsp), %rcx\n\t"
                         "xorl %r8d, %r8d\n\t"
                         "xorl %r9d, %r9d\n\t"
                         "cmpq $0x610d14, %rcx\n\t"
                         "jne 1f\n\t"
                         "movq 32(%rsp), %r8\n\t"
                         "movq %r12, %r9\n\t"
                         "1: jmp observe_al_get_source_i\n\t");
}

SOURCE_WRAPPER(alSourcePause)
/* Capture actual callsite and callee-saved owners before a C prologue.
 * Class restart62a2c9 owns RBX=backend. Pool reset610a87 owns RBX=manager,
 * R14=index; its selected backend is already in the actual pool table.
 * No extra query, command, source stop or API argument is introduced. */
__attribute__((used, noinline)) static void observe_al_source_stop(ALuint source, uintptr_t caller, uintptr_t owner, uintptr_t pool_index) {
    RESOLVE(alSourceStop, void (*real)(ALuint)); real(source);
    if (begin_event("alSourceStop")) {
        fprintf(events, ",\"source\":%u", source);
        if (options_owners) fprintf(events, ",\"callSite\":\"0x%lx\"", (unsigned long)caller);
        if (options_owners && (caller == 0x62a2c9UL || caller == 0x610a87UL))
            options_sfx237_owner(source, owner, caller, 0, pool_index);
        end_event();
    }
}
__attribute__((naked)) void alSourceStop(ALuint source __attribute__((unused))) {
    __asm__ __volatile__("movq (%rsp), %rsi\n\t"
                         "movq %rbx, %rdx\n\t"
                         "movq %r14, %rcx\n\t"
                         "jmp observe_al_source_stop\n\t");
}

void alSourceQueueBuffers(ALuint source, ALsizei count, const ALuint *buffers) {
    RESOLVE(alSourceQueueBuffers, void (*real)(ALuint, ALsizei, const ALuint *));
    real(source, count, buffers);
    source_ids_event("alSourceQueueBuffers", source, count, buffers);
}

void alSourceUnqueueBuffers(ALuint source, ALsizei count, ALuint *buffers) {
    RESOLVE(alSourceUnqueueBuffers, void (*real)(ALuint, ALsizei, ALuint *));
    real(source, count, buffers);
    source_ids_event("alSourceUnqueueBuffers", source, count, buffers);
}
