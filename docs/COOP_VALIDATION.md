# Resistance co-op validation

The local v1.7 campaign was tested on macOS/Metal on 2026-11-16 using
an isolated copy of the installed data and profile, without the campaign
quick save. The normal frame cap was kept; two virtual controllers were
provided with `PAD_FILE` (port 0) and `PAD_FILE2` (port 1).

## Opening-movie stall

The previous co-op run stopped after the movie because the decoder emitted
fewer pictures than the game submitted access units. An idle drain flushes
FFmpeg's reference state. When input resumes before the next IDR, H.264 can
suppress pictures until that IDR. The existing `AV_CODEC_FLAG2_SHOW_ALL`
change in `vdec_ffmpeg.c` allows those pictures to be emitted; concealment
artifacts are possible until the next keyframe.

The new `vdec_resume_avc` regression feeds a 30-picture H.264 stream and
drains it twice between access units. With the flag it emits 30 pictures;
a temporary negative-control build without the flag emits only 18 and
fails the assertion. This test checks picture delivery, not the visual
quality of frames after a flush. The existing decode/RGBA tests continue
to compare uninterrupted decoding with FFmpeg reference output.

## Manual reproduction

1. Start EBOOT with separate temporary HDD roots and both input files.
2. Open **Cooperative → New Game** with controller 1.
3. Press **START on controller 2** to join; select **Medium** on controller 1.
4. Allow the campaign opening movie to finish. In the tested run the log
   reports `idle 1500 ms: drained to 5584 of 5584 pictures`.
5. At **The Gauntlet / Press the START button**, press START on controller 1.
   A frame counter or a low draw count alone cannot distinguish this screen
   from a stalled movie. The earlier movie script never pressed START here.
6. Wait for the in-engine opening scene. Gameplay renders in two vertical
   viewports, with separate player health and ammunition displays. Player
   death and respawn were also observed while the test controllers were idle.

## Automated checks

Configure the runtime with `PS3RECOMP_BUILD_TESTS=ON` and
`PS3RECOMP_FFMPEG_RUNTIME=ON`, build `test_vdec_ffmpeg` and `test_vdec_stub`,
then run `ctest --test-dir <build> -R '^vdec_' --output-on-failure`.
Set `PS3RECOMP_FFMPEG_AVCODEC` and `PS3RECOMP_FFMPEG_AVUTIL` to the installed
libraries if they are outside the dynamic loader's search path.

All 11 video tests passed, including generated media, stub/disabled/missing
library paths, AVC/MPEG-2 decoding, RGBA conversion, and drain/resume.
The standalone `test_pad_rumble`, `test_pad_adaptive`, `test_pad_hotplug`,
and `test_pad_sensor_mode` tests also passed.

This validates entry into the first co-op level with virtual controllers.
A full campaign run, performance benchmarking, and two physical controllers
remain separate checks.
