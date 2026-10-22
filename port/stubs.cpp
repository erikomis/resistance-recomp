/*
 * ps3recomp game project -- game-specific hooks and overrides.
 *
 * main.cpp is the runner and is game-agnostic. This file is where a port puts
 * everything that is not: the PRX loading hook it must answer, overrides for
 * firmware functions the HLE library gets wrong for this title, and patches
 * over recompiled code.
 *
 * It starts out as the empty version of all of that, which is what a game with
 * no lifted PRX and no overrides needs.
 */

#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <pthread.h>
#include <unistd.h>

#include "ppu_recomp.h"   /* ppu_context, for the context-aware HLE handlers */
#include "ps3emu/ps3types.h"
#include "ps3emu/error_codes.h"

#if defined(__APPLE__)
#include <mach-o/dyld.h>   /* _NSGetExecutablePath */
extern "C" void rsx_metal_backend_export_window_state(void);
#endif
#include <cerrno>
#include <climits>
#include <string>

/* ---------------------------------------------------------------------------
 * PRX loading
 *
 * main.cpp calls this once, after the lifted function table is registered and
 * vm_base is live, before the game runs. A title that lifts a real system PRX
 * -- libsre, which is cellSpurs and cellSync -- loads its image into guest RAM
 * here and registers its exports. Most ports never need to: the HLE library
 * answers those modules instead.
 * -----------------------------------------------------------------------*/

/* RES_PEEK=<secs>:<addr>[,<addr>...]: once <secs> after boot (and every <secs>
 * after that), print 32 bytes of guest memory at each address. An address
 * written as *ADDR is dereferenced first (the 32-bit word at ADDR is the
 * address dumped), which is how the game's runtime-allocated structures --
 * pointers that live in .bss -- are reached without a debugger. */
extern "C" uint8_t* vm_base;

static void res_peek_dump(const char* list)
{
    const char* p = list;
    while (*p) {
        int deref = (*p == '*');
        if (deref) p++;
        char* end;
        uint32_t a = (uint32_t)strtoul(p, &end, 0);
        if (end == p) break;
        p = end;
        if (*p == ',') p++;
        uint32_t ea = a;
        if (deref) {
            const uint8_t* m = vm_base + a;
            ea = ((uint32_t)m[0] << 24) | ((uint32_t)m[1] << 16) | ((uint32_t)m[2] << 8) | m[3];
        }
        const uint8_t* m = vm_base + ea;
        fprintf(stderr, "[peek] %s0x%08X -> 0x%08X:", deref ? "*" : "", a, ea);
        for (int i = 0; i < 32; i++)
            fprintf(stderr, "%s%02X", (i & 3) ? "" : " ", m[i]);
        fprintf(stderr, "\n");
    }
    fflush(stderr);
}

static void* res_peek_thread(void* arg)
{
    const char* spec = (const char*)arg;
    unsigned secs = (unsigned)strtoul(spec, nullptr, 0);
    const char* list = strchr(spec, ':');
    if (!secs || !list) return nullptr;
    for (;;) {
        sleep(secs);
        res_peek_dump(list + 1);
    }
}

/* ---------------------------------------------------------------------------
 * sys_game_process_exitspawn (sysPrxForUser NID 0xFC52A7A9)
 *
 * Resistance ships two executables. EBOOT.BIN is the front end (legal screens,
 * the initial install, the main menu, multiplayer) and the campaign is
 * game.self: picking a campaign difficulty shuts the EBOOT's subsystems down
 * and calls sys_game_process_exitspawn("/dev_bdvd/PS3_GAME/USRDIR/game.self",
 * argv, envp, data, data_size, prio, flags), which on the console replaces the
 * process with the new one. Unresolved, the call returned 0, the EBOOT ran on
 * with everything torn down (stores into the null page) and came back up as
 * the front end again.
 *
 * Each executable is lifted and built on its own, so the spawn is an execv of
 * the other build with the guest's boot path and arguments in PS3_BOOT_PATH /
 * PS3_ARGV (which ppu_run already reads). The data block is written to a file
 * named by RES_SPAWN_DATA for the child; see main.cpp for where it lands.
 *
 * Where the other build lives: RES_SPAWN_BIN / RES_SPAWN_ELF override; by
 * default it is the sibling build directory of this executable
 * (build/BCUS98107 <-> build/BCUS98107_game) and the ELF under
 * game/BCUS98107 of the checkout that build directory sits in.
 * -----------------------------------------------------------------------*/

