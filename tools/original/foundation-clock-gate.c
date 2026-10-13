/* Public Linux time/X11 presentation adapter. No original program memory,
 * machine instructions or hidden state are inspected. Main-thread wall clocks
 * advance only when the controller releases a presented frame. Worker threads
 * and CPU clocks retain the real host clocks. Presented frames are not proof of
 * the original simulation tick count. */
#define _GNU_SOURCE
#include <dlfcn.h>
#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/syscall.h>
#include <sys/time.h>
#include <sys/types.h>
#include <sys/un.h>
#include <time.h>
#include <unistd.h>

_Static_assert(sizeof(void *) == 8 && sizeof(unsigned long) == 8, "Native Rebirth and its public X11 ABI require a 64-bit host");
static pid_t owner;
static int enabled, listener = -1, peer = -1;
static unsigned fps = 60;
static uint64_t frame, pending, elapsed_ns;
static uint64_t step_ns = UINT64_C(16667000);
static uint64_t clock_calls, realtime_calls, sleep_calls;
struct InputEvent { int type; unsigned long keysym; uint64_t frame, elapsed; };
static struct InputEvent input_events[2048];
static unsigned input_event_count;
static int input_overflow;
static const uint64_t monotonic_base = UINT64_C(1000000000000);
static uint64_t realtime_base = UINT64_C(1700000000000000000);
static int main_thread(void) { return enabled && (pid_t)syscall(SYS_gettid) == owner; }

__attribute__((constructor)) static void start_gate(void) {
    const char *path = getenv("ISAAC_CLOCK_GATE_SOCKET");
    if (!path || !*path) return;
    const char *epoch = getenv("ISAAC_CLOCK_GATE_EPOCH_SEED");
    if (epoch) {
        char *end = NULL;
        unsigned long long requested = strtoull(epoch, &end, 10);
        if (!*epoch || *end || requested > UINT32_MAX) {
            fprintf(stderr, "clock gate epoch seed must be a u32\n"); _exit(90);
        }
        realtime_base = (uint64_t)requested * UINT64_C(1000000000);
    }
    owner = (pid_t)syscall(SYS_gettid);
    const char *rate = getenv("ISAAC_CLOCK_GATE_FPS");
    if (rate) { unsigned requested = (unsigned)strtoul(rate, NULL, 10); if (requested >= 1 && requested <= 1000) fps = requested; }
    const char *interval = getenv("ISAAC_CLOCK_GATE_STEP_NS");
    if (interval) step_ns = strtoull(interval, NULL, 10);
    struct sockaddr_un addr = { .sun_family = AF_UNIX };
    if (strlen(path) >= sizeof(addr.sun_path)) { fprintf(stderr, "clock gate socket path too long\n"); _exit(90); }
    strcpy(addr.sun_path, path);
    listener = socket(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC, 0);
    if (listener < 0 || bind(listener, (struct sockaddr *)&addr, sizeof(addr)) || listen(listener, 1)) { perror("clock gate listen"); _exit(91); }
    enabled = 1;
}

