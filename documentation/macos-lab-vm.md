# A macOS 15 lab VM for reproducing macOS CI failures

The suite's macOS jobs are mostly run on a lab VM rather than on GitHub's
runners. This note is how to set one up on any Linux host with KVM, so a
failure seen on a macOS runner (or reported by a macOS user) can be
reproduced locally.

Running macOS on non-Apple hardware is outside Apple's licence terms. Use
this only where that is acceptable to you.

## Host requirements

- An x86-64 Intel or AMD CPU with KVM (`/dev/kvm`). Intel is the tested path.
- QEMU 8 or newer, with OVMF. No libvirt is needed: one shell script starts
  the VM.
- 6 GiB of RAM for the guest (4 GiB works for builds but is slow), 4 vCPUs,
  and about 100 GiB of thin-provisioned disk (the guest uses 10 to 60 GiB).

## 1. Install macOS 15

1. Clone OSX-KVM (`github.com/kholia/OSX-KVM`). It provides OpenCore, the
   OVMF firmware and `fetch-macOS-v2.py`.
2. `./fetch-macOS-v2.py -s sequoia` downloads the recovery image. Convert
   it with `qemu-img convert BaseSystem.dmg -O raw BaseSystem.img`.
3. `qemu-img create -f qcow2 macos.qcow2 100G`.
4. Start the VM with the recovery image attached. A working QEMU line:
   `-machine q35 -cpu Skylake-Client,-hle,-rtm,kvm=on,vendor=GenuineIntel,+invtsc,vmware-cpuid-freq=on,+ssse3,+sse4.2,+popcnt,+avx,+aes,+xsave,+xsaveopt,check -smp 4,cores=2 -m 6144`,
   the two OVMF pflash drives (the vars file writable, a copy per VM),
   `isa-applesmc` with the usual OSK string, the OpenCore qcow2 on SATA
   (open it with `snapshot=on` so it is never written), the system disk on
   SATA, a USB keyboard and tablet on `qemu-xhci`, `vmware-svga`, and
   `-display none -vnc 127.0.0.1:N` for a headless host.
5. **Network:** user-mode networking with an SSH forward,
   `-netdev user,id=n0,hostfwd=tcp:127.0.0.1:<port>-:22 -device virtio-net-pci,netdev=n0`.
   Keep the forward on loopback.
