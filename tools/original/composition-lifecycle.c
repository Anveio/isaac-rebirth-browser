#define _GNU_SOURCE
#include <errno.h>
#include <poll.h>
#include <signal.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ptrace.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <sys/types.h>
#include <sys/user.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

/* Optional x86-64, leader-thread reference observation. No actor/instruction
 * writes. The real parent pins the canonical executable before launching us.
 * Source owners: Knife vptr66CCF0 virtual48=441BB0, virtual68=448040.
 * 441BC7 retains RDI after the actual active31E/dead31F stores; 448040 is
 * before BaseUninit clears weak links. No pointers survive their event read. */
static const uintptr_t remove_post = 0x441bc7, uninit_entry = 0x448040;
static const int debug_indices[] = {0, 1, 2, 3, 6, 7};
static volatile sig_atomic_t cancelled;
static FILE *journal;
static unsigned long long records, deliveries, events, ignored, gates;
static unsigned long long previous_gate, previous_virtual_ns;
static unsigned stops, listens, continues;
static void cancel(int number) { cancelled = number; }

static int commit(void) {
    if (ftello(journal) > 64 * 1024 * 1024) { errno = EFBIG; return 0; }
    return !ferror(journal) && fflush(journal) == 0 && fsync(fileno(journal)) == 0;
}
static long debug_read(pid_t pid, int index) {
    errno = 0;
    return ptrace(PTRACE_PEEKUSER, pid,
        (void *)(uintptr_t)(offsetof(struct user, u_debugreg) + index * sizeof(unsigned long)), 0);
}
static int debug_write(pid_t pid, int index, unsigned long value) {
    return ptrace(PTRACE_POKEUSER, pid,
        (void *)(uintptr_t)(offsetof(struct user, u_debugreg) + index * sizeof(unsigned long)),
        (void *)(uintptr_t)value) == 0;
}
static int identity(pid_t pid, pid_t owner, unsigned long long ticks,
                    unsigned long long device, unsigned long long inode) {
    char path[80], line[4096]; struct stat executable;
    snprintf(path, sizeof(path), "/proc/%d/stat", pid);
    FILE *stream = fopen(path, "r");
    if (!stream) return 0;
    int ok = fgets(line, sizeof(line), stream) != NULL;
    fclose(stream);
    if (!ok) return 0;
    char *end = strrchr(line, ')'), *save = NULL;
    if (!end || end[1] != ' ') return 0;
    unsigned long long start = 0; long parent = -1;
    char *token = strtok_r(end + 2, " ", &save);
    for (int field = 3; token && field <= 22; field++, token = strtok_r(NULL, " ", &save)) {
        if (field == 4) parent = strtol(token, NULL, 10);
        if (field == 22) start = strtoull(token, NULL, 10);
    }
    snprintf(path, sizeof(path), "/proc/%d/exe", pid);
    return owner == getppid() && parent == owner && start == ticks &&
        stat(path, &executable) == 0 && (unsigned long long)executable.st_dev == device &&
        (unsigned long long)executable.st_ino == inode;
}
/* Source scalar reads only, restricted to current writable mappings. Never
 * PEEKTEXT, never retain a last-seen actor pointer for a later observation. */