static std::string res_self_exe(void)
{
    char buf[PATH_MAX] = {0};
#if defined(__APPLE__)
    uint32_t n = sizeof buf;
    if (_NSGetExecutablePath(buf, &n) != 0) return "";
#else
    ssize_t n = readlink("/proc/self/exe", buf, sizeof buf - 1);
    if (n <= 0) return "";
    buf[n] = 0;
#endif
    char real[PATH_MAX];
    return realpath(buf, real) ? std::string(real) : std::string(buf);
}

static std::string res_dirname(const std::string& p)
{
    size_t s = p.find_last_of('/');
    return s == std::string::npos ? std::string(".") : p.substr(0, s);
}

static const char* res_guest_str(uint32_t ea)
{
    return ea ? (const char*)(vm_base + ea) : "";
}

static void hle_sys_game_process_exitspawn(ppu_context* ctx)
{
    const uint32_t path_ea = (uint32_t)ctx->gpr[3];
    const uint32_t argv_ea = (uint32_t)ctx->gpr[4];
    const uint32_t envp_ea = (uint32_t)ctx->gpr[5];
    const uint32_t data_ea = (uint32_t)ctx->gpr[6];
    const uint32_t data_sz = (uint32_t)ctx->gpr[7];
    const std::string path = res_guest_str(path_ea);

    fprintf(stderr, "[exitspawn] path='%s' data=0x%08X size=%u prio=%d flags=0x%llX\n",
            path.c_str(), data_ea, data_sz, (int)ctx->gpr[8],
            (unsigned long long)ctx->gpr[9]);

    /* argv/envp: NULL-terminated arrays of 32-bit guest pointers. */
    std::string args;
    for (uint32_t i = 0; argv_ea && i < 31; i++) {
        const uint8_t* p = vm_base + argv_ea + i * 4;
        uint32_t a = ((uint32_t)p[0] << 24) | ((uint32_t)p[1] << 16) | ((uint32_t)p[2] << 8) | p[3];
        if (!a) break;
        fprintf(stderr, "[exitspawn]   argv[%u] = '%s'\n", i + 1, res_guest_str(a));
        if (!args.empty()) args += ';';
        args += res_guest_str(a);
    }
    for (uint32_t i = 0; envp_ea && i < 31; i++) {
        const uint8_t* p = vm_base + envp_ea + i * 4;
        uint32_t a = ((uint32_t)p[0] << 24) | ((uint32_t)p[1] << 16) | ((uint32_t)p[2] << 8) | p[3];
        if (!a) break;
        fprintf(stderr, "[exitspawn]   envp[%u] = '%s'\n", i, res_guest_str(a));
    }

    std::string data_file;
    if (data_ea && data_sz) {
        fprintf(stderr, "[exitspawn]   data:");
        for (uint32_t i = 0; i < data_sz && i < 64; i++)
            fprintf(stderr, "%s%02X", (i & 3) ? "" : " ", vm_base[data_ea + i]);
        fprintf(stderr, "%s\n", data_sz > 64 ? " ..." : "");
        const char* tmp = getenv("TMPDIR");
        data_file = std::string(tmp && *tmp ? tmp : "/tmp") + "/res_spawn_" +
                    std::to_string(getpid()) + ".bin";
        if (FILE* f = fopen(data_file.c_str(), "wb")) {
            fwrite(vm_base + data_ea, 1, data_sz, f);
            fclose(f);
        } else {
            data_file.clear();
        }
    }

    /* The other executable of the pair. */
    const std::string exe = res_self_exe();
    const std::string build_root = res_dirname(res_dirname(exe));   /* .../build */
    const std::string repo = res_dirname(build_root);
    const std::string base = path.substr(path.find_last_of('/') + 1);
    std::string bin, elf;
    if (base == "game.self") {
        bin = build_root + "/BCUS98107_game/" + exe.substr(exe.find_last_of('/') + 1);
        elf = repo + "/game/BCUS98107/modules/game.elf";
    } else if (base == "EBOOT.BIN") {
        bin = build_root + "/BCUS98107/" + exe.substr(exe.find_last_of('/') + 1);
        elf = repo + "/game/BCUS98107/EBOOT.elf";
    }
    if (const char* e = getenv("RES_SPAWN_BIN")) bin = e;
    if (const char* e = getenv("RES_SPAWN_ELF")) elf = e;
    if (bin.empty() || access(bin.c_str(), X_OK) != 0 || access(elf.c_str(), R_OK) != 0) {
        fprintf(stderr, "[exitspawn] no build for '%s' (bin='%s' elf='%s'): exiting\n",
                path.c_str(), bin.c_str(), elf.c_str());
        fflush(stdout); fflush(stderr);
        _exit(0);
    }

    setenv("PS3_BOOT_PATH", path.c_str(), 1);
    if (args.empty()) unsetenv("PS3_ARGV"); else setenv("PS3_ARGV", args.c_str(), 1);
    if (data_file.empty()) unsetenv("RES_SPAWN_DATA"); else setenv("RES_SPAWN_DATA", data_file.c_str(), 1);
#ifdef __APPLE__
    rsx_metal_backend_export_window_state();   /* the next window opens where this one is */
#endif
    fprintf(stderr, "[exitspawn] exec %s %s\n", bin.c_str(), elf.c_str());
    fflush(stdout); fflush(stderr);
    char* const av[] = { (char*)bin.c_str(), (char*)elf.c_str(), nullptr };
    execv(bin.c_str(), av);
    fprintf(stderr, "[exitspawn] execv failed: %s\n", strerror(errno));
    _exit(1);
}

