/*
 * The engine's spin delay, made to give its core back.
 *
 * Every executable of the game has one leaf function that is 32 times
 * `or r31,r31,r31` and a blr: the Cell's db8cyc, an 8-cycle pause, so about
 * 256 cycles of nothing (0x5CA40 in the EBOOT, 0x613B4 in the v1.7 EBOOT,
 * 0x50DA8 in game.self 01.00, 0x50EFC in v1.7). The job manager's wait loops
 * call it between two reads of memory an SPU job will write when it is done
 * (func_00051514 in v1.7, the PPU waiting for its WWS jobs). On a PS3 that
 * spin costs the waiting thread only. Lifted, a call is a few host
 * nanoseconds, so the loop holds a Mac core at 100% for as long as the SPUs
 * take, heats the machine and competes with the threads it waits for.
 *
 * res_spin_pause is an insn-patch hook at the delay function's entry
 * (port/patches/<elf>.insn.json). Calls less than RES_SPIN_GAP_NS apart
 * (default 2000) are one wait; once a wait has lasted RES_SPIN_AFTER_US
 * (default 200) each further call first sleeps RES_SPIN_SLEEP_US (default 20;
 * 0 turns the pause off and leaves only the count). Any real work between two
 * calls ends the wait. Most waits are short: in "The Cathedral", uncapped,
 * about 12,000 a second, most under 10 us, on the main thread's critical
 * path. Sleeping in those (after 64 calls, about 1 us) took it from 54 to 31
 * fps; the long ones (1-10 ms, the CPU this is about) are past 200 us.
 *
 * RES_SPIN_STATS=1 logs, every 5 s, how many waits there were, how long they
 * lasted and how much of them was spent asleep.
 */

#include <atomic>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <ctime>

struct ppu_context;

extern "C" uint64_t host_monotonic_ns(void);

namespace {

int      g_sleep_us = -1;
uint64_t g_after;
uint64_t g_gap;
int      g_stats;

struct Stats {
    std::atomic<uint64_t> waits{0}, calls{0}, spin_ns{0}, sleep_ns{0}, slept{0};
    std::atomic<uint64_t> hist[6]{};      /* < 10 us, 100 us, 1 ms, 10 ms, 100 ms, more */
    std::atomic<uint64_t> last_print{0};
};
Stats g_st;

void init()
{
    const char* e = getenv("RES_SPIN_SLEEP_US"); g_sleep_us = e ? atoi(e) : 20;
    if (g_sleep_us < 0) g_sleep_us = 0;
    e = getenv("RES_SPIN_AFTER_US"); g_after = (e ? (uint64_t)atoll(e) : 200u) * 1000u;
    e = getenv("RES_SPIN_GAP_NS"); g_gap = e ? (uint64_t)atoll(e) : 2000u;
    e = getenv("RES_SPIN_STATS"); g_stats = e && *e && *e != '0';
    if (g_stats)
        fprintf(stderr, "[spin] pause after %llu us of waiting, %d us asleep per call, gap %llu ns\n",
                (unsigned long long)(g_after / 1000u), g_sleep_us, (unsigned long long)g_gap);
}

void end_wait(uint64_t start, uint64_t last)
{
    const uint64_t d = last - start;
    g_st.waits++; g_st.spin_ns += d;
    int b = 0;
    for (uint64_t lim = 10000; b < 5 && d >= lim; lim *= 10) b++;
    g_st.hist[b]++;
}

void maybe_print(uint64_t now)
{
    uint64_t lp = g_st.last_print.load(std::memory_order_relaxed);
    if (lp == 0) { g_st.last_print.compare_exchange_strong(lp, now); return; }
    if (now - lp < 5000000000ull || !g_st.last_print.compare_exchange_strong(lp, now)) return;
    const double secs = (double)(now - lp) / 1e9;
    fprintf(stderr, "[spin] %.0f waits/s, %.0f calls/s, waiting %.1f%% of a core (%.1f%% asleep), "
                    "<10us %llu <100us %llu <1ms %llu <10ms %llu <100ms %llu more %llu\n",
            (double)g_st.waits.exchange(0) / secs, (double)g_st.calls.exchange(0) / secs,
            100.0 * (double)g_st.spin_ns.exchange(0) / 1e9 / secs,
            100.0 * (double)g_st.sleep_ns.exchange(0) / 1e9 / secs,
            (unsigned long long)g_st.hist[0].exchange(0), (unsigned long long)g_st.hist[1].exchange(0),
            (unsigned long long)g_st.hist[2].exchange(0), (unsigned long long)g_st.hist[3].exchange(0),
            (unsigned long long)g_st.hist[4].exchange(0), (unsigned long long)g_st.hist[5].exchange(0));
}

}  // namespace

extern "C" void res_spin_pause(ppu_context*)
{
    if (g_sleep_us < 0) init();
    static thread_local uint64_t t_start, t_last;
    static thread_local unsigned t_streak;

    uint64_t now = host_monotonic_ns();
    if (t_last && now - t_last < g_gap) {
        ++t_streak;
    } else {
        if (g_stats && t_streak) end_wait(t_start, t_last);
        t_start = now; t_streak = 1;
    }
    if (g_stats) { g_st.calls++; maybe_print(now); }
    if (g_sleep_us > 0 && now - t_start >= g_after) {
        struct timespec ts = { 0, (long)g_sleep_us * 1000L };
        nanosleep(&ts, nullptr);
        const uint64_t after = host_monotonic_ns();
        if (g_stats) { g_st.sleep_ns += after - now; g_st.slept++; }
        now = after;
    }
    t_last = now;
}
