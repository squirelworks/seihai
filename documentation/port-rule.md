## Port rule (by login type; updated after the matrix of 2026-10-25)

14921 is dead. The public service resets it. Do not use it as a default for any version.

The public listserver chooses its reply by the **login type** the client sends, not by the version number or the token. The user ran every profile against both ports (19 profiles, ports 14922 and 14923, one list-only login each, 38 runs, 2026-10-25 22:43 JST). Rule for `listserver.graal.in`:

- A profile that sends the **classic login** (one plain zlib frame: id 0 `newmain`, and for most profiles then the id 1 account and password; the real 2.00 and 2.05 send id 0 only, see `listserver-login.md`) defaults to **14923**. This is 1.41, 1.42, 2.00 to 2.04 (with 2.02 rev 2), 2.05, 2.1, 2.12 to 2.17. `kind` is `legacy`.
- A profile that sends the **gen-5 login** (id 7 with `GNW30123`, then a gen-5 frame) defaults to **14922**. This is 2.22, 2.3 and 2.31. `kind` is `supported`. 14923 answers it too.
- Host is always `listserver.graal.in`.

The login a profile sends is decided in `Connection::login_listserver`: gen-5 only where the profile's encryption is gen 5 (2.22 and later), classic for everything else. A unit test keeps `kind` and the encryption in step, so a profile cannot get a default port its login does not work on.

What the matrix showed:

| login | profiles | port 14922 | port 14923 |
|---|---|---|---|
| classic | 1.41, 1.42, 2.00, 2.01, 2.02, 2.02 rev 2, 2.03, 2.04, 2.1, 2.12, 2.13, 2.14, 2.15, 2.16, 2.17 (15) | connected, **silent** for 5.5 s, no reply, in all 15 | a **legacy list** (plain zlib id 0, 2 or 4 rows, no status or site lines) in all 15 |
| gen-5 | 2.22, 2.3, 2.31 (and the matrix binary's 2.05, see below) | the modern list: 8 rows plus status, site and upgrade lines | the modern list too: the legacy list first, then the same frames (12 rows, 2.31: 17) |

The port the binary carries is a historical fact. It is kept as `built_in_port` in the TOML and `Listserver::built_in` in the code for the record; it is not the default Seihai dials. The default is the port that gets a list. Those history values are unchanged.

## What "confirmed" means here

`status` is `confirmed` where the matrix ran Seihai's own list login for that profile on that port and got a list. It does **not** mean a real client of that version was captured: none was, and 2.1 to 2.16 have no binary at all. For these profiles it says "the login Seihai sends for this profile gets a list on this port".

- Confirmed: 1.41, 2.1, 2.12 to 2.17 (on 14923) and 2.22, 2.3, 2.31 (on 14922). The matrix ran Seihai's older id 0 + id 1 login with the `78 9C` zlib header; Seihai now sends `78 DA`, which has not been run.
- **Assumed since the login changed: 1.42, 2.00 to 2.04 (with 2.02 rev 2) and 2.05.** They now send the real 2.00/2.05 id-0-only login (shape A); the matrix confirmed their port only for the older login. Nothing is claimed for shape A until a run.
- **Assumed: 2.05.** The matrix binary sent the gen-5 login for the 2.05 profile (a 28-byte frame like 2.22's) and both ports answered it. The current main sends the classic login for 2.05 (checked on a local mock; `net/mod.rs` test `classic_listserver_login`), and that login was not run for 2.05. So 2.05 takes the classic-login port, 14923, and is marked assumed; 14923 answered the classic login of every other classic profile. Running the matrix again with a build of this change would settle it (`VERSIONS=2.05 ./run_port_matrix.sh ...`).

The launcher port box must follow the selected version's default. When the user changes the version, the port box updates to that version's `listserver_port` unless the user has typed over it. Saved and typed ports stay. This changes the box for 2.05 and 2.1 to 2.17: it now shows 14923 (it showed 14922).

## Where it is implemented

- `Listserver { host, port, kind, status, built_in, token }` in `client/src/protocol/profile.rs`; the chain steps in `chain.rs` set it; `ClientVersion::listserver_host/port/addr` read it.
- The launcher (`ListEndpoint` in `client/src/gui.rs`): the port box shows the version's default and follows the version selector. The line under it says the login type and whether the port is confirmed or assumed (hover for what that means). A typed port, or a port in the settings file, is kept and marked user-set ("Use default" gives it back). The settings file gets a port only when the user set one.
- `--online host:port` is used as given. A bare `--online listserver.graal.in` gets the version's default port. 1.33 to 1.39 have no listserver and borrow 2.05's (classic login, 14923).
- Tests: no profile defaults to 14921, every host is `listserver.graal.in`, `login_type_decides_the_default_port` (classic -> 14923, gen-5 -> 14922, kind agrees with the login the code sends), `port_status_confirmed_versus_assumed`, the fixture agrees with the Graal_analysis profiles, and the launcher follow tests.
- Evidence: `protocol/listserver-login-type-matrix-2026-10-25.md` and `client-internals/version-profiles/list-ports.md` in Graal_analysis.