6. Recovery's "Reinstall macOS" downloads from Apple's CDN very slowly
   through QEMU's user network (about 130 KiB/s, and it can fail). It is
   much faster to download the full `InstallAssistant.pkg` for the
   release on the host (from Apple's software catalog), serve it to the
   guest over HTTP on `10.0.2.2`, and in Recovery's Terminal run
   `pkgutil --expand-full`, move the pkg into the app's
   `Contents/SharedSupport/SharedSupport.dmg`, then
   `startosinstall --agreetolicense --volume /Volumes/<disk>`.
7. The OpenCore picker has no timeout and stops on "EFI": press Right,
   then Return, about 20 s after power-on (earlier key presses are lost).
   Over QMP this is `send-key right` then `send-key ret`.

### Set up the account for lab use at install time

Decide this in Setup Assistant; it is hard to change later over SSH:

- Make one admin user for builds, with a simple lab-only password, and
  **turn on automatic login** for it (System Settings > Users & Groups).
  Without a logged-in GUI user no app can open a window, and
  `screencapture` over SSH fails ("could not create image from display").
- Leave FileVault off (automatic login needs it off).
- The first user gets the volume's Secure Token, and macOS then refuses a
  root password reset without the old password. Don't lose that password.
- Turn off sleep and automatic updates (download-only is fine), enable
  Remote Login (SSH) with a key, and give the build user passwordless
  `sudo` via a file in `/etc/sudoers.d/`.

## 2. Toolchains

Homebrew no longer installs on Intel Macs ("only supported on Apple
Silicon"), so everything below comes from upstream installers or source.

| Tool | Source | Notes |
|---|---|---|
| Command Line Tools | `softwareupdate --list` then `softwareupdate -i "<label>"` (no Apple Account needed) | clang, git, make, Python 3.9 |
| Rust | rustup (`--profile minimal`), then `rustup component add rustfmt clippy` | |
| Python 3.12 | python.org universal2 `.pkg` (3.12.10 is the last 3.12 with an installer) | some tools need 3.12 syntax |
| CMake | Kitware's universal `.dmg`/`.tar.gz` from GitHub releases, e.g. into `/opt/cmake` | |
| vcpkg | `git clone` + `./bootstrap-vcpkg.sh -disableMetrics`; set `VCPKG_ROOT` | triplet `x64-osx` |
| Ninja | `vcpkg fetch ninja` (or the GitHub release binary) | |
| pkg-config | pkgconf from source (`Makefile.lite`) | vcpkg needs a system one |
| GNU Bison ≥ 3.4 | ftp.gnu.org source | macOS ships 2.3 |
| GNU m4, autoconf, automake, libtool, autoconf-archive | ftp.gnu.org sources (check the GPG signatures) into a private prefix | vcpkg's libsodium port runs autoreconf |
| libsodium | release tarball (minisign-verified), `./configure --prefix=/usr/local` | for builds that use the system copy |
| Java | Temurin 21 JDK `.tar.gz` (Adoptium) | ANTLR for the C++ server; `javac` for the Java clients |
| Free Pascal 3.2.2 | the official `x86_64-darwin` `.dmg` | `fpc` lands in `/usr/local/bin`; non-login SSH shells need that on `PATH` |

Put the extra tools on `PATH` in `~/.zprofile` **and** in your CI driver: a
non-interactive `ssh host 'cmd'` runs zsh without reading `.zprofile`.
Note that zsh is the login shell: `read -p` means something else there, and
unquoted variables are not word-split (`${=VAR}` splits).

## 3. Reproducing a CI job

1. Clone the repo fresh from GitHub in the guest (not a local mirror), at
   the commit the runner used.
2. Run the job's steps by hand, or with a small driver that runs each step
   in order, logs it between `=== STEP name START/END rc=` markers and
   records `repo step rc seconds`. Translate `brew install …` lines to the
   tools above; replace Homebrew's `llvm`/`lld` with Apple clang and ld
   unless the failure is specific to them.
3. Copy logs out with `scp`. To reach a server running in the guest from
   the host, use `ssh -N -L <port>:127.0.0.1:<port> …`.

## 4. Running clients in the VM

- The guest has full internet by default. **Before starting any game
  client, restart the VM with a restricted network**
  (`-netdev user,…,restrict=on` with only the SSH forward, or `-nic none`),
  so a client cannot reach a public server on its own. Run the lab
  servers inside the guest.
- Start GUI programs in the logged-in user's session, e.g.
  `sudo launchctl asuser $(id -u <user>) sudo -u <user> open -a …`, and
  check them with `screencapture -x` over SSH or a screenshot over QMP.
  Keyboard input for a test can be sent with `osascript -e 'tell
  application "System Events" to keystroke …'` (needs Accessibility
  permission for the sending process) or with QMP `send-key`.

## 5. Gotchas seen so far

- macOS has no fully static executables: anything linked with `-static`
  fails with `ld: library not found for -lcrt0.o`.
- ld64 pulls whole objects out of static libraries differently from GNU ld,
  so a symbol that is only defined in a program's `main.cpp` can be missing
  when a test links the library alone.
- Timing-sensitive tests are slower and noisier in the VM; rerun a single
  failing test a few times before calling it a real failure.
- `vmware-svga` gives the guest no Metal device
  (`MTLCreateSystemDefaultDevice()` returns nil). Anything that needs Metal,
  such as minifb windows (Mae), cannot open a window in this VM. Treat those
  as "blocked by the VM", not as bugs, and test them on real Mac hardware.
- `screencapture` run over SSH returns a blank white frame until the
  calling process has Screen Recording permission. A QMP `screendump` of
  the framebuffer always works.
- macOS ships BSD `sed`: `sed -i "s/…/…/" file` reads the script as the
  backup suffix and fails with `invalid command code`. Use `sed -i.bak` or
  write to a temp file in scripts that must run on both.
- Apple clang warns about things GCC does not, e.g.
  `-Wunused-private-field`, so code built with `-Werror` can fail only on
  macOS.
- Budget host RAM: the guest plus any other VM must leave the host enough
  headroom, or the host's OOM killer can take QEMU down mid-run.
