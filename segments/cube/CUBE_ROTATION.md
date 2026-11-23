# Cube rotation: from 24 angles to 60

## Problem

The cube's turning looked clunky. The jerkiness wasn't the 96×48 resolution. The motion itself was uneven.

## Diagnosis

The vertex machine (`vtx_trig.asm`) took an angle index into a printed sine table with 24 entries (15° a step, `12 sin(15k°)`). It read cos A as sin(A + 6), mod 24. Three things made the motion uneven:

1. **A stalled frame every five.** The 3D spin's angle was `floor(4u/5) mod 24`, which moves 0.8 of a step a frame. On four frames out of five the cube turned 15°. On the fifth it didn't move. That hitch came 2.4 times a second.
2. **Coarse steps.** At 15° a step and about 144° a second, every move was a visible jump at 12 fps.
3. **The same table in 2D.** The flat rock and the speed-up spin also stepped in 15°. The speed-up (`floor(u²/160) mod 24`) moved by uneven whole steps as it gathered pace.

## Fix

- **Table.** The table now has 60 entries, `round(12 sin(6k°))`. It's still amplitude 12, so the cube is the same size.
- **Vertex machine.** Only constants change: the quarter-turn offset goes from 6 to 15, and the wrap and the table length from 24 to 60. The table is read off cards, so the machine is still 40 words of code (last cell 45), with no new cells.
- **3D spin.** `spin(u) = 2u mod 60`, exactly 12° every frame. That's the same speed as before (two full turns every 60 frames), with no stalled frames.
- **Rock.** The tips are now 18° and 30°, in 6° steps (previously 15° and 30°), with the same holds.
- **Speed-up spin.** `floor(u²/64) mod 60`, the same acceleration rescaled by 2.5. The 2 s tail after A CARDIAC PRODUCTION is `221 + floor(15(u + 1)/4)`, about 22.5° a frame. That matches the old 1.5 steps of 15°, and it carries on from where the speed-up stops.
- **Updated to match:** `cube.js` (SIN24 renamed to SIN60), `render.js`, `test_machines.js` (all 60 angles on all 3 axes), `gen_ref_data.js` / `script_data.rs`, and `long_ref.rs`.

## Cost

- **Instructions:** each vertex run reads 36 more table cards. There are 524 vertex runs in the long sequence, which went from 1267M to 1268M instructions. By hand, at 5 s an instruction, that moves the total from 200.9 to 201.1 years.
- **Cards:** about 19,000 more table cards in total.
- **Timing:** frame counts didn't change, so the cut's timing is the same.

## Verification

- Machine tests: 1738 checks passed.
- Long sequence: 1214/1214 frames identical to the native Rust reference.
- Cut: 3146/3146 frames identical to the native Rust reference.