extern "C" void ps3_hle_register_ctx(uint32_t nid, const char* name, void (*fn)(ppu_context*));

extern "C" void ps3_load_prx_modules(void)
{
    ps3_hle_register_ctx(0xFC52A7A9u, "sys_game_process_exitspawn",
                         hle_sys_game_process_exitspawn);

    /* The HLE libraries log through printf and the runtime through stderr.
     * Redirected to a file, stdout is fully buffered, so a run that is killed
     * (every headless run is) loses its printf lines and the two streams come
     * out of order. Line-buffer it. */
    setvbuf(stdout, nullptr, _IOLBF, 0);

    if (const char* spec = getenv("RES_PEEK")) {
        pthread_t t;
        pthread_create(&t, nullptr, res_peek_thread, (void*)spec);
        pthread_detach(t);
    }
}

/* ---------------------------------------------------------------------------
 * Example: Override a specific NID
 *
 * The module system resolves function imports by NID.  To replace a default
 * HLE stub with custom logic, register your function at init time.
 * -----------------------------------------------------------------------*/

/*
 * Example: override cellGcmSetFlipMode (NID 0xA53D12AE) to force vsync.
 *
 *   static void my_cellGcmSetFlipMode(u32 mode)
 *   {
 *       printf("[hook] cellGcmSetFlipMode: forcing VSYNC (was %u)\n", mode);
 *       // Force vsync regardless of what the game requested
 *       cellGcmSetFlipMode(CELL_GCM_DISPLAY_VSYNC);
 *   }
 *
 * Then in your init code:
 *
 *   ps3::modules::override_nid(0xA53D12AE,
 *       reinterpret_cast<void*>(my_cellGcmSetFlipMode));
 */

/* ---------------------------------------------------------------------------
 * Example: Hook into module loading
 *
 * You can intercept cellSysmoduleLoadModule to perform custom actions
 * when a particular module is loaded.
 * -----------------------------------------------------------------------*/

/*
 *   static s32 my_loadmodule_hook(u16 id)
 *   {
 *       printf("[hook] Module 0x%04X loaded\n", id);
 *
 *       if (id == CELL_SYSMODULE_AUDIO) {
 *           // Perform custom audio backend initialization
 *           init_my_audio_backend();
 *       }
 *
 *       // Call the original implementation
 *       return cellSysmoduleLoadModule(id);
 *   }
 */

/* ---------------------------------------------------------------------------
 * Game-specific patches
 *
 * Sometimes you need to patch recompiled code at specific addresses.
 * Use the patch system to modify behavior without editing generated code.
 * -----------------------------------------------------------------------*/

/*
 *   // Skip an integrity check at guest address 0x00812FA0
 *   ps3::patches::nop_range(0x00812FA0, 0x00812FB0);
 *
 *   // Replace a function call with a direct return
 *   ps3::patches::force_return(0x009A1000, 0);  // return 0
 */
