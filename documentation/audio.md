# Audio playback

Sound is client-side only: no packet says "play this". Seihai plays the
`PLAYSOUND` cues of the walk and swim animations and the scripts' `play`,
`playlooped`, `play2` and `stopsound`. Everything is optional.
With `--no-sound`, `sound=0`, no `sounds` folder, no audio device or a build
without the device backend, the client runs the same and plays nothing.

Sources: `client-internals/sound-and-music.md` (the 2.05 sound path) and
`formats/gani.txt` in Graal_analysis, and the real 2.22 data folder
(`levels/ganis/*.gani`, `sounds/*.wav`). Only what those state is built.

## Where it lives

- `client/src/sound/mod.rs`: the `SoundOut` trait (`play`, `stop`,
  `stop_all_loops`), `NullSound`, `RecordingSound` (for tests), `pan_gains`,
  `default_backend`.
- `client/src/sound/engine.rs`: `SoundEngine`, the rules below. It talks to
  the backend only through `SoundOut`, so every rule is tested without a
  device.
- `client/src/sound/wav.rs`: the WAV decoder.
- `client/src/sound/rodio_out.rs`: the device backend, only with the
  `audio-rodio` feature.
- `client/src/assets/gani.rs`: the parser now reads steps and `PLAYSOUND`.
- `OfflineSession`: steps the walk or swim GANI on every successful move and
  queues a `SoundEvent` for each cue in the frame entered
  (`take_sound_events`). `bin/main.rs` plays them.

## What plays

- **GANI cues.** `PLAYSOUND name x y`. In every shipped GANI the cue sits
  right after the rows of a step, so it belongs to that step. A step is four
  direction rows, or one row with `SINGLEDIRECTION`. A blank line also closes
  a short step. (The earlier parser took each row as a frame and attached the
  cue to the wrong one.) In the 2.22 folder: 108 cues in 62 of the 137 files, names such
  as `steps2.wav`, `steps.wav`, `lift.wav`, `sword.wav`; the usual
  coordinates are `1.5 2`, the player's own point.
- **Once per frame entry.** One GANI step per move (a move is half a tile, 50 ms
  at walking speed, which is also the length of a GANI step with no `WAIT`).
  Each frame entered with a cue plays it once. Frame 0 is the pose before the
  first step and does not play until the loop wraps back to it.
- **In water** the swim GANI is stepped instead of the walk one.
- **Scripts.** `play file` plays at the NPC's centre (its x/y plus half its
  size), `playlooped file` the same with the `+` loop prefix, `stopsound file`
  stops it. `play2 file,x,y,volume` (2.1 and later) plays at `x,y`; the volume
  is ignored. A weapon's script has no place of its own, so its sound is at the
  player. `SessionScripts::apply` queues these with the GANI cues
  (`sound_events`, `sound_stops`), and `bin/main.rs` plays them. The NPC's size
  is its `setimgpart` part or its `setshape` shape; the image's own size is not
  known to the script host, so with neither a 32-pixel image is assumed. Only x
  sets the pan, so a wrong guess moves it a little.
- **Missing files online.** A sound that is in no folder is asked for with
  `PLI_WANTFILE` by its raw name (as written, without `+`), once per name
  (`FileSession::need`). It is saved to the cache's `sounds` folder, which is
  one of the engine's folders, and plays on the next trigger, as in 2.05.
- **Folder.** `<assets>/sounds`, else `<assets>/levels/sounds`. File names
  match without regard to case or path, as on Windows.

## Rules (2.05, from the client note)

- A leading `+` means looped and is stripped. Leaving a level stops looped
  effects (`stop_all_loops`, called when the level changes).
- `.wav` is an effect. Any other extension is music: "music skipped" is logged
  once, nothing plays. MIDI is not emulated.
- At most 8 effects at once; a ninth is dropped. A one-shot occupies a voice
  for its length, a loop until stopped.
- Pan is `(sound x - listener x) * 150`, clamped to +/-10000 DirectSound
  units, negated by `reversestereo`. The listener is the player at
  `(x + 1.5, y + 2)`. A device backend turns pan into left and right gain at
  hundredths of a dB.
- Decoded sounds are cached by lower-cased file name.
- A missing file is logged once per name. The folder is rescanned at most once
  a second, so a file that arrives later plays on the next trigger.

## Settings and flags

`--no-sound` and `--sound` on the command line, `sound=0` / `sound=1` in
`seihai.conf` (same values as `follow_serverwarp`). The command line wins, then
the file, then the default (on).

## Formats

`wav.rs` reads PCM 8, 16, 24 and 32-bit (mono or stereo) and IMA ADPCM (format
tag 17), also inside WAVE_FORMAT_EXTENSIBLE. All 104 files of the 2.22 `sounds`
folder decode: 100 PCM (8-bit at 5 to 22 kHz, and some 16-bit), 4 IMA ADPCM
(`fire_torch0-2`, `cauldron_bubbles`). The ADPCM output differs from ffmpeg's by
at most 27 of 32768 (ffmpeg rounds each step a little differently); PCM matches
exactly.

## The device backend

Real output is the `audio-rodio` cargo feature (`rodio` 0.19 on `cpal`). It is on by default; `--no-default-features --features dev` compiles it out. On Linux it needs the ALSA development files and `pkg-config` to build. If no device opens at run time, the engine falls back to the null backend and logs one line.

The output stream runs on its own thread (`seihai-audio`) and the engine sends it play and stop commands over a channel. On Windows the launcher's window (winit, drag and drop on) makes the main thread a single-threaded COM apartment, and a WASAPI stream opened on that thread opened fine but played nothing. The first Windows run (2026-10-30, Realtek headphones, Graal the Adventure on 2.22) was silent that way. With the stream on its own thread, footsteps and script sounds played.

## Unresolved

- **Volume with distance.** The note says volume is an attenuation that grows
  with distance on both axes, floored at -10000, but gives no formula. Every
  effect plays at full volume. Pan is implemented.
- **Frame timing.** GANI `WAIT` is not modelled; a frame lasts one move. The real
  client advances GANIs by time, so footstep spacing is approximate.
- **NPC image size.** Script sounds use the image part or shape for the NPC's
  centre, else a 32-pixel image (see above).
- **Gmap neighbours.** 2.05 shifts a sound from a neighbouring gmap level by 64
  tiles. Only the current level's NPCs run scripts here, so nothing is shifted.
- **Built-in effects** (sword, bomb, item pickup, chest, jump: the 24 names the
  2.05 client plays without any NPC) are not triggered.
- **Missing files offline.** Offline there is no server to ask; the name is
  logged once. Three sounds named by 2.22 GANIs are not
  in the 2.22 folder: `freeze.wav`, `hammer3.wav`, `water2.wav`.
- **Other players' ganis** are not stepped, so their footsteps are silent.
- **Music** (MIDI, MP3, BASS streams on 2.17+), `setmusicvolume`, and the 2.22
  volume options (`sfxvolume`, `midivolume`...) are not handled. The 2.22
  extension-to-backend dispatch is still untraced.

## Sword cue (item 22)

The sword swing queues the `PLAYSOUND` cue of `sword.gani` when the cue's step is entered, through the same `SoundEvent` queue as the walk and swim cues, so `sword.wav` plays once per swing (the built-in set ships a synthesised `sounds/sword.wav`). No sound, no cue: with `--no-sound` the events are dropped as before.
