/*
 * Optional game patches: unlocked frame rate and a wider field of view.
 *
 * They are the community patches RPCS3 ships in its patch.yml for this game
 * ("Unlock FPS" and "Extended FOV" for game.self, by Whatcookie, FlexBy,
 * illusion and Score; "60 FPS (menus + multiplayer)" for the EBOOT), carried
 * over to the static recompilation. All are off unless asked for:
 *
 *   RES_UNLOCK_FPS=1   campaign (game.self 01.00 or v1.7): frame limiter off
 *                      and the minimum timestep 1/30 -> 1/240 s. The game then
 *                      flips once per vblank at most: about 60 fps on the
 *                      frame clock's 62.5 Hz (on a PS3 it is half the vblank
 *                      rate, which is why RPCS3 pairs it with 120 Hz; here
 *                      PS3_VBLANK_HZ=120 lets it run past 100 fps).
 *   RES_FOV=<float>    campaign: the field-of-view factor (the game's own is
 *                      1.0; RPCS3's "Extended FOV" is 0.7).
 *   RES_MENU_60FPS=1   EBOOT (front end, multiplayer): one vblank per frame
 *                      instead of two, and its timesteps 1/30 -> 1/60 s.
 *   RES_SPEAKERS=0     both: the game's speaker-count getter reads its global
 *                      again. On by default: the getter answers 6 (the 5.1
 *                      layout the runtime reports). The global starts at 2 and
 *                      the audio-out setup sets it to 6 on another thread, so a
 *                      sound setup that read it first mixed the whole session
 *                      in the 2ch layout. From the Resistance PC port by
 *                      Psyykosis (MIT).
 *   RES_FRAME_CAP_FIX=0  campaign: keep the frame cap a checkpoint reload can
 *                      leave behind (see res_campaign_frame below). On by
 *                      default.
 *
 * A patch is two halves. The data half (timesteps, FOV) is written into guest
 * memory here, after the ELF is loaded and before the guest starts. The code
 * half cannot be: a static recompilation never executes the guest's code
 * words. Those instructions were lifted in both forms instead (ppu_lifter.py
 * --insn-patches with port/patches/<elf>.insn.json), and the lifted
 * code picks one by the g_res_* switches defined below.
 *
 * Every patch set is keyed to the words it expects at and around its
 * addresses; an ELF that does not hold all of them gets nothing.
 */

#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>

extern "C" {
extern uint8_t* vm_base;
/* Read by the lifted code at the patched instructions. */
int g_res_unlock_fps = 0;
int g_res_menu_60fps = 0;
int g_res_speakers = 1;
/* The vblank rate this process's patches want (0 = the frame clock's own
 * default); PS3_VBLANK_HZ still wins. */
double g_res_vblank_hz = 0.0;
}

namespace {

struct Word { uint32_t addr, value; };

struct PatchSet {
    const char* name;
    const Word* sig;        /* all must match, or the set is not applied */
    size_t      nsig;
    uint32_t    min_dt;     /* minimum timestep (1/30 s), 0 = none */
    uint32_t    fov;        /* field-of-view factor (1.0), 0 = none */
    uint32_t    pace;       /* the word holding the frame-pacing block's address */
};

/* game.self v1.7 (RPCS3 PPU-a08a187d78dd68ebc15b4c6c34335ea55d8dc0f7) */
const Word kGameV17[] = {
    {0x001E6F64, 0x7FBFE840}, {0x001E6F68, 0x419DFFDC}, {0x001E6F6C, 0x4BE989BD},
    {0x00688F00, 0x3D088889}, {0x00688F08, 0x102CB2D4},
    {0x006C6C38, 0x3EDBB51E}, {0x006C6C3C, 0x3F800000},
};
/* game.self 01.00 (RPCS3 PPU-0cc2ddc637def2fe22b5e89563f773e4c0cae024) */
const Word kGame100[] = {
    {0x001E5DAC, 0x7FBFE840}, {0x001E5DB0, 0x419DFFDC}, {0x001E5DB4, 0x4BE999ED},
    {0x00678ED0, 0x3D088889}, {0x00678ED8, 0x102CBED4},
    {0x006B638C, 0xBEDBB51E}, {0x006B6390, 0x3F800000},
};
const PatchSet kCampaign[] = {
    {"game.self v1.7",  kGameV17, sizeof kGameV17 / sizeof *kGameV17, 0x00688F00, 0x006C6C3C, 0x00688C88},
    {"game.self 01.00", kGame100, sizeof kGame100 / sizeof *kGame100, 0x00678ED0, 0x006B6390, 0x00678C58},
};

/* EBOOT.BIN, disc 01.00 (RPCS3 PPU-cb404d48...) */
const Word kEboot[] = {
    {0x0003F264, 0x2F800002}, {0x0003F268, 0x419EF3DC},
    {0x0003F274, 0x38600002}, {0x0003F278, 0x48060EFD},
    {0x0008D680, 0x38600002}, {0x0008D684, 0x48012AF1},
    {0x0073A3D8, 0x3D088889}, {0x0073A3E0, 0x10347CD4},
    {0x00732F00, 0x101B990E}, {0x00732F04, 0x3D088889},
};
const uint32_t kEbootDt[] = {0x0073A3D8, 0x00732F04};

uint32_t rd32(uint32_t a)
{
    const uint8_t* p = vm_base + a;
    return (uint32_t)p[0] << 24 | (uint32_t)p[1] << 16 | (uint32_t)p[2] << 8 | p[3];
}

void wr32(uint32_t a, uint32_t v)
{
    uint8_t* p = vm_base + a;
    p[0] = (uint8_t)(v >> 24); p[1] = (uint8_t)(v >> 16); p[2] = (uint8_t)(v >> 8); p[3] = (uint8_t)v;
}

void wrf32(uint32_t a, float f)
{
    uint32_t v;
    memcpy(&v, &f, 4);
    wr32(a, v);
}

bool matches(const Word* w, size_t n)
{
    for (size_t i = 0; i < n; i++)
        if (rd32(w[i].addr) != w[i].value) return false;
    return true;
}

bool env_on(const char* name)
{
    const char* e = getenv(name);
    return e && *e && strcmp(e, "0") != 0;
}

}  // namespace