static int read_live(pid_t pid, uintptr_t address, void *output, size_t bytes) {
    char path[80], line[512]; unsigned long low, high; char permissions[5];
    size_t rounded = (bytes + sizeof(long) - 1) / sizeof(long) * sizeof(long);
    if (!address || bytes > 64 || address > UINTPTR_MAX - rounded) return 0;
    snprintf(path, sizeof(path), "/proc/%d/maps", pid);
    FILE *maps = fopen(path, "r"); int admitted = 0;
    if (!maps) return 0;
    while (fgets(line, sizeof(line), maps)) {
        if (sscanf(line, "%lx-%lx %4s", &low, &high, permissions) == 3 &&
            permissions[1] == 'w' && low <= address && address + rounded <= high) {
            admitted = 1; break;
        }
    }
    fclose(maps);
    if (!admitted) return 0;
    for (size_t offset = 0; offset < bytes; offset += sizeof(long)) {
        errno = 0;
        long word = ptrace(PTRACE_PEEKDATA, pid, (void *)(address + offset), 0);
        if (errno) return 0;
        size_t count = bytes - offset < sizeof(word) ? bytes - offset : sizeof(word);
        memcpy((char *)output + offset, &word, count);
    }
    return 1;
}
static int event(pid_t pid, const struct user_regs_struct *registers) {
    uintptr_t at = registers->rdi; uint64_t vptr, parent, system;
    uint32_t header[5], seed, counter; unsigned char flags[3], fast;
    if (!read_live(pid, at, &vptr, sizeof(vptr))) return 0;
    if (vptr != 0x66ccf0) { ignored++; return 1; }
    if (!read_live(pid, at + 8, header, sizeof(header))) return 0;
    if (header[2] != 8 || header[3] != 0 || header[4] != 1) { ignored++; return 1; }
    if (!read_live(pid, at + 0x7d0, &seed, sizeof(seed)) ||
        !read_live(pid, at + 0x31d, flags, sizeof(flags)) ||
        !read_live(pid, at + 0x322, &fast, sizeof(fast)) ||
        !read_live(pid, at + 0x498, &parent, sizeof(parent)) ||
        !read_live(pid, 0x8ed560, &system, sizeof(system)) ||
        !read_live(pid, system + 0x2efa0, &counter, sizeof(counter))) return 0;
    if (flags[0] > 1 || flags[1] > 1 || flags[2] > 1 || fast > 1 ||
        (registers->rip == remove_post && (flags[1] != 0 || flags[2] != 1))) return 0;
    /* IDs identify deliveries, not actor+kind. Repeated Remove is legitimate. */
    if (++events > 100000) { errno = EOVERFLOW; return 0; }
    fprintf(journal, "{\"record\":%llu,\"kind\":\"event\",\"eventId\":%llu,\"delivery\":%llu,\"event\":\"%s\",\"sourcePc\":%llu,\"threadId\":%d,\"address\":%llu,\"vptr\":%llu,\"actor\":{\"kind\":8,\"variant\":0,\"subtype\":1,\"sequenceId\":%u,\"spawnSeed\":%u},\"visible\":%s,\"active\":%s,\"dead\":%s,\"fast322\":%s,\"parent498\":%llu,\"systemCounter\":%u,\"previousBlockedGate\":%llu,\"previousVirtualElapsedNs\":%llu}\n",
        ++records, events, deliveries, registers->rip == remove_post ? "remove-post" : "uninit-entry",
        (unsigned long long)registers->rip, pid, (unsigned long long)at, (unsigned long long)vptr,
        header[0], seed, flags[0] ? "true" : "false", flags[1] ? "true" : "false",
        flags[2] ? "true" : "false", fast ? "true" : "false", (unsigned long long)parent,
        counter, previous_gate, previous_virtual_ns);
    return commit();
}
static int wait_stop(pid_t pid, int *status) {
    for (int attempt = 0; attempt < 5000; attempt++) {
        pid_t waited = waitpid(pid, status, WNOHANG);
        if (waited == pid) return WIFSTOPPED(*status);
        if (waited < 0 && errno != EINTR) return 0;
        usleep(1000);
    }
    errno = ETIMEDOUT; return 0;
}
int main(int argc, char **argv) {
    if (argc != 7) return 64;
    umask(0077);
    pid_t pid = (pid_t)strtol(argv[1], NULL, 10), owner = getppid();
    unsigned long long ticks = strtoull(argv[2], NULL, 10), device = strtoull(argv[3], NULL, 10), inode = strtoull(argv[4], NULL, 10);
    unsigned long timeout = strtoul(argv[6], NULL, 10);
    if (pid <= 0 || !ticks || !timeout || timeout > 86400) return 64;
    setvbuf(stdout, NULL, _IONBF, 0);
    signal(SIGTERM, cancel); signal(SIGINT, cancel); signal(SIGPIPE, SIG_IGN);
    journal = fopen(argv[5], "wx");
    if (!journal) return 65;
    int attached = 0, stopped = 0, status = 0, saved_count = 0, restored = 0, detached = 0;
    int pending_signal = 0, detach_requested = 0, success = 0, error = 0, target_exited = 0;
    int pidfd = (int)syscall(SYS_pidfd_open, pid, 0);
    unsigned long saved[6] = {0}; const char *stage = "same real parent, executable and PID lifetime";
    time_t started = time(NULL);
    if (pidfd < 0 || !identity(pid, owner, ticks, device, inode)) goto cleanup;
    stage = "PTRACE_SEIZE leader only";
    if (ptrace(PTRACE_SEIZE, pid, 0, (void *)(uintptr_t)(PTRACE_O_EXITKILL | PTRACE_O_TRACEEXEC))) goto cleanup;
    attached = 1;
    stage = "initial interrupt";
    if (ptrace(PTRACE_INTERRUPT, pid, 0, 0) || !wait_stop(pid, &status) || (unsigned)status >> 16 != PTRACE_EVENT_STOP) goto cleanup;
    stopped = 1;
    stage = "stopped target identity before debug-register installation";
    if (!identity(pid, owner, ticks, device, inode)) goto cleanup;
    for (int i = 0; i < 6; i++) { saved[i] = (unsigned long)debug_read(pid, debug_indices[i]); if (errno) goto cleanup; saved_count++; }
    stage = "exclusive disabled debug slots";
    if (saved[5] & 0xff) { errno = EBUSY; goto cleanup; }
    stage = "install two hardware execution addresses";
    if (!debug_write(pid, 7, 0) || !debug_write(pid, 0, remove_post) || !debug_write(pid, 1, uninit_entry) || !debug_write(pid, 6, 0) || !debug_write(pid, 7, 5)) goto cleanup;
    fprintf(journal, "{\"record\":%llu,\"kind\":\"start\",\"schema\":1,\"threadId\":%d,\"ownerPid\":%d,\"startTicks\":%llu,\"scope\":\"leader-thread8/0/1actual-source-boundaries\",\"removePost\":%llu,\"uninitEntry\":%llu,\"actorMemoryWritten\":false,\"instructionsWritten\":false,\"cpuDebugRegistersTemporarilyWritten\":true}\n", ++records, pid, owner, ticks, (unsigned long long)remove_post, (unsigned long long)uninit_entry);
    if (!commit() || ptrace(PTRACE_CONT, pid, 0, 0)) goto cleanup;
    stopped = 0; puts("ARMED");
    while (!cancelled && (unsigned long)(time(NULL) - started) < timeout) {
        struct pollfd input = {STDIN_FILENO, POLLIN | POLLHUP, 0};
        if (poll(&input, 1, 0) > 0) {
            char line[128];
            stage = "parent command channel";
            if (!fgets(line, sizeof(line), stdin)) goto cleanup;
            unsigned long long gate, ns; char tail;
            if (!strcmp(line, "DETACH\n")) {
                if (detach_requested || ptrace(PTRACE_INTERRUPT, pid, 0, 0)) goto cleanup;
                detach_requested = 1;
            } else if (sscanf(line, "GATE %llu %llu %c", &gate, &ns, &tail) == 2 && !detach_requested) {
                if (gates && (gate < previous_gate || ns < previous_virtual_ns)) goto cleanup;
                previous_gate = gate; previous_virtual_ns = ns; gates++;
                fprintf(journal, "{\"record\":%llu,\"kind\":\"blocked-gate\",\"presentedFrame\":%llu,\"virtualElapsedNs\":%llu}\n", ++records, gate, ns);
                if (!commit()) goto cleanup;
                puts("GATE");
            } else goto cleanup;
        }
        pid_t waited = waitpid(pid, &status, WNOHANG);
        if (waited == 0 || (waited < 0 && errno == EINTR)) { usleep(1000); continue; }
        stage = "actual leader trace stop";
        if (waited == pid && (WIFEXITED(status) || WIFSIGNALED(status))) {
            target_exited = 1; attached = 0; goto cleanup;
        }
        if (waited != pid || !WIFSTOPPED(status)) goto cleanup;
        stopped = 1; pending_signal = 0;
        int sig = WSTOPSIG(status); unsigned ptrace_event = (unsigned)status >> 16;
        if (ptrace_event == PTRACE_EVENT_STOP && detach_requested) {
            /* The interrupt can report SIGSTOP while LISTENing. Detach keeps
             * the existing job-control stop; it does not send SIGCONT. */
            success = identity(pid, owner, ticks, device, inode); break;
        } else if (ptrace_event == PTRACE_EVENT_STOP && sig == SIGSTOP) {
            stage = "preserve actual SIGSTOP group stop";
            if (ptrace(PTRACE_LISTEN, pid, 0, 0)) goto cleanup;
            listens++;
            fprintf(journal, "{\"record\":%llu,\"kind\":\"group-stop-listen\",\"signal\":%d}\n", ++records, sig);
            if (!commit()) goto cleanup;
        } else if (ptrace_event == PTRACE_EVENT_STOP) {
            if (ptrace(PTRACE_CONT, pid, 0, 0)) goto cleanup;
        } else if (ptrace_event) { stage = "unexpected exec or ptrace event"; goto cleanup;
        } else if (sig == SIGTRAP) {
            struct user_regs_struct registers; siginfo_t info;
            stage = "hardware delivery identity";
            if (ptrace(PTRACE_GETREGS, pid, 0, &registers) || ptrace(PTRACE_GETSIGINFO, pid, 0, &info)) goto cleanup;
            if (info.si_code == 4) {
                unsigned long dr6 = (unsigned long)debug_read(pid, 6);
                if (errno || (dr6 & 15) == 0 || (dr6 & 15) > 2 ||
                    ((dr6 & 1) && registers.rip != remove_post) || ((dr6 & 2) && registers.rip != uninit_entry)) goto cleanup;
                if (++deliveries > 1000000 || !event(pid, &registers) || !debug_write(pid, 6, 0)) goto cleanup;
            } else pending_signal = sig;
            if (ptrace(PTRACE_CONT, pid, 0, (void *)(uintptr_t)pending_signal)) goto cleanup;
        } else {
            pending_signal = sig;
            if (sig == SIGSTOP) stops++;
            if (sig == SIGCONT) continues++;
            fprintf(journal, "{\"record\":%llu,\"kind\":\"signal-delivery\",\"signal\":%d}\n", ++records, sig);
            if (!commit()) goto cleanup;
            if (ptrace(PTRACE_CONT, pid, 0, (void *)(uintptr_t)sig)) goto cleanup;
        }
        stopped = 0; pending_signal = 0;
    }
    if (!success) stage = cancelled ? "cancelled" : "host watchdog or target identity";
cleanup:
    error = errno;
    if (attached && !stopped && !target_exited) {
        if (ptrace(PTRACE_INTERRUPT, pid, 0, 0) == 0 && wait_stop(pid, &status)) {
            stopped = 1;
            unsigned event_kind = (unsigned)status >> 16;
            pending_signal = event_kind ? 0 : WSTOPSIG(status);
            if (!event_kind && pending_signal == SIGTRAP) success = 0;
        }
    }
    if (attached && stopped) {
        int ok = 1;
        if (saved_count == 6) {
            ok = debug_write(pid, 7, 0);
            for (int i = 0; i < 5; i++) if (!debug_write(pid, debug_indices[i], saved[i])) ok = 0;
            if (!debug_write(pid, 7, saved[5])) ok = 0;
            for (int i = 0; i < 6; i++) { unsigned long actual = (unsigned long)debug_read(pid, debug_indices[i]); if (errno || actual != saved[i]) ok = 0; }
            restored = ok;
        } else if (saved_count) ok = 0;
        if (ok && ptrace(PTRACE_DETACH, pid, 0, (void *)(uintptr_t)pending_signal) == 0) { detached = 1; attached = 0; }
    }
    /* Never leave a failed owned child running with an armed debug register.
     * EXITKILL also covers abrupt tracer death; no unrelated PID is signalled. */
    if (attached) {
        /* A pidfd pins the actual lifetime even if the real parent reaps it. */
        (void)syscall(SYS_pidfd_send_signal, pidfd, SIGKILL, NULL, 0); success = 0;
    }
    if (!restored || !detached) success = 0;
    fprintf(journal, "{\"record\":%llu,\"kind\":\"finish\",\"status\":\"%s\",\"events\":%llu,\"deliveries\":%llu,\"ignoredOtherActors\":%llu,\"gates\":%llu,\"sigstopForwarded\":%u,\"groupStopListened\":%u,\"sigcontForwarded\":%u,\"debugRegistersRestoredExactly\":%s,\"detached\":%s,\"targetExited\":%s,\"stage\":\"%s\",\"errno\":%d}\n", ++records, success ? "pass" : "fail", events, deliveries, ignored, gates, stops, listens, continues, restored ? "true" : "false", detached ? "true" : "false", target_exited ? "true" : "false", stage, error);
    if (!commit() || fclose(journal)) success = 0;
    if (pidfd >= 0) close(pidfd);
    puts(success ? "CLOSED" : "FAILED");
    return success ? 0 : 1;
}
