# The stadium audience

The stadium party's crowd: 65,536 people dancing at night in front of the stage and the giant screen, 32,768 of them
drawn in the film. This page covers how the crowd is made and why: what makes a crowd read at this size, the models
and their licences, the photographs tried against them, the beat they dance to, and the cost estimate that shaped
the drawing. The segment itself, how it runs on CARDIAC and its measured numbers are in
[`segments/stadium`](../../segments/stadium/README.md).

## The people

People are the hardest thing in the scene to make convincing, so the crowd keeps to what reads:

1. **Always from behind or far off, as silhouettes with a thin rim of light**: almost black (albedo 0.025), a rim
   where the surface turns away from the camera, lit by the screen, the LED walls and the beams ahead of them. No
   faces, no front views up close; hands are never large for more than a moment.
2. **Motion is what sells them.** Five rigged CC0 characters by Quaternius (below), posed in Blender by procedural
   dance moves on their own rigs: **jump** (arms up, a jump on every beat), **cheer** (both arms up, pumping),
   **raise** (one arm up holding a phone, can or lighter, bobbing), **pump** (a fist pumped on the beat), **sway**
   (arms overhead, side to side), **bob**, **dance** (fists by the shoulders, alternating, the body twisting) and
   **point** (at the screen). Each move is a 2-beat cycle of 12 phases; each person keeps the beat, give or take a
   frame. The waves: on the pitch, a band of arms going up rolls from the stage to the back every 4 bars; on the
   tiers, a Mexican wave goes round the bowl.
3. **Crowds as instances**: the 40 nearest in view within 26 m are drawn as the posed meshes; everyone else is an
   impostor, the same pose rendered ahead of time into an atlas (from behind and 14 degrees above, 56x96, with its
   normals, and a version three times as fine for the nearer ones), set upright and turned to the camera, lit per
   pixel by the same light, so the switch doesn't show. Variation: the model (5), the move, the phase, the height
   (0.93-1.06), the clothes' darkness, the facing (+-10 degrees). The clothes are dark solid colours.

**Models against photographs.** The same frame was tried with the near crowd as photo cut-outs (nine people
photographed from behind, public domain or CC0, cut out with macOS Vision's subject lifting, darkened to silhouettes
with the rim on their outline's inside edge, bobbing on the beat) against the posed models. The models won, clearly
at 384x192:

- The photos are of people standing still (watching geese, a station board, a football match): heads and shoulders
  with their arms down. A party's silhouette is the arms in the air; static photos can't do that, and none of the
  free back-view photos found shows anyone with arms up (the one with both arms up is holding game controllers).
- They can't move except bobbing; the models jump, pump and sway on the beat, and at 384x192 the motion is what reads.
- Up close their heads and hoods do look more real, but as black silhouettes the difference is small; and a flat
  photo turned to the camera shows when the camera moves past.

The photos and cut-outs are kept with the source files, not in this repository (`audience/photos` and
`audience/cutouts`; `chosen.json` has each one's page and licence: Wikimedia Commons files marked CC0 (Unsplash
uploads) or public domain).

Also not used: Mixamo's animations (free, but under Adobe's terms, behind a login, raw files not to be redistributed;
not needed, since the procedural moves are the project's own); Quaternius's chibi-proportioned characters (big heads
read as cartoons).

## The models and their licences

All CC0 1.0 (no attribution needed; credited in `CREDITS.md`), by **Quaternius** from Poly Pizza:
[Man](https://poly.pizza/m/HMnuH5geEG) (1,851 triangles), [Man](https://poly.pizza/m/fjHyMd5Wxw) (1,853),
[Man in Long Sleeves](https://poly.pizza/m/DLptRuewTn) (2,049), [Woman Casual](https://poly.pizza/m/jpKRgGDxhk)
(2,856), [Woman in Tank Top](https://poly.pizza/m/XqzeZGB7iU) (2,063). They come rigged, with a T-pose rest, and the
moves are posed on those rigs.

The stadium is built for the scene from Blender primitives: the pitch, an oval bowl of 30 stepped rows (72 segments
a ring, open at the stage end), the outer wall with three bands of lit windows, the canopy over the upper rows with
its lit edge, the stage, its towers and two trusses with the moving heads, the wall behind the screen. 24,876
triangles.

## The beat

**137.14 BPM**, the soundtrack's tempo: 5.26 frames a beat at 12 fps, a bar 1.75 s. The beat's phase against the
soundtrack is `BEAT_T0` = 0.378 s (beats 1 and 3 at `BEAT_T0` + 0.875 j). A dance move is a 2-beat cycle of 12
phases, so the whole crowd's motion runs on that clock.

## What it would cost on CARDIAC

Drawn in the straightforward way, a frame inside the crowd is about 370,000 triangles (160 meshes of about 2,000
triangles, 12,000 impostors, the stadium), every one of 1.18M pixels lit per pixel by about 40 lights, and 33 beams
worked out per ray. The estimate below, made before the segment was built at the mesh kit's rates (about 8,000
instructions a facing triangle, 135 a pixel smooth) and the rest guessed, is what decided how the crowd is drawn.
The segment's measured numbers are in its README.

| Part | Straightforward, a frame | With the changes, a frame |
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

With the changes it is heavy but fits; lit per pixel by every light it would not.

- **The animation**: pre-posed meshes per phase, as the orcas' swim phases are: 5 models x 8 moves x 12 phases (480
  poses) as data; CARDIAC turns and places them. No bones on CARDIAC. The impostors are an asset atlas (480 sprites
  of 56x96, coverage and normal). Which pose each person takes each frame is a fetch by an index a machine computes
  (the wave's place, a comparison per person).
- **Depth:** the scene runs from 15 cm to about 900 m, so depths are kept as K / z.
- **Textures:** the impostors and the screens are fetched by the place walked across them.

## Notes

- Silhouettes are kept nearly black with a thin rim, even close; the near hands are the weakest part (low-poly
  mittens), so the camera keeps them small or brief.
- A wide lens (focal 1000 at 1536, 75 degrees across) fits the stage, the screen and the crowd together.
