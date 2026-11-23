# The stadium party (design stage)

The film's climax: a stadium full of people dancing at night, with everything escalated as far as it goes. It is the
destination of the train's ending: the camera follows a drone up and over a station roof at night, climbs over the
stadium's rim and flies into the party. The night arrival's flight (`prototypes/train_night.py`) joins its last
frames to this scene.

This document covers the design stage: a floating-point Python prototype and its previews. The segment built on
CARDIAC from it is [`segments/stadium`](../../segments/stadium/README.md).

## Files

| File | What |
|---|---|
| `prototypes/stadium.py` | The prototype: the scene, the crowd, the show, the camera, the renderer, the previews |
| `prototypes/audience/stadium_build.py` | Blender (headless): the stadium's solid geometry, `stadium(r)`; writes `stadium.npz` + `stadium.json` |
| `prototypes/audience/pose_people.py` | Blender (headless): the dancers posed by procedural moves; writes `people.npz` |

Inputs and caches live in `audience/` in the source files' folder ($CARD_ATTACK_WORK, not in this repository): `dl/` (the five glTF models),
`people.npz`, `stadium/`, `atlas.npz` (the impostors, made on first use), `frames_*/`; the demos' clips in
`demos/` there.

```
python prototypes/stadium.py prep                         # Blender: the dancers and the stadium (once)
python prototypes/stadium.py film OUT.mp4 [--procs 8]     # the preview (TAG names the frame cache)
python prototypes/stadium.py film OUT.mp4 --start 3       # the preview as joined to the night arrival (from JOIN_T)
python prototypes/stadium.py still T OUT.png
python prototypes/stadium.py sheet OUT.png                # 8 stills, and the same at 384x192 (OUT_384.png)
python prototypes/stadium.py compare OUT.png [T]          # the near crowd as models, then as photo cut-outs
```

### The scene

The stadium is a standalone scene, centred on the origin, metres, Z up, the stage and the giant screen at the north
end (+Y), the crowd facing them. Seen from outside it reads as a lit oval (the outer wall's LED skin, the canopy's lit
inner edge), searchlights and the moving heads' beams reaching up into the night sky, lasers over the bowl, the
city's street lights and windows round it out to 1.4 km.

- `Scene()` loads everything (about 3 s); `render(sc, eye, target, t, w=1536, h=768)` draws any camera at show time
  `t` (seconds; the beams, lasers, dancers and pyro all run on `t`), returning a float image 0..1. `camera(t)` is this
  design's own path.
- **The hand-off:** the night arrival hands over at the crest, **t = 3.0 s** (`JOIN_T`): eye (-48, -110, 48) looking
  at (0, 40, 10), 2.2 m a frame north-east and down. The keys before `JOIN_T` (hugging the outer wall, climbing) are
  only for running the scene standalone; t = 0 still works that way.
- The night arrival's sky deepens to dark violet before the hand-off, and the stadium's sky does the same from
  `JOIN_T` on, so the two shots match. The outer wall's LED skin is the night arrival's (`skin_light()`, after
  train_night.py's K_STADSKIN and callouts.py's `marquee()`): two rows of every callout in the VGA font scrolling
  opposite ways round the bowl, its scroll on the arrival's frames (`SKIN_F_HAND` = 389, its frame at the hand-off),
  so both shots show the same wall.
- The scene's API (`Scene()`, `render()`, `camera()`, `load_callouts()`, `callout_image()`) is shared with the night
  arrival, which uses the same flags and signs on its way in.

## The look

**From the crest:** the night flight ends hugging the outer wall, the music muffled, then climbs over the rim. The
shot starts at the crest with the whole bowl bursting into view, everything firing: the moving heads and searchlights
full on, every laser fanning (flat over the heads and up into the sky), the pyro's first burst, the confetti already
in the air from the cannons, the screens at full, the beat landing (`MUSIC_FULL`). It settles into the beat over 2 s
(`show(t)['crest']`).

**The orbit and the party:** the camera sweeps a full circle round the bowl above the crowd, counter-clockwise, 40 m
up: east over the crowd with the stage ahead, round the north across the sea of people to the stands, the west, up
at the airship's screen from the south, the stage again (no strobe until the dive). Then it dives into the crowd: a
low, fast weave among the dancers at head height, under the lasers, past the flags at eye level, round to the south;
the look up at the airship (held, no strobe: `AIRSHIP_HOLD`); back down, rising over the front of the crowd to the
screen, banking round over the sea of arms and phones, down in front of the stage and backing away low under the
lasers, and the final rise over the whole bowl. Near people are big black silhouettes against the stage's light, the
raised arms crossing the bright band of lasers, beams and screen.

