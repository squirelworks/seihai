# The 2.31 login id 46 and the early id 24

Queue item 15, from the differential testing (`documentation/difftest.md`, divergence 6: "2.31:
`ENTERLEVEL` (46) goes out before the login burst, and twice. The server then warps the player to the
online start level first. The early 46 is the likely cause, but that is not proven").

## What the real 2.31 client does

One `PLI_ENTERLEVEL` per login, inside the frame that answers the server's login burst: after the file
checks, before the process list and the props (and, with the opt-in sender, the first
`PLI_SHOWIMGPLAYER` closes the frame). It sends nothing else in its first frame but the language and the
tamper check. Lab frame, real 2.31: `UPDATEFILE x2, WANTFILE, UPDATEFILE, WANTFILE x3, ENTERLEVEL,
PROCESSLIST, PLAYERPROPS, SHOWIMGPLAYER`.

## What Seihai did, and the cause

Seihai sent the 46 in its first frame, at t = 0 (and, in the parity runs, a second one after the extra
warp below). The 46 was **not** the cause of the extra warp. Four runs on the 2.31 `login` scenario, same
lab, only the senders changed:

| Seihai sends | extra warp to `onlinestartlocal.nw` | `ENTERLEVEL` count |
|---|---|---|
| default senders (46 at t = 0) | no | 1 |
| `--send-tampercheck` (46 held to the post-login frame) | no | 1 |
| `--send-showimg` (46 held) | **yes**, 30 ms in | 2 |
| both (`seihai-parity`, 46 held) | **yes**, 24 ms in | 2 |

The cause is the opt-in `SHOWIMGPLAYER` (id 24): Seihai's sender treated the first tick after login as a
wrap and sent it at about 20 ms, before the server's login burst; the real client sends its first one at the
end of the post-login frame (89 ms). The server answers an id 24 that early by warping the player to its
online start level and then to the account's level; the second level arrival is what made Seihai send a
second 46. So the duplicate 46 is a consequence, and the early 46 is only a bad frame position.

## Change

* The 2.31 login 46 is held for the post-login frame (`EnterLevelSender::owe_login` / `take_login`) and goes
  after the file checks and before the `gr` flags and the process list, once. A server that never ends
  its burst with `PLO_LISTPROCESSES` gets it alone after 3 s. Builds without the post-login frame keep the
  old behaviour.
* The opt-in `PLI_SHOWIMGPLAYER` sender waits for the post-login frame (up to the same 3 s) and its first
  message ends that frame, like the real client's.

## Before and after (lab, 2.31, `seihai-parity` = both opt-in senders on; real client for reference)

| scenario | `PLAYERWARP`s real / before / after | `ENTERLEVEL`s real / before / after | divergences high-medium-low before | after |
|---|---|---|---|---|
| `login` | 1 / 2 / 1 | 1 / 2 / 1 | 3-10-21 | 0-4-2 |
| `chat` | 1 / 2 / 1 | 1 / 2 / 1 | 2-11-21 | 0-4-2 |
| `gmap_cross` | 0 / 1 / 0 | 5 / 6 / 5 | 2-13-23 | 0-9-12 |

The login frame's message order is now the real one. With the default senders the frame is the same minus
the opt-in id 24; the report still lists the process-list text and `UDPPORT` as before.
