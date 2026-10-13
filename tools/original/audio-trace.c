/* Public OpenAL forwarding observer. No game instructions or memory
 * are patched. All OpenAL arguments and return behavior pass through unchanged.
 * Main-thread monotonic readings pass through the existing public clock gate;
 * worker timestamps are explicitly real clocks, not presentation phases. */
#define _GNU_SOURCE
#include <dlfcn.h>
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

void alSourcef(ALuint source, ALenum parameter, ALfloat value) {
    RESOLVE(alSourcef, void (*real)(ALuint, ALenum, ALfloat));
    real(source, parameter, value);
    if (begin_event("alSourcef")) {
        uint32_t bits; memcpy(&bits, &value, sizeof(bits));
        fprintf(events, ",\"source\":%u,\"parameter\":%d,\"value\":", source, parameter);
        if (isfinite(value)) fprintf(events, "%.9g", (double)value); else fputs("null", events);
        fprintf(events, ",\"valueBits\":%u", bits);
        end_event();
    }
}

#define SOURCE_WRAPPER(name) \
void name(ALuint source) { \
    RESOLVE(name, void (*real)(ALuint)); real(source); \
    if (begin_event(#name)) { fprintf(events, ",\"source\":%u", source); end_event(); } \
}
SOURCE_WRAPPER(alSourcePlay)
SOURCE_WRAPPER(alSourcePause)
SOURCE_WRAPPER(alSourceStop)

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
