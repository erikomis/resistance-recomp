#include "../../runtime/spu/spu_workload.h"

extern void spu_begin_image(int image_id);
extern void resistance_spu_spu_func_00000A00(spu_context*);
extern void resistance_spu_spu_recomp_register(void);

/* The two raw-SPU programs game.self streams in by proxy DMA when a level
 * starts (LS 0x60.., entry 0x90) -- the only two it links; EBOOT.BIN links the
 * same two. extract_rawspu_elf.py cuts them out of game.elf; they are lifted at
 * base 0x60. spu_raw.c fingerprints the GET run as it sits in local store
 * (FNV-1a-64).
 *
 * The second one carries a 0x50-byte parameter area at image+0x5680 (LS
 * 0x56E0) that the PPU fills in before every load -- heap pointers, per-level
 * values, the raw SPU's own MMIO window -- so the bytes it is loaded with
 * differ with the level, the checkpoint and the SPU number (one run captured
 * 5A10969E466A2298). It is registered with that area masked out;
 * 2B2DB6A5DE233A5F is the fingerprint of the program with the area zero, as
 * it is in game.elf. */
extern void resistance_rawspu0_spu_func_00000090(spu_context*);
extern void resistance_rawspu0_spu_recomp_register(void);
extern void resistance_rawspu1_spu_func_00000090(spu_context*);
extern void resistance_rawspu1_spu_recomp_register(void);

static void resistance_spu_register(void)
{
    spu_begin_image(1);
    resistance_spu_spu_recomp_register();
    spu_workload_register_img(0x4E4076E700C6AFF3ULL,
                              resistance_spu_spu_func_00000A00,
                              1,
                              "resistance_spu_4E4076E700C6AFF3");
    spu_begin_image(2);
    resistance_rawspu0_spu_recomp_register();
    spu_workload_register_img(0x1894F66385AFD052ULL,
                              resistance_rawspu0_spu_func_00000090,
                              2,
                              "resistance_rawspu_1894F66385AFD052");
    spu_begin_image(3);
    resistance_rawspu1_spu_recomp_register();
    spu_workload_register_img_masked(0x2B2DB6A5DE233A5FULL,
                                     resistance_rawspu1_spu_func_00000090,
                                     3,
                                     "resistance_rawspu_2B2DB6A5DE233A5F",
                                     0x56D0, 0x5680, 0x50);
    spu_begin_image(0);
}

__attribute__((constructor))
static void resistance_spu_autoreg(void)
{
    resistance_spu_register();
}
