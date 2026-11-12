/*
 * Direct mouse look for the campaign (game.self 01.00 and v1.7).
 *
 * The keyboard-and-mouse layer turns the mouse into the right stick
 * (libs/input/cellPad.c). A stick is a turn RATE: the game caps it at its
 * own top speed and ramps up to it, so a mouse driving it felt slow and
 * floaty. Here the mouse turns the view by its motion itself, as a PC
 * shooter does, through three hooks the lifter puts in the game's code
 * (ppu_lifter.py --insn-patches, "hook"; port/patches/game_100.insn.json and
 * game_v17.insn.json):
 *
 *   res_mouselook_pre   the player's look code (func_002A4D14 in 01.00), just
 *                       before it integrates the look rates at player+0x3D0
 *                       (pitch) and +0x3D4 (yaw) into the view. While the
 *                       mouse drives the stick it learns from the rate the
 *                       game derived which way the game turns for that stick
 *                       (so the game's own Invert setting is followed), then
 *                       zeroes both rates: the stick no longer turns the view.
 *   res_mouselook       the same function after the integration, before the
 *                       pitch clamp: adds the mouse motion since the last
 *                       frame to pitch (+0x360) and yaw (+0x364, wrapped).
 *   res_mouselook_fov   the camera FOV setter's entry (func_000B674C in 01.00;
 *                       f3 = horizontal FOV in degrees, 82 in play): zooming
 *                       in narrows the player's camera, and the mouse slows by
 *                       the same magnification, so a move covers the same part
 *                       of the screen zoomed or not.
 *
 * The scale also follows the game's own Look Sensitivity (Options, 0-9; the
 * stick turns at 0.5x to 1.5x of its speed) and PS3_MOUSE_SENS (the
 * launcher's and the in-game menu's slider). A controller, the look keys and
 * scripted camera moves (a chapter's opening walk turns the view itself) keep
 * the game's own stick handling.
 *
 * Settings:
 *   RES_MOUSE_LOOK=0      the old stick emulation
 *   RES_MOUSE_DEG=<f>     degrees per point of mouse motion at sensitivity 1
 *                         (0.07)
 *   RES_MOUSE_ZOOM=<f>    how much zoom slows the mouse: 1 = matched to the
 *                         magnification (default), 0 = not at all
 *   RES_MOUSE_LOOK_LOG=1  print what the hooks do
 *
 * The player-object offsets, the options block and the approach are the
 * Resistance PC port's (Psyykosis, MIT), found by its author on 01.00; the
 * v1.7 sites were found by their instruction bytes.
 */

#include <atomic>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include "ppu_recomp.h"   /* ppu_context */

extern "C" {
extern uint8_t* vm_base;
int cellPad_host_mouse_look_state(int* fed_rx, int* fed_ry);
int cellPad_host_mouse_look_take(float* dx, float* dy);
}

namespace {

constexpr double kPi = 3.14159265358979323846;
constexpr uint32_t kPitch = 0x360, kYaw = 0x364;          /* view angles, radians */
constexpr uint32_t kPitchRate = 0x3D0, kYawRate = 0x3D4;  /* from the stick */
constexpr uint32_t kLocked = 0x3F07;                      /* 1 = input-locked sequence */
constexpr uint32_t kOptsPtr = 16016, kOpts = 6848, kSens = 68;   /* [[p+16016]+6848]+68 = sens/9 */

uint32_t rd32(uint32_t a)
{
    const uint8_t* p = vm_base + a;
    return (uint32_t)p[0] << 24 | (uint32_t)p[1] << 16 | (uint32_t)p[2] << 8 | p[3];
}
float rdf(uint32_t a) { const uint32_t w = rd32(a); float f; memcpy(&f, &w, 4); return f; }
void wrf(uint32_t a, float f)
{
    uint32_t w; memcpy(&w, &f, 4);
    uint8_t* p = vm_base + a;
    p[0] = (uint8_t)(w >> 24); p[1] = (uint8_t)(w >> 16); p[2] = (uint8_t)(w >> 8); p[3] = (uint8_t)w;
}
bool sane_ptr(uint32_t a) { return a >= 0x10000 && a < 0xF0000000u; }

struct Cfg { int on = -1; double deg = 0.07, zoom = 1.0; int log = 0; };
Cfg& cfg()
{
    static Cfg c;
    if (c.on < 0) {
        const char* e = getenv("RES_MOUSE_LOOK");
        c.on = !(e && e[0] == '0');
        if ((e = getenv("RES_MOUSE_DEG")) && atof(e) > 0.0) c.deg = atof(e);
        if ((e = getenv("RES_MOUSE_ZOOM"))) { c.zoom = atof(e); if (c.zoom < 0) c.zoom = 0; if (c.zoom > 2) c.zoom = 2; }
        c.log = getenv("RES_MOUSE_LOOK_LOG") ? 1 : 0;
        fprintf(stderr, "[mouselook] %s (%.3f deg per point, zoom %.1f)\n",
                c.on ? "direct mouse look" : "off: the mouse drives the stick", c.deg, c.zoom);
    }
    return c;
}

/* Which way the view turns for a positive stick deflection (learned). */
int s_sx = -1, s_sy = 1, s_bad_x, s_bad_y;
void learn(int fed, float rate, int* sign, int* bad, const char* what)
{
    if (fed > -16 && fed < 16) return;
    if (!(rate > 1e-5f || rate < -1e-5f)) return;
    const int s = (fed > 0) == (rate > 0) ? 1 : -1;
    if (s == *sign) { *bad = 0; return; }
    if (++*bad >= 8) {
        *sign = s; *bad = 0;
        if (cfg().log) fprintf(stderr, "[mouselook] %s direction now %d (the game's setting)\n", what, s);
    }
}

std::atomic<unsigned long long> s_script_until{0};
unsigned long long s_last_ms;
std::atomic<double> s_zoom{1.0};   /* tan(fov/2) / tan(82/2) of the player's camera */
std::atomic<uint32_t> s_view_cam{0};

unsigned long long now_ms()
{
    struct timespec ts; clock_gettime(CLOCK_MONOTONIC, &ts);
    return (unsigned long long)ts.tv_sec * 1000ull + (unsigned long long)ts.tv_nsec / 1000000ull;
}

}  // namespace