**The end:** type-only thank-yous, no photographs or likenesses: THANK YOU / SAUL FINGERMAN on the airship's screen
through its hold (`thanks_fingerman()`), THANK YOU / D. W. HAGELBARGER on the giant screen in the last part
(`thanks_hagelbarger()`, `HAG_AT`), each with CO-CREATORS OF CARDIAC, BELL LABS 1968 under it; then the giant
screen's last picture, a photo of the author's CARDIAC (`segments/ending/data/cardiac.bin`), held big to the cut
(`CARDIAC_AT`). The beams and lasers ease to 40% from the thanks on, so the words read through them; no strobe in
either. The picture cuts to black `BLACK_TAIL` = 3 s before `T_END` (the music stops; a thunk and its reverb over the
black).

**Timing:** the party runs from the crest to the end of the film. `T_END` (env `STADIUM_END`, seconds on the show
clock; 47.25 by default) sets the length, and the camera's moves after the crest (`FLIGHT`, nominal times) stretch to
fill it; every beat of the shot above is a nominal time on `FLIGHT`'s clock, stretched with it. The aim turns 10
degrees right after the sweep down (`CHAT_YAW`) so the subject sits left of the stream chat (the credits) in the
right ~380 px. To lengthen the party, add keys after the last and move `T_END` to the last key's time; the show, the
waves, the screens' slots and the beat all run on t without limit.

**65,536 people** exactly (41,100 on the pitch, 24,400 on the tiers): the 16-bit number. The live film's version
draws 32,768 of them (`PARTY_COUNT`): everyone within 12 m of the camera's low flight, the rest thinned, the tiers
more than the pitch.

**The screens.** The giant screen (96 x 48 m, 2:1, a 384x192 picture) runs a montage of the 8 demos whose footage may
be shown: a different one every 2 beats (0.875 s) on the beat's clock from `BEAT_T0`, round them in the callouts'
order, each 4:3 with TITLE GROUP along the bottom, its clip at its own time. The LED walls flanking it run a demo each
at the top with its title and group in big type under it; the banners round the bowl are LED screens running demos
with captions; each changes every bar (`SLOT`), each screen its own order (`on_screen()`). The 8 demos play their
footage (`demos/` in the source files' folder: `index.json`, per demo `frames_160x120.rgb` at 12 fps, `still.png`,
`logo.png`; `load_demos()`); the other 55 callouts are parodies, each a procedural stand-in effect (plasma, tunnel,
stars, rotozoomer, rings in its own colours) under the parody's title and group, never footage.

