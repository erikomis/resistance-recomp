#include "../../runtime/spu/spu_workload.h"

extern void spu_begin_image(int image_id);
extern void resistance_spu_spu_func_00000A00(spu_context*);
extern void resistance_spu_spu_recomp_register(void);

/* v1.7 game.self: the job-manager policy module is byte-identical to 01.00's
 * (fingerprint 4E4076E700C6AFF3). The two raw-SPU programs game.self streams
 * in after the intro movie changed in v1.7 (01.00: 1894F66385AFD052 and
 * 5A10969E466A2298); captured with SPU_RAW_DUMP_DIR and lifted at base 0x60
 * like the 01.00 ones. */
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
    spu_workload_register_img(0x47C9CB480016D493ULL,
                              resistance_rawspu1_spu_func_00000090,
                              3,
                              "resistance_rawspu_47C9CB480016D493");
    spu_begin_image(0);
}

__attribute__((constructor))
static void resistance_spu_autoreg(void)
{
    resistance_spu_register();
}