/* The campaign's frame-time target: the float at +4 of the frame-pacing block
 * (0x10331FC0 in v1.7, 0x10330B40 in 01.00). The main loop's frame limiter
 * (func_001E6EC4 in v1.7) waits until 0.9 x target has passed since the last
 * frame. The game sets it to 1/30 s while it loads and at a checkpoint, and to
 * 0 (no wait) when play starts and when the player respawns -- but the respawn
 * path (func_002AD4DC) clears it in one of its states only. After the other
 * kinds of checkpoint reload it stayed at 1/30, and the game ran at a flat 33
 * fps (30.3 ms frames) until the next level loaded, from 50-60 before. A PS3,
 * held to 30 by vsync, never showed it. */
static uint32_t s_frame_target;

/* Called from the player's look code every frame the player has control
 * (mouselook.cpp's res_mouselook_pre): a 1/30 target left there is cleared,
 * as the game itself does when play starts. */
extern "C" void res_campaign_frame(void)
{
    const uint32_t a = s_frame_target;
    if (!a || rd32(a) != 0x3D088889u) return;   /* 1/30 */
    wr32(a, 0);
    static int n;
    if (n++ < 4)
        fprintf(stderr, "[patch] frame cap: a 1/30 s frame target left by a load or a checkpoint was cleared "
                        "(RES_FRAME_CAP_FIX=0 keeps it)\n");
}

/* Apply what the environment asks for to the ELF just loaded. */
extern "C" void res_apply_patches(void)
{
    const bool unlock = env_on("RES_UNLOCK_FPS");
    const char* fov_s = getenv("RES_FOV");
    const bool menu60 = env_on("RES_MENU_60FPS");
    if (const char* e = getenv("RES_SPEAKERS"); e && strcmp(e, "0") == 0) {
        g_res_speakers = 0;
        fprintf(stderr, "[patch] RES_SPEAKERS=0: the speaker-count getter reads its global\n");
    }
    if (!vm_base) return;
    if (const char* e = getenv("RES_FRAME_CAP_FIX"); !(e && strcmp(e, "0") == 0)) {
        for (const PatchSet& ps : kCampaign) {
            if (!matches(ps.sig, ps.nsig)) continue;
            const uint32_t block = rd32(ps.pace);
            if (block >= 0x10000000u && block < 0x20000000u) s_frame_target = block + 4;
            break;
        }
    }
    if (!unlock && !(fov_s && *fov_s) && !menu60) return;

    for (const PatchSet& ps : kCampaign) {
        if (!matches(ps.sig, ps.nsig)) continue;
        if (unlock) {
            g_res_unlock_fps = 1;
            wrf32(ps.min_dt, 1.0f / 240.0f);
            fprintf(stderr, "[patch] %s: unlock fps (frame limiter off, min timestep 1/240)\n",
                    ps.name);
            /* Uncapped, the campaign flips on every vblank tick it makes. On
             * 16 ms ticks a 17 ms frame waits for the tick after next (32 ms),
             * which held it to ~50. On 120 Hz ticks it measured 75 fps on a
             * loaded machine, game speed unchanged (the patch moves the game
             * to real time steps). Capping flips at 60 (PS3_FLIP_MAX_HZ=60)
             * measured lower, 50: the game waits for the previous flip before
             * its next frame, so a held flip adds to the frame time. Read by
             * the frame clock after this; set PS3_VBLANK_HZ to override. Not
             * an environment variable: an exitspawn back to the menus would
             * inherit it, and the EBOOT's fixed 1/60 timesteps would then run
             * at double speed on 120 Hz ticks. */
            g_res_vblank_hz = 120.0;
        }
        if (fov_s && *fov_s) {
            const float f = (float)atof(fov_s);
            if (f > 0.1f && f < 4.0f) {
                wrf32(ps.fov, f);
                fprintf(stderr, "[patch] %s: fov factor %.3f (game default 1.0)\n", ps.name, f);
            } else {
                fprintf(stderr, "[patch] RES_FOV=%s ignored (0.1 .. 4)\n", fov_s);
            }
        }
        return;
    }

    /* The front end: the campaign switches are for the game.self it spawns
     * (which inherits this environment), so only RES_MENU_60FPS applies. */
    if (matches(kEboot, sizeof kEboot / sizeof *kEboot)) {
        if (menu60) {
            g_res_menu_60fps = 1;
            for (uint32_t a : kEbootDt) wrf32(a, 1.0f / 60.0f);
            fprintf(stderr, "[patch] EBOOT: 60 fps menus (one vblank per frame, timestep 1/60)\n");
        }
        return;
    }

    fprintf(stderr, "[patch] RES_UNLOCK_FPS/RES_FOV/RES_MENU_60FPS: no patch set matches "
                    "this ELF; nothing applied\n");
}