**Callouts to the demos:** 63 titles and groups (`callouts_final.json` with the source files) as plain text, no
logos: the title in Impact, the group under it in DIN Condensed, white and yellow on a coloured cloth with a white
border (`callout_image()`, `load_callouts()`). **Flags** flown in the crowd: 80 on poles 3.9 m up (30 placed 2.5-9 m
beside the camera's low path, so it passes them at eye level; 50 across the pitch), 2 x 1.25 m, waving, lit by the
show and through the cloth, a little light always on them. About 55% of them are national flags: Finland, Sweden,
Norway, Denmark most, then Germany, the USA, the Netherlands, Poland, Iceland, Estonia, Hungary, and one or two of
the UK, France, Belgium, Czechia, Slovakia (`NATIONS`, weighted; `national_flag()` draws each from its public-domain
design: the Nordic crosses in their own proportions, the tricolours, the Stars and Stripes with 50 stars, the Union
Jack and Slovakia's shield simplified). **LED banners** (12 x 7.5 m) hang from the canopy's inner edge round the
stands, pulsing.

**Clothing:** about half the near crowd wear T-shirts and hoodies printed on the back with a group's name (or its
title screen, for the demos shown), in UV-reactive ink: faint between the hits, readable when the light or the strobe
catches them (`shirt_image()`). The far crowd's impostors carry no prints; their clothes are dark solid colours.

**The airship:** the advertising airship (the night arrival's `blimp()`, `models/blimp.obj`, 7,142 triangles) hovers
over the bowl where `models/blimp.json` puts it, drifting and bobbing; its running lights and beacons from the same
file; its screen running a demo, then the thank-you. Its uplights (sources and points on the hull) are searchlights
here, sweeping over the hull on the arrival's clock and lighting pools on it; the bowl's glow lights its belly
(`load_blimp()`, `blimp_at(t)`; without blimp.json it falls back to a placement of its own).

**The show, everything at once:** 14 moving heads on the truss over the stage (patterns changing every 2 bars: fans
swinging together, crossing, circles; steep enough to land on the front half of the crowd), 18 searchlights (8 on
the stage's front edge, 10 round the canopy) sweeping the sky, 5 laser sources x 12 beams (a flat fan just over the
heads, a fan up into the sky, a rotating tunnel, a chase strobing on the beat), 8 pyro columns, about 9,000 pieces of
confetti, 600 balloons let go from the crowd (a quarter near the camera's path, rising 0.9-1.7 m/s into the beams,
lit from the front and through the rubber: `make_balloons()`, `balloons_at(t)`), a phone in about one raised hand in
five (about 13,000 phone lights), the city's lights.

**Dark, on the beat** (`show(t)`): the moving heads and the lasers hit on beats 1 and 3 and die away (to 5% and
12%), the searchlights on the off-beat, the screen and the LED walls pulse (the screen 30-80%); in the last bar of
every 4-bar phrase, white strobes on the truss flash on the eighth notes and light the haze from 15 m out, so
everything nearer stands black against it.

## The people (the main risk)

People are the hardest thing here to make convincing, so the design keeps them to what reads:

1. **Always from behind or far off, as silhouettes with a thin rim of light**: almost black (albedo 0.025), a rim
   where the surface turns away from the camera, lit by the screen, the LED walls and the beams ahead of them. No
   faces, no front views up close; hands are never large for more than a moment.
2. **Motion is what sells them.** Five rigged CC0 characters by Quaternius (below), posed in Blender by procedural
   dance moves on their own rigs: **jump** (arms up, a jump on every beat), **cheer** (both arms up, pumping),
   **raise** (one arm up holding a phone/can/lighter, bobbing), **pump** (a fist pumped on the beat), **sway** (arms
   overhead, side to side), **bob**, **dance** (fists by the shoulders, alternating, the body twisting), **point** (at
   the screen); and two seated moves (unused now). Each move is a 2-beat cycle of 12 phases; each person keeps the
   beat, give or take a frame. The waves: on the pitch, a band of arms going up rolls from the stage to the back
   every 4 bars; on the tiers, a Mexican wave goes round the bowl.
3. **Crowds as instances**: the 160 nearest in view as the posed meshes; everyone else an impostor: the same pose
   rendered ahead of time into an atlas (from behind and 14 degrees above, 56x96, with its normals), a camera-facing quad, lit per pixel by
   the same lights, so the switch doesn't show. Variation: the model (5), the move, the phase, the height (0.93-1.06),
   the clothes' darkness, the facing (+-10 degrees).

**Models against photographs** (`compare`; the preview is not in this repository):
the same frame with the near crowd as photo cut-outs (nine people photographed from behind, public domain or CC0,
cut out with macOS Vision's subject lifting, darkened to silhouettes with the rim on their outline's inside edge,
bobbing on the beat) against the posed models. The models won, clearly at 384x192:

- The photos are of people standing still (watching geese, a station board, a football match): heads and shoulders
  with their arms down. A party's silhouette is the arms in the air; static photos can't do that, and none of the
  free back-view photos found shows anyone with arms up (the one with both arms up is holding game controllers).
- They can't move except bobbing; the models jump, pump and sway on the beat, and at 384x192 the motion is what reads.
- Up close their heads and hoods do look more real, but as black silhouettes the difference is small; and a flat
  photo turned to the camera shows when the camera moves past.

Kept for the record (not used): the photos and cut-outs in `audience/photos` and
`audience/cutouts` in the source files' folder (`chosen.json` has each one's page and licence: Wikimedia Commons files marked CC0 (Unsplash uploads) or
public domain).

Also not used: Mixamo's animations (free, but under Adobe's terms, behind a login, raw files not to be redistributed;
not needed: the procedural moves are ours); Quaternius's chibi-proportioned characters (big heads read as cartoons).

