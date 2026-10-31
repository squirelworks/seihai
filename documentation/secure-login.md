# Seihai secure login (version token `SEI00001`)

Status: revision 1, 2026-10-31. Seihai's own protocol, not part of any Graal
build. Real clients and servers never send or expect it; nothing in this
document comes from a decompile or from GServer-v2.

## Why

Every classic login sends the account and the password in the clear: the
gserver login line (`type key token GSTRING(account) GSTRING(password)`) and
the listserver's id 1 are only zlib-compressed, and the gen-3 and gen-5
"ciphers" are keyed with a byte that rides in the same frame. Anyone on the
path reads the password.

`SEI00001` is a version token in the same place and of the same length as
`GNW03014` and the others. A client that sends it runs a key exchange first
and sends the account and password sealed. Older tokens are untouched: a
server that does not know `SEI00001` refuses it like any unknown version,
and a server that knows it still takes every classic login as before.

## Goals and limits

- The account, the password and the rest of the classic login tail are
  secret and tamper-proof in transit, against passive listeners and, once
  the server's key is pinned, against an active man in the middle.
- Fresh keys per connection (forward secrecy): a stolen server key does not
  open recorded logins.
- No change to the session after the login. The game traffic is still the
  inner build's gen-3/gen-5 stream, which is not secret. Encrypting the whole
  session is a possible later revision (`SEI00002`), not this one.
- The server still sees the password (it checks it against its argon2 hash).
  A PAKE such as OPAQUE would hide it from the server too; that is a larger
  change, noted under "Later".
- First contact is trust on first use (TOFU), as with SSH, unless the player
  gives the key (`--server-key`). Admins should publish their key fingerprint.

## Primitives

| Use | Primitive |
|---|---|
| Key exchange | X25519, one fresh key per side per connection |
| Server identity | Ed25519 long-term key, strict verification |
| Transcript hash | SHA-256 |
| Key derivation | HKDF-SHA256 |
| Sealing | ChaCha20-Poly1305 (IETF, 96-bit nonce) |
| Text encoding | base64, standard alphabet, no padding |

Base64 has no line feed and no byte below 0x20, so every handshake message is
one ordinary Graal message line.

## Messages

Every handshake message is one line (ending `\n`) in a **plain zlib frame**
(`u16be length` + zlib body), as every classic login frame is.

```
C1  prefix "SEI00001" inner[8] b64(client_x25519_pub)[43] suffix
S1  "SEI00001" b64(server_x25519_pub)[43] b64(server_ed25519_pub)[43] b64(signature)[86]
C2  "SEI00001" b64(ciphertext)
```

### C1, client hello

| Link | prefix | suffix |
|---|---|---|
| gserver | client type GCHAR, key GCHAR (gen 2: type only) | empty |
| listserver | id 7 GCHAR (`'`), key GCHAR | the list name (`newmain`) |

`inner` is the eight-byte token of the build the session then speaks as:
the game token for a gserver (`GNW28015` for 2.31), the list token for a
listserver (`GNW30123`). The prefix and the key byte are those that build's
classic login would send.

### S1, server hello

The server makes a fresh X25519 key and signs, with its Ed25519 key:

```
signed = "seihai-login-v1/server-hello\0"
       || u16le(len(C1 line without \n)) || C1 line without \n
       || server_x25519_pub || server_ed25519_pub
```

Signing all of C1 binds the client type, key byte, inner token and list
name: a man in the middle cannot downgrade the inner build.

A server that refuses (unknown inner token, banned address, secure login
turned off) sends what it would send a classic login it refuses instead of
S1, in a plain zlib frame, then closes: `PLO_DISCMESSAGE` (16) from a
gserver, id 4 (error text) from a listserver. The client shows that line.

### Key schedule

```
shared = X25519(own_secret, peer_pub)       -- all-zero result: abort
th     = SHA-256(signed || signature)
key    = HKDF-SHA256(salt = th, ikm = shared, info = "seihai-login-v1/c2s key", L = 32)
```

### Client checks

1. S1 starts with the token and has exactly 43 + 43 + 86 base64 characters.
2. The signature verifies (strict Ed25519) under `server_ed25519_pub`.
3. `server_ed25519_pub` is the pinned key for this server (see "Pinning").
   On a mismatch the client stops and **sends nothing more**.

