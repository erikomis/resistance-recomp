#include "../../runtime/spu/spu_workload.h"

extern void spu_begin_image(int image_id);
extern void resistance_spu_spu_func_00000A00(spu_context*);
extern void resistance_spu_spu_recomp_register(void);

/* v1.7 game.self: the job-manager policy module is byte-identical to 01.00's
 * (fingerprint 4E4076E700C6AFF3). The two raw-SPU programs game.self streams
 * in when a level starts (LS 0x60.., entry 0x90) changed in v1.7; they are the
 * only two it links (extract_rawspu_elf.py cuts them out of game.elf) and are
 * lifted at base 0x60 like the 01.00 ones.
 *
 * The second one carries a 0x50-byte parameter area at image+0x5780 (LS
 * 0x57E0) that the PPU fills in before every load -- heap pointers, per-level
 * values, the raw SPU's own MMIO window -- so the bytes it is loaded with
 * differ with the level, the checkpoint and the SPU number (one run captured
 * 47C9CB480016D493, a Continue from a later checkpoint 5771C48D28664765). It
 * is registered with that area masked out; F9726FDA4A599084 is the
 * fingerprint of the program with the area zero, as it is in game.elf. */
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
    spu_workload_register_img(0x6D03497950639952ULL,
                              resistance_rawspu0_spu_func_00000090,
                              2,
                              "resistance_rawspu_6D03497950639952");
    spu_begin_image(3);
    resistance_rawspu1_spu_recomp_register();
    spu_workload_register_img_masked(0xF9726FDA4A599084ULL,
                                     resistance_rawspu1_spu_func_00000090,
                                     3,
                                     "resistance_rawspu_F9726FDA4A599084",
                                     0x57D0, 0x5780, 0x50);
    spu_begin_image(0);
}

__attribute__((constructor))
static void resistance_spu_autoreg(void)
{
    resistance_spu_register();
}
