Card Attack
Strange Query, Demosplash 2026

Every pixel and every sample of sound is computed live on CARDIACs, the 1968 Bell Labs
cardboard computer, emulated by HEARTBURN on every core. card-attack.nfo, beside this
file, says how it works and who made it.

Requirements
  macOS on Apple Silicon. Uses all cores but two, and about 30 GB of memory at its
  peak. Tested only on a Mac Studio (M5 Ultra, 30 cores, 96 GB); on a smaller Mac
  it may not keep up.

If macOS won't open it
  card-attack isn't signed by an identified developer, so if the zip was downloaded
  in a browser, macOS may refuse to run it. In Terminal, in this folder, run:

      xattr -dr com.apple.quarantine .

  Or, after the refusal, open System Settings > Privacy & Security and click
  Open Anyway next to card-attack.

Running it
  Open Terminal in this folder and run:

      ./card-attack play

  The film plays fullscreen with sound. Escape quits. Sound goes to the Mac's current
  output device (System Settings > Sound > Output), so set that first.

Options
  --windowed          plays in a window
  --audio-offset MS   delays the sound by MS milliseconds, for a projector that shows
                      pictures late
  --mute              plays without sound
  --dry-run           checks the load without opening a window
