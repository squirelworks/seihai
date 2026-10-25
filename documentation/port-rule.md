## Port rule (correction to #70)

14921 is dead. The public service resets it. Do not use it as a default for any version.

Rule for `listserver.graal.in`:

- **Legacy** (classic login, versions that the current service only answers on the legacy port): default port **14923**. This is 2.00 and any earlier build that has a listserver.
- **Supported** (gen-5 login, versions the current service answers on the current port): default port **14922**. This is 2.05 and later (2.17, 2.22, 2.31).
- Host is always `listserver.graal.in`.

The port the binary carries is a historical fact. Keep it labelled `built_in_port` in the TOML if you want the record, but it is not the default Seihai dials. The default is the port that gets a list.

The launcher port box must follow the selected version's default. Today it hardcodes 14922 unless the settings file already has a port. When the user changes the version, the port box should update to that version's `listserver_port` unless the user has typed over it.

2.01–2.04 have not been run against the public service. Treat them as legacy (14923) until a check shows otherwise. Do not leave them on 14921.