extern "C" void res_mouselook_pre(ppu_context* ctx)
{
    const uint32_t p = (uint32_t)ctx->gpr[31];
    if (!cfg().on || !sane_ptr(p)) return;
    int frx = 0, fry = 0;
    if (!cellPad_host_mouse_look_state(&frx, &fry)) return;   /* a controller or the keys: as on PS3 */
    const float rp = rdf(p + kPitchRate), ry = rdf(p + kYawRate);
    learn(frx, ry, &s_sx, &s_bad_x, "yaw");
    learn(fry, rp, &s_sy, &s_bad_y, "pitch");
    /* A rate with no stick fed = the game turning the view (a scripted
     * walk): leave it, and keep the mouse out until it stops. */
    const bool fed = frx >= 16 || frx <= -16 || fry >= 16 || fry <= -16;
    if (!fed && (std::fabs(rp) > 0.02f || std::fabs(ry) > 0.02f)) {
        s_script_until = now_ms() + 400;
        return;
    }
    if (now_ms() < s_script_until || (vm_base[p + kLocked] == 1)) return;
    wrf(p + kPitchRate, 0.0f);
    wrf(p + kYawRate, 0.0f);
}

extern "C" void res_mouselook(ppu_context* ctx)
{
    const uint32_t p = (uint32_t)ctx->gpr[31];
    if (!cfg().on || !sane_ptr(p)) return;
    float dx, dy;
    int mouse = cellPad_host_mouse_look_take(&dx, &dy);
    /* RES_MOUSE_LOOK_TEST=<points per frame>: a steady move right, for a
     * headless check that the view turns (with RES_MOUSE_LOOK_LOG=1) */
    static const double test = getenv("RES_MOUSE_LOOK_TEST") ? atof(getenv("RES_MOUSE_LOOK_TEST")) : 0.0;
    if (test != 0.0) { mouse = 1; dx = (float)test; dy = 0.0f; }
    const unsigned long long now = now_ms();
    /* motion gathered while the look code was not running (menus, pause,
     * cutscenes, a vehicle) is dropped, not applied in one jump */
    const bool stale = now - s_last_ms > 150;
    s_last_ms = now;
    if (!mouse || stale || now < s_script_until || vm_base[p + kLocked] == 1) return;
    if (dx == 0.0f && dy == 0.0f) return;
    double k = cfg().deg * kPi / 180.0;
    /* the game's Look Sensitivity, stored as setting/9: 0.5x .. 1.5x */
    const uint32_t c = rd32(p + kOptsPtr);
    const uint32_t o = sane_ptr(c) ? rd32(c + kOpts) : 0;
    if (sane_ptr(o)) {
        const float s = rdf(o + kSens);
        if (s >= 0.0f && s <= 1.0f) k *= 0.5 + s;
    }
    if (cfg().zoom > 0.0) {
        const double z = s_zoom.load();
        if (z > 0.0 && z < 1.0) k *= std::pow(z, cfg().zoom);
    }
    double yaw = (double)rdf(p + kYaw) + (double)s_sx * (double)dx * k;
    while (yaw > kPi) yaw -= 2.0 * kPi;
    while (yaw < -kPi) yaw += 2.0 * kPi;
    wrf(p + kYaw, (float)yaw);
    wrf(p + kPitch, (float)((double)rdf(p + kPitch) + (double)s_sy * (double)dy * k));
    if (cfg().log) {
        static int n;
        if (n++ < 300) fprintf(stderr, "[mouselook] d=(%.1f,%.1f) k=%.5f -> yaw %.3f pitch %.3f\n",
                               dx, dy, k, yaw, (double)rdf(p + kPitch));
    }
}

/* The FOV setter: the camera given the plain gameplay FOV (82) is the
 * player's; a narrower FOV on it is a zoom. */
extern "C" void res_mouselook_fov(ppu_context* ctx)
{
    const uint32_t cam = (uint32_t)ctx->gpr[3];
    const double f = ctx->fpr[3];
    if (!(f > 1.0 && f < 179.0)) return;
    if (std::fabs(f - 82.0) < 0.01) s_view_cam = cam;
    const uint32_t v = s_view_cam.load();
    if (!v || cam != v) return;
    s_zoom = std::tan(f * 0.5 * kPi / 180.0) / std::tan(82.0 * 0.5 * kPi / 180.0);
}
