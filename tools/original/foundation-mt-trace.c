// Opt-in shared-MT draw observer for the pinned base Rebirth Linux binary
// (sha256 pinned in tools/e2e/lib/original.mjs). foundation-clock-run.py
// compiles and chains it after the clock gate when --mt-trace is given.
//
// The binary's MT19937 draw function 5e4fe0 begins with the 6-byte
// `mov 0x8e6b64(%rip),%eax` index load. Its first byte becomes int3; the
// SIGTRAP handler records the index, Game+ce9f4 (the full-update counter)
// and the in-image return addresses on the stack, then emulates the load
// and resumes. Each record is eight little-endian u64 words:
//   'D', index | counter << 32, up to six return addresses (0 padded).
// glXSwapBuffers appends a presentation marker:
//   'F', index | counter << 32, presentation ordinal, draws so far, 0...
// Records go to $ISAAC_MT_TRACE_DIR/trace-<pid>.bin. The only change to
// game memory is that int3 byte; game state and the drawn values are not
// touched.
#define _GNU_SOURCE
#include <dlfcn.h>
#include <signal.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <ucontext.h>
#include <unistd.h>

#define MT_DRAW 0x5e4fe0UL
#define MT_INDEX 0x8e6b64UL
#define GAME 0x8ecd30UL
#define GAME_COUNTER 0xce9f4UL
#define TEXT_BEGIN 0x408f20UL
#define TEXT_END 0x663e44UL

static FILE *out;
static int active;
static uint64_t draws, presentations;

static uint64_t stamp(void) {
  uint64_t game = *(uint64_t *)GAME;
  uint32_t counter = game ? *(uint32_t *)(game + GAME_COUNTER) : 0xffffffffu;
  return *(uint32_t *)MT_INDEX | ((uint64_t)counter << 32);
}

static void trap(int sig, siginfo_t *info, void *context) {
  (void)info;
  ucontext_t *uc = context;
  greg_t *regs = uc->uc_mcontext.gregs;
  if ((uint64_t)regs[REG_RIP] - 1 != MT_DRAW) {
    signal(sig, SIG_DFL);
    return;
  }
  uint64_t *stack = (uint64_t *)regs[REG_RSP];
  uint64_t record[8] = {'D', stamp(), 0, 0, 0, 0, 0, 0};
  for (int i = 0, n = 2; i < 48 && n < 8; i++)
    if (stack[i] >= TEXT_BEGIN && stack[i] < TEXT_END) record[n++] = stack[i];
  fwrite(record, sizeof record[0], 8, out);
  draws++;
  regs[REG_RAX] = (greg_t)(uint64_t)*(uint32_t *)MT_INDEX;
  regs[REG_RIP] = (greg_t)(MT_DRAW + 6);
}

typedef void (*swap_t)(void *, unsigned long);
void glXSwapBuffers(void *display, unsigned long drawable) {
  static swap_t real;
  if (!real) real = (swap_t)dlsym(RTLD_NEXT, "glXSwapBuffers");
  if (active) {
    uint64_t record[8] = {'F', stamp(), presentations++, draws, 0, 0, 0, 0};
    fwrite(record, sizeof record[0], 8, out);
    fflush(out);
  }
  real(display, drawable);
}

__attribute__((constructor)) static void start(void) {
  char exe[512];
  ssize_t n = readlink("/proc/self/exe", exe, sizeof exe - 1);
  if (n <= 0) return;
  exe[n] = 0;
  const char *directory = getenv("ISAAC_MT_TRACE_DIR");
  if (!strstr(exe, "isaac.x64") || !directory) return;
  char path[600];
  snprintf(path, sizeof path, "%s/trace-%d.bin", directory, (int)getpid());
  if (!(out = fopen(path, "wb"))) return;
  setvbuf(out, NULL, _IOFBF, 1 << 22);
  struct sigaction action;
  memset(&action, 0, sizeof action);
  action.sa_sigaction = trap;
  action.sa_flags = SA_SIGINFO | SA_RESTART;
  sigaction(SIGTRAP, &action, NULL);
  uintptr_t page = MT_DRAW & ~0xfffUL;
  if (*(uint8_t *)MT_DRAW != 0x8b || mprotect((void *)page, 0x2000, PROT_READ | PROT_WRITE | PROT_EXEC)) {
    fprintf(stderr, "mt-trace: unexpected MT draw prologue; is this the pinned binary?\n");
    return;
  }
  *(uint8_t *)MT_DRAW = 0xcc;
  mprotect((void *)page, 0x2000, PROT_READ | PROT_EXEC);
  active = 1;
}

__attribute__((destructor)) static void finish(void) {
  if (out) fflush(out);
}
