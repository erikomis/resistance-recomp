#include "../../runtime/spu/spu_workload.h"

extern void spu_begin_image(int image_id);
extern void resistance_spu_spu_func_00000A00(spu_context*);
extern void resistance_spu_spu_recomp_register(void);

/* The two raw-SPU programs game.self streams in by proxy DMA after the intro
 * movie (LS 0x60.., entry 0x90). Their fingerprints are FNV-1a-64 over the
 * GET run as it sits in local store, which is what spu_raw.c matches;
 * captured with SPU_RAW_DUMP_DIR and lifted at base 0x60. */
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
    spu_workload_register_img(0x5A10969E466A2298ULL,
                              resistance_rawspu1_spu_func_00000090,
                              3,
                              "resistance_rawspu_5A10969E466A2298");
    spu_begin_image(0);
}

__attribute__((constructor))
static void resistance_spu_autoreg(void)
{
    resistance_spu_register();
}