static int virtual_clock(clockid_t id, struct timespec *value) {
    if (!main_thread()) return 0;
    uint64_t now;
    switch (id) {
        case CLOCK_REALTIME: case CLOCK_REALTIME_COARSE: now = realtime_base + elapsed_ns; realtime_calls++; break;
        case CLOCK_MONOTONIC: case CLOCK_MONOTONIC_COARSE: case CLOCK_MONOTONIC_RAW: now = monotonic_base + elapsed_ns; clock_calls++; break;
        default: return 0;
    }
    value->tv_sec = (time_t)(now / UINT64_C(1000000000));
    value->tv_nsec = (long)(now % UINT64_C(1000000000));
    return 1;
}
int clock_gettime(clockid_t id, struct timespec *value) {
    if (virtual_clock(id, value)) return 0;
    int (*real)(clockid_t,struct timespec *) = dlsym(RTLD_NEXT,"clock_gettime");
    return real(id,value);
}
int gettimeofday(struct timeval *value, void *zone) {
    if (main_thread()) {
        uint64_t now = realtime_base + elapsed_ns; realtime_calls++;
        value->tv_sec = (time_t)(now / UINT64_C(1000000000)); value->tv_usec = (suseconds_t)((now % UINT64_C(1000000000)) / 1000);
        if (zone) memset(zone, 0, sizeof(struct timezone));
        return 0;
    }
    int (*real)(struct timeval *,void *) = dlsym(RTLD_NEXT,"gettimeofday");
    return real(value,zone);
}
time_t time(time_t *value) {
    if (main_thread()) { time_t now = (time_t)((realtime_base + elapsed_ns) / UINT64_C(1000000000)); if (value) *value = now; realtime_calls++; return now; }
    time_t (*real)(time_t *) = dlsym(RTLD_NEXT,"time"); return real(value);
}
int nanosleep(const struct timespec *requested, struct timespec *remaining) {
    if (main_thread()) {
        if (!requested) { errno=EFAULT;return -1; }
        if (requested->tv_sec < 0 || requested->tv_nsec < 0 || requested->tv_nsec >= 1000000000) { errno=EINVAL; return -1; }
        sleep_calls++; if (remaining) memset(remaining,0,sizeof(*remaining)); return 0;
    }
    int (*real)(const struct timespec *,struct timespec *) = dlsym(RTLD_NEXT,"nanosleep"); return real(requested,remaining);
}
int usleep(useconds_t duration) {
    if (main_thread()) { (void)duration; sleep_calls++; return 0; }
    int (*real)(useconds_t) = dlsym(RTLD_NEXT,"usleep"); return real(duration);
}
static int send_all(const char *bytes, size_t size) {
    while (size) { ssize_t n = send(peer,bytes,size,MSG_NOSIGNAL); if (n<0 && errno==EINTR) continue; if(n<=0)return 0; bytes+=n;size-=(size_t)n; } return 1;
}
static int command(char *line, size_t capacity) {
    size_t used=0;
    while (used+1<capacity) { char c;ssize_t n=read(peer,&c,1);if(n<0&&errno==EINTR)continue;if(n<=0)return 0;if(c=='\n'){line[used]=0;return 1;}line[used++]=c; }
    return 0;
}
static void advance(void) { frame++; elapsed_ns = step_ns ? frame*step_ns : (frame / fps)*UINT64_C(1000000000) + ((frame % fps)*UINT64_C(1000000000))/fps; }
static void observed_event(void *event) {
    if (!main_thread() || !event) return;
    int type; memcpy(&type,event,sizeof(type));
    if (type != 2 && type != 3 && type != 9 && type != 10) return; /* Public X11 key and focus event types. */
    unsigned long (*lookup)(void *,int)=dlsym(RTLD_NEXT,"XLookupKeysym");
    if(input_event_count>=2048) { input_overflow=1;return; }
    input_events[input_event_count++]=(struct InputEvent){type,(type==2||type==3)&&lookup?lookup(event,0):0,frame,elapsed_ns};
}
int XNextEvent(void *display,void *event) {
    int (*real)(void *,void *)=dlsym(RTLD_NEXT,"XNextEvent");
    int result=real(display,event);observed_event(event);return result;
}
int XCheckTypedEvent(void *display,int type,void *event) {
    int (*real)(void *,int,void *)=dlsym(RTLD_NEXT,"XCheckTypedEvent");
    int result=real(display,type,event);if(result)observed_event(event);return result;
}
int XCheckIfEvent(void *display,void *event,int (*predicate)(void *,void *,char *),char *argument) {
    int (*real)(void *,void *,int (*)(void *,void *,char *),char *)=dlsym(RTLD_NEXT,"XCheckIfEvent");
    int result=real(display,event,predicate,argument);if(result)observed_event(event);return result;
}
void glXSwapBuffers(void *display, unsigned long drawable) {
    void (*real)(void *,unsigned long) = dlsym(RTLD_NEXT,"glXSwapBuffers");
    real(display,drawable);
    if (!main_thread()) return;
    if (pending > 1) { pending--; advance(); return; }
    pending=0;
    if (peer<0) { do { peer=accept4(listener,NULL,NULL,SOCK_CLOEXEC); } while(peer<0&&errno==EINTR); if(peer<0)_exit(92); }
    char line[512];
    int size=snprintf(line,sizeof(line),"{\"schema\":1,\"presentedFrame\":%llu,\"virtualElapsedNs\":%llu,\"virtualFps\":%u,\"monotonicCalls\":%llu,\"realtimeCalls\":%llu,\"sleepCalls\":%llu,\"simulationTick\":null,\"fullInternalState\":false}\n",(unsigned long long)frame,(unsigned long long)elapsed_ns,fps,(unsigned long long)clock_calls,(unsigned long long)realtime_calls,(unsigned long long)sleep_calls);
    if(size<2)_exit(93);
    line[size-2]=0;
    if(!send_all(line,(size_t)size-2))_exit(93);
    size=snprintf(line,sizeof(line),",\"inputOverflow\":%s,\"consumedX11Events\":[",input_overflow?"true":"false");
    if(!send_all(line,(size_t)size))_exit(93);
    unsigned emitted=0;
    for(unsigned i=0;i<input_event_count;i++) {
        struct InputEvent event=input_events[i];
        if(event.type!=2 && event.type!=3)continue;
        size=snprintf(line,sizeof(line),"%s{\"down\":%s,\"keysym\":%lu,\"presentedFramePhase\":%llu,\"virtualElapsedNs\":%llu}",emitted++?",":"",event.type==2?"true":"false",event.keysym,(unsigned long long)event.frame,(unsigned long long)event.elapsed);
        if(!send_all(line,(size_t)size))_exit(93);
    }
    const char *focus_start="],\"consumedX11FocusEvents\":[";
    if(!send_all(focus_start,strlen(focus_start)))_exit(93);
    emitted=0;
    for(unsigned i=0;i<input_event_count;i++) {
        struct InputEvent event=input_events[i];
        if(event.type!=9 && event.type!=10)continue;
        size=snprintf(line,sizeof(line),"%s{\"focused\":%s,\"presentedFramePhase\":%llu,\"virtualElapsedNs\":%llu}",emitted++?",":"",event.type==9?"true":"false",(unsigned long long)event.frame,(unsigned long long)event.elapsed);
        if(!send_all(line,(size_t)size))_exit(93);
    }
    input_event_count=0;
    if(!send_all("]}\n",3))_exit(93);
    for (;;) {
        if(!command(line,sizeof(line)))_exit(94);
        unsigned long long count=0; char extra;
        if(sscanf(line,"step %llu %c",&count,&extra)==1 && count>=1 && count<=36000) { pending=count;advance();return; }
        if(!strcmp(line,"quit"))_exit(0);
        const char *error="{\"error\":\"expected step 1..36000 or quit\"}\n";
        if(!send_all(error,strlen(error)))_exit(95);
    }
}
