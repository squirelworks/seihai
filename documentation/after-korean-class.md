# After Korean class

- Build Seihai with the real audio backend and verify sound plays:
  ```bash
  # On a machine with a sound card and libasound2-dev (or equivalent)
  cargo build --release   # audio is default
  ./target/release/seihai --sound
  ```
  Walk and swim should play the GANI cues from `<assets>/sounds`. If the device fails to open, the client still runs (null backend).
