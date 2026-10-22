#include "../../runtime/spu/spu_workload.h"

extern void spu_begin_image(int image_id);
extern void resistance_spu_spu_func_00000A00(spu_context*);
extern void resistance_spu_spu_recomp_register(void);

static void resistance_spu_register(void)
{
    spu_begin_image(1);
    resistance_spu_spu_recomp_register();
    spu_workload_register_img(0x4E4076E700C6AFF3ULL,
                              resistance_spu_spu_func_00000A00,
                              1,
                              "resistance_spu_4E4076E700C6AFF3");
    spu_begin_image(0);
}

__attribute__((constructor))
static void resistance_spu_autoreg(void)
{
    resistance_spu_register();
}