### The models and their licences

All CC0 1.0 (no attribution needed; credited in `CREDITS.md`), by **Quaternius** from Poly Pizza:
[Man](https://poly.pizza/m/HMnuH5geEG) (1,851 triangles), [Man](https://poly.pizza/m/fjHyMd5Wxw) (1,853),
[Man in Long Sleeves](https://poly.pizza/m/DLptRuewTn) (2,049), [Woman Casual](https://poly.pizza/m/jpKRgGDxhk)
(2,856), [Woman in Tank Top](https://poly.pizza/m/XqzeZGB7iU) (2,063). Rigged (a T-pose rest, the arm bones' axes
found by test and noted in `pose_people.py`), with clips (Sitting used as a base for the seated moves).

The stadium is built for the scene (Blender primitives, `stadium_build.py`): the pitch, an oval bowl of 30 stepped rows (72 segments a
ring, open at the stage end), the outer wall with three bands of lit windows, the canopy over the upper rows with its
lit edge, the stage, its towers and two trusses with the moving heads, the wall behind the screen. 24,876 triangles.

## For the music

**137.14 BPM**, the soundtrack's tempo: 5.26 frames a beat at 12 fps, a bar 1.75 s. The beat's phase against the
soundtrack is `BEAT_T0` = 0.378 s (beats 1 and 3 at `BEAT_T0` + 0.875 j). The lights change pattern every 2 bars;
the pyro fires on the first beat of every second bar; a dance move is a 2-beat cycle of 12 phases.

## On CARDIAC: what it would cost

As prototyped, a frame inside the crowd is about 370,000 triangles (160 meshes of about 2,000 triangles, 12,000
impostors, the stadium), every one of 1.18M pixels lit per pixel by about 40 lights, and 33 beams worked out per ray.
At the mesh kit's rates (about 8,000 instructions a facing triangle, 135 a pixel smooth) and the rest estimated:

| Part | As prototyped, a frame | Built (the changes below), a frame |
|---|---|---|
| Near dancers as meshes | 160 x 2,000 tris: ~1.3G | 40 x ~600 tris (decimated for the outline): ~100M |
| Impostors (place, sprite pixels) | 65k quads as triangles: ~1.0G | a sprite each (one projection, ~1,000): 65M; pixels: 60M |
| The stadium (25k tris) and the pixels' depth test | ~250M | ~250M |
| The light | per pixel, ~40 lights: ~10G | per corner (stadium) and per dancer (impostor: one value, the atlas's rim and normal scale it): ~150M |
| Beams in the haze | per ray x 33 beams: ~3G | each beam a screen-space wedge with a profile across it, only its own pixels: ~300M |
| Lasers (60 beams) | ~60M | ~60M |
| Flags (80 x 40 tris, textured), banners, 600 balloons (sprites, a sphere's normal) | ~60M | ~40M |
| Phones, confetti, city, stars (30k points) | ~20M | ~20M |
| Bloom (quarter size), the tone curve | ~75M | ~75M |
| **A frame** | **~15G** | **~1.1G** |
| **204 frames (17 s)** | **~3,000G** | **~220G** |

So as built it is heavy (about 7 s of the Mac's 28 cores for the preview, a solid share of the show's compute), but it
fits; the prototype's per-pixel version would not.

- **The animation**: pre-posed meshes per phase, as the orcas' swim phases are: 5 models x 8 moves x 12 phases (480
  poses, about 1M corners as data, or decimated about 300k); CARDIAC turns and places them. No bones on CARDIAC. The
  impostors are an asset atlas (480 sprites of 56x96, alpha and normal). Which pose each person takes each frame is a
  fetch by an index a machine computes (the wave's place, a comparison per person: ~3M a frame).
- **Depth:** the kit's depth test is 1,000 units per draw; the scene runs from 0.5 m to 900 m. Two draws, far (in
  metres) then near (in 5 cm), as the kit allows.
- **Textures:** the impostors and the screen need the kit's texture fetch (walk u, v; fetch by the punched index),
  which did not exist yet at the design stage.

## Notes

- Silhouettes are kept nearly black with a thin rim, even close; the near hands are the weakest part (low-poly
  mittens), so the camera keeps them small or brief.
- A wide lens (focal 1000 at 1536, 75 degrees across) fits the stage, the screen and the crowd together.
