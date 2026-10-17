/* Public Linux time/X11 presentation adapter. No original program memory,
 * machine instructions or hidden state are inspected. Main-thread wall clocks
 * advance only when the controller releases a presented frame. Worker threads
 * and CPU clocks retain the real host clocks. Presented frames are not proof of
 * the original simulation tick count.
 *
 * Released presentations. A presentation the controller releases inside a
 * `step N` (every one but the last) is never acknowledged, captured or
 * observed: the controller reads pixels and memory only while the main thread
 * is blocked here. For those frames the adapter drops glDrawElements and
 * glDrawRangeElements aimed at the default framebuffer, and the real
 * glXSwapBuffers. Draws into framebuffer objects, clears, texture uploads,
 * readbacks and every game instruction still run, so the game's state, its GL
 * objects and the virtual clock advance exactly as before. The blocking
 * presentation (the one that acknowledges) always draws and presents
 * unchanged. Dropping is unobservable because the original begins the default
 * framebuffer of every frame with an unscissored colour+depth glClear (plus
 * stencil once stencil testing is on), so a blocking frame never inherits a
 * dropped draw. The adapter checks that law on every frame: a dropped draw
 * leaves the default framebuffer stale until a full clear (colour, depth and
 * stencil once those tests are on, scissor off, write masks full) erases it.
 * Presenting it or reading it back (glReadPixels, glReadnPixels, glCopyPixels,
 * glCopyTex*, glBlitFramebuffer) while stale, or dropping a draw inside a
 * query or transform feedback, exits 96 with a stderr reason instead of
 * degrading silently. Other draw entry points are never dropped, and indexed
 * masks or capabilities make later clears count as partial. Setting
 * ISAAC_CLOCK_GATE_RENDER_ALL=1 draws and presents every frame (diagnosis, and
 * graphics observers that record submitted draws).
 *
 * The original asks the X server for its window attributes dozens of times a
 * frame. Within one main-thread frame, with no X event consumed and no window
 * request made, the server-side window cannot change (the controller acts only
 * while this adapter blocks), so XGetWindowAttributes is answered from the
 * frame's first reply. ISAAC_CLOCK_GATE_WINDOW_QUERIES=1 forwards every call. */
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
static int main_thread(void) { static __thread pid_t self; if (!self) self = (pid_t)syscall(SYS_gettid); return enabled && self == owner; }
static int render_all, window_queries;
static int flag(const char *name) { const char *value = getenv(name); return value && *value && strcmp(value, "0"); }

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
    render_all = flag("ISAAC_CLOCK_GATE_RENDER_ALL");
    window_queries = flag("ISAAC_CLOCK_GATE_WINDOW_QUERIES");
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
static void window_changed(void);
static void observed_event(void *event) {
    if (!main_thread() || !event) return;
    window_changed();
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
/* Window attribute replies, valid for one main-thread frame (see header). */
static unsigned char window_reply[136]; /* sizeof(XWindowAttributes) on the LP64 Xlib ABI */
static void *window_display; static unsigned long window_id; static int window_status, window_cached;
static void window_changed(void) { window_cached = 0; }
int XGetWindowAttributes(void *display, unsigned long window, void *attributes) {
    static int (*real)(void *,unsigned long,void *); if (!real) real = dlsym(RTLD_NEXT,"XGetWindowAttributes");
    int cache = main_thread() && !window_queries;
    if (cache && window_cached && display == window_display && window == window_id) { memcpy(attributes,window_reply,sizeof(window_reply)); return window_status; }
    int status = real(display,window,attributes);
    if (cache && status) { memcpy(window_reply,attributes,sizeof(window_reply)); window_display=display; window_id=window; window_status=status; window_cached=1; }
    return status;
}
/* Every window request the original imports drops the frame's reply. */
#define WINDOW_REQUEST(type, name, params, args) type name params { \
    static type (*real) params; if (!real) real = dlsym(RTLD_NEXT,#name); window_changed(); return real args; }
WINDOW_REQUEST(int, XMoveWindow, (void *d, unsigned long w, int x, int y), (d,w,x,y))
WINDOW_REQUEST(int, XResizeWindow, (void *d, unsigned long w, unsigned width, unsigned height), (d,w,width,height))
WINDOW_REQUEST(int, XMapWindow, (void *d, unsigned long w), (d,w))
WINDOW_REQUEST(int, XMapRaised, (void *d, unsigned long w), (d,w))
WINDOW_REQUEST(int, XUnmapWindow, (void *d, unsigned long w), (d,w))
WINDOW_REQUEST(int, XRaiseWindow, (void *d, unsigned long w), (d,w))
WINDOW_REQUEST(int, XDestroyWindow, (void *d, unsigned long w), (d,w))
WINDOW_REQUEST(int, XIconifyWindow, (void *d, unsigned long w, int screen), (d,w,screen))
WINDOW_REQUEST(int, XChangeWindowAttributes, (void *d, unsigned long w, unsigned long mask, void *values), (d,w,mask,values))
WINDOW_REQUEST(int, XChangeProperty, (void *d, unsigned long w, unsigned long property, unsigned long type, int format, int mode, const unsigned char *data, int count), (d,w,property,type,format,mode,data,count))
WINDOW_REQUEST(int, XSendEvent, (void *d, unsigned long w, int propagate, long mask, void *event), (d,w,propagate,mask,event))
WINDOW_REQUEST(unsigned long, XCreateWindow, (void *d, unsigned long parent, int x, int y, unsigned width, unsigned height, unsigned border, int depth, unsigned kind, void *visual, unsigned long mask, void *values), (d,parent,x,y,width,height,border,depth,kind,visual,mask,values))
WINDOW_REQUEST(int, XRRSetCrtcConfig, (void *d, void *resources, unsigned long crtc, unsigned long timestamp, int x, int y, unsigned long mode, unsigned short rotation, unsigned long *outputs, int count), (d,resources,crtc,timestamp,x,y,mode,rotation,outputs,count))

/* Released-presentation drawing (see header). The GL context is the main
 * thread's; other threads and ISAAC_CLOCK_GATE_RENDER_ALL forward unchanged. */
typedef unsigned GLenum, GLuint, GLbitfield; typedef int GLint, GLsizei; typedef unsigned char GLboolean;
typedef void (*GLproc)(void);
static GLuint draw_fb, read_fb;
static int scissor, depth_test_used, stencil_test_used, color_masked, depth_masked, stencil_masked, indexed_state;
static int stale; /* a dropped default-framebuffer draw no full clear has erased yet */
static unsigned active_records; /* queries and transform feedback a dropped draw would change */
static int dropping(void) { return !render_all && main_thread(); }
static int released(void) { return pending > 1; }
static void violation(const char *reason) {
    fprintf(stderr, "clock gate: released-presentation law broken after presented frame %llu: %s; rerun with ISAAC_CLOCK_GATE_RENDER_ALL=1\n", (unsigned long long)frame, reason);
    _exit(96);
}
static int drop_draw(void) {
    if (!dropping() || draw_fb != 0 || !released()) return 0;
    if (active_records) violation("a default-framebuffer draw inside a query or transform feedback");
    stale = 1; return 1;
}
static void default_read(const char *api) { if (dropping() && read_fb == 0 && stale) violation(api); }
#define GL_REAL(name, params) static void (*real) params; if (!real) real = (void (*) params)dlsym(RTLD_NEXT,#name)
void glDrawElements(GLenum mode, GLsizei count, GLenum type, const void *indices) {
    GL_REAL(glDrawElements, (GLenum,GLsizei,GLenum,const void *));
    if (!drop_draw()) real(mode,count,type,indices);
}
void glClear(GLbitfield mask) {
    GL_REAL(glClear, (GLbitfield));
    /* Colour, depth once depth testing is on and stencil once stencil testing is on. */
    if (dropping() && draw_fb == 0 && (mask & 0x4000u) && (!depth_test_used || (mask & 0x100u)) && (!stencil_test_used || (mask & 0x400u))
        && !scissor && !color_masked && !depth_masked && !(stencil_test_used && stencil_masked) && !indexed_state) stale = 0;
    real(mask);
}
static void capability(GLenum cap, int on) {
    if (!main_thread()) return;
    if (cap == 0x0C11u) scissor = on;
    if (on && cap == 0x0B71u) depth_test_used = 1;
    if (on && cap == 0x0B90u) stencil_test_used = 1;
}
void glEnable(GLenum cap) { GL_REAL(glEnable, (GLenum)); capability(cap,1); real(cap); }
void glDisable(GLenum cap) { GL_REAL(glDisable, (GLenum)); capability(cap,0); real(cap); }
void glColorMask(GLboolean r, GLboolean g, GLboolean b, GLboolean a) {
    GL_REAL(glColorMask, (GLboolean,GLboolean,GLboolean,GLboolean)); if (main_thread()) color_masked = !(r && g && b && a); real(r,g,b,a);
}
void glDepthMask(GLboolean flag) { GL_REAL(glDepthMask, (GLboolean)); if (main_thread()) depth_masked = !flag; real(flag); }
void glStencilMask(GLuint mask) { GL_REAL(glStencilMask, (GLuint)); if (main_thread()) stencil_masked = (mask & 0xffu) != 0xffu; real(mask); }
void glReadPixels(GLint x, GLint y, GLsizei width, GLsizei height, GLenum format, GLenum type, void *pixels) {
    GL_REAL(glReadPixels, (GLint,GLint,GLsizei,GLsizei,GLenum,GLenum,void *)); default_read("glReadPixels from the default framebuffer"); real(x,y,width,height,format,type,pixels);
}
void glCopyPixels(GLint x, GLint y, GLsizei width, GLsizei height, GLenum type) {
    GL_REAL(glCopyPixels, (GLint,GLint,GLsizei,GLsizei,GLenum)); default_read("glCopyPixels from the default framebuffer"); real(x,y,width,height,type);
}
void glCopyTexImage2D(GLenum target, GLint level, GLenum format, GLint x, GLint y, GLsizei width, GLsizei height, GLint border) {
    GL_REAL(glCopyTexImage2D, (GLenum,GLint,GLenum,GLint,GLint,GLsizei,GLsizei,GLint)); default_read("glCopyTexImage2D from the default framebuffer"); real(target,level,format,x,y,width,height,border);
}
void glCopyTexSubImage2D(GLenum target, GLint level, GLint xoffset, GLint yoffset, GLint x, GLint y, GLsizei width, GLsizei height) {
    GL_REAL(glCopyTexSubImage2D, (GLenum,GLint,GLint,GLint,GLint,GLint,GLsizei,GLsizei)); default_read("glCopyTexSubImage2D from the default framebuffer"); real(target,level,xoffset,yoffset,x,y,width,height);
}
/* Extension entry points reach the original through glXGetProcAddress. */
static GLproc next_bind_fb, next_bind_fb_ext, next_delete_fb, next_delete_fb_ext, next_range, next_range_ext, next_copy3d, next_copy3d_ext,
    next_blit, next_blit_ext, next_readn, next_readn_arb, next_color_maski, next_color_maski_ext, next_enablei, next_disablei,
    next_begin_query, next_begin_query_arb, next_end_query, next_end_query_arb, next_begin_indexed, next_end_indexed,
    next_begin_feedback, next_begin_feedback_ext, next_end_feedback, next_end_feedback_ext;
static void bind_fb(GLproc next, GLenum target, GLuint id) {
    if (main_thread()) { if (target != 0x8CA8u) draw_fb = id; if (target != 0x8CA9u) read_fb = id; }
    ((void (*)(GLenum,GLuint))next)(target,id);
}
static void bind_fb_core(GLenum target, GLuint id) { bind_fb(next_bind_fb,target,id); }
static void bind_fb_ext(GLenum target, GLuint id) { bind_fb(next_bind_fb_ext,target,id); }
static void delete_fb(GLproc next, GLsizei count, const GLuint *ids) {
    /* Deleting a bound framebuffer object rebinds the default framebuffer. */
    for (GLsizei i = 0; main_thread() && ids && i < count; i++) { if (ids[i] && ids[i] == draw_fb) draw_fb = 0; if (ids[i] && ids[i] == read_fb) read_fb = 0; }
    ((void (*)(GLsizei,const GLuint *))next)(count,ids);
}
static void delete_fb_core(GLsizei count, const GLuint *ids) { delete_fb(next_delete_fb,count,ids); }
static void delete_fb_ext(GLsizei count, const GLuint *ids) { delete_fb(next_delete_fb_ext,count,ids); }
static void range(GLproc next, GLenum mode, GLuint start, GLuint end, GLsizei count, GLenum type, const void *indices) {
    if (!drop_draw()) ((void (*)(GLenum,GLuint,GLuint,GLsizei,GLenum,const void *))next)(mode,start,end,count,type,indices);
}
static void range_core(GLenum mode, GLuint start, GLuint end, GLsizei count, GLenum type, const void *indices) { range(next_range,mode,start,end,count,type,indices); }
static void range_ext(GLenum mode, GLuint start, GLuint end, GLsizei count, GLenum type, const void *indices) { range(next_range_ext,mode,start,end,count,type,indices); }
static void copy3d(GLproc next, GLenum target, GLint level, GLint xo, GLint yo, GLint zo, GLint x, GLint y, GLsizei width, GLsizei height) {
    default_read("glCopyTexSubImage3D from the default framebuffer");
    ((void (*)(GLenum,GLint,GLint,GLint,GLint,GLint,GLint,GLsizei,GLsizei))next)(target,level,xo,yo,zo,x,y,width,height);
}
static void copy3d_core(GLenum t, GLint l, GLint xo, GLint yo, GLint zo, GLint x, GLint y, GLsizei w, GLsizei h) { copy3d(next_copy3d,t,l,xo,yo,zo,x,y,w,h); }
static void copy3d_ext(GLenum t, GLint l, GLint xo, GLint yo, GLint zo, GLint x, GLint y, GLsizei w, GLsizei h) { copy3d(next_copy3d_ext,t,l,xo,yo,zo,x,y,w,h); }
static void blit(GLproc next, GLint sx0, GLint sy0, GLint sx1, GLint sy1, GLint dx0, GLint dy0, GLint dx1, GLint dy1, GLbitfield mask, GLenum filter) {
    default_read("glBlitFramebuffer from the default framebuffer");
    ((void (*)(GLint,GLint,GLint,GLint,GLint,GLint,GLint,GLint,GLbitfield,GLenum))next)(sx0,sy0,sx1,sy1,dx0,dy0,dx1,dy1,mask,filter);
}
static void blit_core(GLint a, GLint b, GLint c, GLint d, GLint e, GLint f, GLint g, GLint h, GLbitfield m, GLenum fl) { blit(next_blit,a,b,c,d,e,f,g,h,m,fl); }
static void blit_ext(GLint a, GLint b, GLint c, GLint d, GLint e, GLint f, GLint g, GLint h, GLbitfield m, GLenum fl) { blit(next_blit_ext,a,b,c,d,e,f,g,h,m,fl); }
static void readn(GLproc next, GLint x, GLint y, GLsizei width, GLsizei height, GLenum format, GLenum type, GLsizei size, void *data) {
    default_read("glReadnPixels from the default framebuffer");
    ((void (*)(GLint,GLint,GLsizei,GLsizei,GLenum,GLenum,GLsizei,void *))next)(x,y,width,height,format,type,size,data);
}
static void readn_core(GLint x, GLint y, GLsizei w, GLsizei h, GLenum f, GLenum t, GLsizei s, void *d) { readn(next_readn,x,y,w,h,f,t,s,d); }
static void readn_arb(GLint x, GLint y, GLsizei w, GLsizei h, GLenum f, GLenum t, GLsizei s, void *d) { readn(next_readn_arb,x,y,w,h,f,t,s,d); }
/* Indexed colour masks and capabilities are not modelled: a clear after one
 * no longer counts as full, so only a later present or readback can fail. */
static void color_maski(GLproc next, GLuint index, GLboolean r, GLboolean g, GLboolean b, GLboolean a) {
    if (main_thread()) indexed_state = 1;
    ((void (*)(GLuint,GLboolean,GLboolean,GLboolean,GLboolean))next)(index,r,g,b,a);
}
static void color_maski_core(GLuint i, GLboolean r, GLboolean g, GLboolean b, GLboolean a) { color_maski(next_color_maski,i,r,g,b,a); }
static void color_maski_ext(GLuint i, GLboolean r, GLboolean g, GLboolean b, GLboolean a) { color_maski(next_color_maski_ext,i,r,g,b,a); }
static void enablei(GLenum cap, GLuint index) { if (main_thread()) indexed_state = 1; ((void (*)(GLenum,GLuint))next_enablei)(cap,index); }
static void disablei(GLenum cap, GLuint index) { if (main_thread()) indexed_state = 1; ((void (*)(GLenum,GLuint))next_disablei)(cap,index); }
static void record(GLproc next, int begin, GLenum target, GLuint id) {
    if (main_thread()) { if (begin) active_records++; else if (active_records) active_records--; }
    ((void (*)(GLenum,GLuint))next)(target,id);
}
static void begin_query(GLenum target, GLuint id) { record(next_begin_query,1,target,id); }
static void begin_query_arb(GLenum target, GLuint id) { record(next_begin_query_arb,1,target,id); }
static void end_query(GLenum target) { record(next_end_query,0,target,0); }
static void end_query_arb(GLenum target) { record(next_end_query_arb,0,target,0); }
static void begin_indexed(GLenum target, GLuint index, GLuint id) {
    if (main_thread()) active_records++;
    ((void (*)(GLenum,GLuint,GLuint))next_begin_indexed)(target,index,id);
}
static void end_indexed(GLenum target, GLuint index) { if (main_thread() && active_records) active_records--; ((void (*)(GLenum,GLuint))next_end_indexed)(target,index); }
static void begin_feedback(GLenum mode) { if (main_thread()) active_records++; ((void (*)(GLenum))next_begin_feedback)(mode); }
static void begin_feedback_ext(GLenum mode) { if (main_thread()) active_records++; ((void (*)(GLenum))next_begin_feedback_ext)(mode); }
static void end_feedback(void) { if (main_thread() && active_records) active_records--; ((void (*)(void))next_end_feedback)(); }
static void end_feedback_ext(void) { if (main_thread() && active_records) active_records--; ((void (*)(void))next_end_feedback_ext)(); }
static GLproc procedure(const unsigned char *name, GLproc next) {
    if (!next || render_all || !enabled) return next;
    static const struct { const char *name; GLproc *next; GLproc wrapper; } table[] = {
        {"glBindFramebuffer",&next_bind_fb,(GLproc)bind_fb_core}, {"glBindFramebufferEXT",&next_bind_fb_ext,(GLproc)bind_fb_ext},
        {"glDeleteFramebuffers",&next_delete_fb,(GLproc)delete_fb_core}, {"glDeleteFramebuffersEXT",&next_delete_fb_ext,(GLproc)delete_fb_ext},
        {"glDrawRangeElements",&next_range,(GLproc)range_core}, {"glDrawRangeElementsEXT",&next_range_ext,(GLproc)range_ext},
        {"glCopyTexSubImage3D",&next_copy3d,(GLproc)copy3d_core}, {"glCopyTexSubImage3DEXT",&next_copy3d_ext,(GLproc)copy3d_ext},
        {"glBlitFramebuffer",&next_blit,(GLproc)blit_core}, {"glBlitFramebufferEXT",&next_blit_ext,(GLproc)blit_ext},
        {"glReadnPixels",&next_readn,(GLproc)readn_core}, {"glReadnPixelsARB",&next_readn_arb,(GLproc)readn_arb},
        {"glColorMaski",&next_color_maski,(GLproc)color_maski_core}, {"glColorMaskIndexedEXT",&next_color_maski_ext,(GLproc)color_maski_ext},
        {"glEnablei",&next_enablei,(GLproc)enablei}, {"glDisablei",&next_disablei,(GLproc)disablei},
        {"glBeginQuery",&next_begin_query,(GLproc)begin_query}, {"glBeginQueryARB",&next_begin_query_arb,(GLproc)begin_query_arb},
        {"glEndQuery",&next_end_query,(GLproc)end_query}, {"glEndQueryARB",&next_end_query_arb,(GLproc)end_query_arb},
        {"glBeginQueryIndexed",&next_begin_indexed,(GLproc)begin_indexed}, {"glEndQueryIndexed",&next_end_indexed,(GLproc)end_indexed},
        {"glBeginTransformFeedback",&next_begin_feedback,(GLproc)begin_feedback}, {"glBeginTransformFeedbackEXT",&next_begin_feedback_ext,(GLproc)begin_feedback_ext},
        {"glEndTransformFeedback",&next_end_feedback,(GLproc)end_feedback}, {"glEndTransformFeedbackEXT",&next_end_feedback_ext,(GLproc)end_feedback_ext},
    };
    for (size_t i = 0; i < sizeof(table)/sizeof(table[0]); i++)
        if (!strcmp((const char *)name,table[i].name)) { *table[i].next = next; return table[i].wrapper; }
    return next;
}
GLproc glXGetProcAddress(const unsigned char *name) {
    static GLproc (*real)(const unsigned char *); if (!real) real = (GLproc (*)(const unsigned char *))dlsym(RTLD_NEXT,"glXGetProcAddress");
    return procedure(name,real(name));
}
GLproc glXGetProcAddressARB(const unsigned char *name) {
    static GLproc (*real)(const unsigned char *); if (!real) real = (GLproc (*)(const unsigned char *))dlsym(RTLD_NEXT,"glXGetProcAddressARB");
    return procedure(name,real(name));
}
/* A new current context starts on its default framebuffer. */
int glXMakeCurrent(void *display, unsigned long drawable, void *context) {
    static int (*real)(void *,unsigned long,void *); if (!real) real = dlsym(RTLD_NEXT,"glXMakeCurrent");
    if (main_thread()) draw_fb = read_fb = 0;
    return real(display,drawable,context);
}
void glXSwapBuffers(void *display, unsigned long drawable) {
    void (*real)(void *,unsigned long) = dlsym(RTLD_NEXT,"glXSwapBuffers");
    window_changed();
    if (dropping() && released()) {
        /* Released: nobody reads this presentation; the next frame clears. */
    } else {
        if (dropping() && stale) violation("presenting a default framebuffer not fully cleared since a dropped draw");
        real(display,drawable);
    }
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
        if(sscanf(line,"step %llu %c",&count,&extra)==1 && count>=1 && count<=36000) { pending=count;advance();window_changed();return; }
        if(!strcmp(line,"quit"))_exit(0);
        const char *error="{\"error\":\"expected step 1..36000 or quit\"}\n";
        if(!send_all(error,strlen(error)))_exit(95);
    }
}
