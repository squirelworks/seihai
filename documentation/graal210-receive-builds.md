# Explicit Windows 2.1 receive builds

Mae can opt into one statically observed receive dispatcher while retaining
the generic 2.1 baseline for login, outgoing packets and script semantics:

```sh
seihai --version 2.1 --receive-build public-2001-11-13 --online 127.0.0.1:14803
```

The exact labels are `public-2001-11-13`, `ver7`, `ver8`, `ver9`, `ver10`,
`ver13` and `v22`. The option requires an explicitly given `--version 2.1`;
it is not inferred from saved settings, a login token or a FileVersion.
Without the option, the existing generic profile is unchanged. The selection
is applied again when the client creates state for another server.
It is not saved when returning to the start window or starting a new process.

This is a partial receive override, **not a complete online build profile**.
In particular, the baseline login token and send rules remain generic 2.1.
The early world builds have different observed tokens; selecting their
receive tables does not make the generic login correct for those builds.

The public build has the verified 170–174 reader details, including the
two-field map descriptor. World-build selections change receive bindings
only; they do not inherit the public payload layouts. The v22 bindings for
175–177 do not establish complete shot or mode behavior. No local
`controlnpc.txt` loading, v22 `loadgmaps.txt` preload, native resource lookup,
or script execution is added by this option.

Source: the exact-build static study in
[Graal_analysis by squirelworks](https://github.com/squirelworks/Graal_analysis/blob/bb0a330816e0f71be4df6ca509937be8a5461908/client-internals/graal2002-and-140-builds.md),
under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).
Synthetic consumer regressions verify selection and decoding; they do not
constitute original-client or server acceptance.