### C2, sealed login

```
plaintext  = u16le(len(tail)) || tail || zero bytes up to a multiple of 64
ciphertext = ChaCha20-Poly1305(key, nonce = 12 zero bytes, aad = th, plaintext)
```

The key is used once, so the zero nonce is safe. Padding to 64 bytes hides
the password length to that granularity. `tail` is at most 1024 bytes.

| Link | tail |
|---|---|
| gserver | everything the classic login line has after the token: GSTRING account, GSTRING password and, for 5.x/6.x, the platform fields |
| listserver | id 1's payload: GSTRING account, GSTRING password |

On a gserver, C2's frame stands in for the classic login frame: messages the
inner build bundles with its login (2.17.1's language and tamper check,
3.04's file checks) follow the C2 line in the same frame, encoded exactly as
they would follow the classic login line (gen-3 insertion from the first
iterator step).

The server opens C2; a failure to open closes the link with no answer
(it is either an attack or a broken client). Padding must be zero.

### After C2

Both sides continue as after the inner build's classic login frame. The
gen-3 or gen-5 cipher starts from its seed with the key byte from C1, in
both directions. The gserver checks the account and password and sends the
login burst or its usual refusal; the listserver answers as it answers id 1
(gen-5 frames: list, then status, site URL and upgrade URL; or id 4).

On a listserver the client sends no gen-5 frame of its own before the
reply: id 1 went in C2. Both iterators are at the seed when the reply comes.

## Pinning

The client keeps `known_servers.txt` in its settings folder, one line per
server: `host:port base64(server_ed25519_pub)`. `#` lines are comments.

- No line for the server: trust the key, append it, and say so on standard
  error with the fingerprint.
- Same key: go on.
- Different key: refuse, name both fingerprints and the file. The player
  removes the line by hand if the admin changed the key on purpose.
- `--server-key KEY` trusts that key alone and leaves the file alone.

Fingerprint format: `SHA256:` + base64 (no padding) of SHA-256 of the public
key, as servers print it at start.

## Server key

The server keeps its Ed25519 seed in a file, one line of base64 (no
padding), mode 0600 on Unix, created on first start:
`seihai-login.key` beside the config by default. Servers print the public
key and its fingerprint at start. Losing the file means every player sees a
changed key; back it up.

## Version handling on servers

- The token is checked where the version token is read. Anything else goes
  the classic way.
- The inner token goes through the server's normal allow-list. `SEI00001`
  itself is not a client version and never appears in a player's version
  prop or in logs as the client version; log it as `SEI00001 (inner X)`.
- A server may require the secure login (refuse classic logins with a
  password) as a setting; off by default so real clients keep working.

## Implementations

| Where | What |
|---|---|
| `seihai` `crates/seihai-core/src/protocol/seclogin.rs` | Messages, crypto, pins, server key file: one definition for client and servers |
| `seihai` `crates/seihai-core/src/net/mod.rs` | `login_gserver_secure`, `Connection::login_secure_with`, `fetch_server_list_secure` |
| `seihai` `seihai-mae` | `--secure-login`, `--server-key KEY` |
| `seihai-listserver` | Secure id 7 on every client port |
| `seihai-gserver` | Login accept module (classic and secure); the rest of stage 1 to come |
| `seihai-gserver-v2` (GPL fork) | Not yet. Write it from this document with libsodium or OpenSSL, not from the Rust code, so the licences stay apart |

## Test vectors

Every message has a random part, so there are no fixed vectors. To check
an implementation, run it against the Rust side on loopback: the seihai-core
tests `protocol::seclogin::tests` and `net::tests::secure_*` and
seihai-listserver's `tests/secure_login.rs` show each step. A deterministic
vector set (fixed X25519 and Ed25519 keys) is a TODO for the fork's tests.

## Later

- `SEI00002`: keep the AEAD keys and encrypt the whole session (each frame
  sealed with a counter nonce) in place of gen-3/gen-5.
- OPAQUE (RFC 9807) so the server never sees the password.
- Gserver to listserver account checks (`VERIACC2`) still carry the
  password in the clear on that link; secure that link too, or have the
  gserver check accounts itself.
- RC logins (`seihai-rc`) through the same handshake.
- The GUI login screen: a "secure login" tick box and the fingerprint prompt.
