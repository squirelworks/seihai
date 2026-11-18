# Seihai's copy of minifb 0.27.0

This is minifb 0.27.0 from crates.io (MIT OR Apache-2.0, see LICENSE*),
used through `[patch.crates-io]` in the workspace `Cargo.toml`. The
examples, CI files and lock file were left out. One change:

**macOS without Metal.** Upstream `mfb_open` returns no window when
`MTLCreateSystemDefaultDevice()` gives nil ("Your device/OS doesn't support
Metal."), which is the case in a macOS VM with an emulated display
(QEMU vmware-svga and similar). Now it opens the window anyway with
`MiniFBCGView`, an `NSView` that draws the last buffer with CoreGraphics
(`CGImage` from the 0RGB words, nearest-neighbour, same scale modes as the
Metal path). Set `MINIFB_NO_METAL=1` to take that path on a Mac that has
Metal. Files: `src/native/macosx/MacMiniFB.m`, `OSXWindowFrameView.h`,
`OSXWindowFrameView.m`.

Drop this copy when upstream minifb has a non-Metal path.
