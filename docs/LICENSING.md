# Stella Scope: licensing

Plain language, not legal advice.

## Stella Scope

Stella Scope is open source under the **GNU Affero General Public License 3.0** (`LICENSE`), from version
0.6.3 onward as before it (0.6.0 and 0.6.1 were also AGPL-3.0). You may run it with any number of printers,
change it and share it under that licence. There is no printer limit and no licence key. The farm licence
described in some earlier notes was withdrawn before this release; the code that checked it has been
switched off (`src/licence.rs`, `allowed`) and the old key format is ignored.

Essentially all of the code is written by and for 3D3D Canada. Its Rust dependencies are permissively
licensed (MIT, Apache-2.0, ISC, BSD, Zlib, Unicode, public domain; see `docs/THIRD_PARTY.md`), which the
AGPL allows.

## What is not 3D3D's

- **3D printer models** are not in this repository or in the program. The app downloads one only when you press
  its button, from its maker's published address or, for five with none (Prusa CORE One L, CORE One, MK3S+, MINI
  under Prusa's Open Community License or GPL; a Trident R2 Stealthburner cut under GPL-3.0), from 3D3D's download
  server, with a notice that is shown first (`docs/notices/`, `docs/licences/`, `docs/PRINTER-MODELS-NOTICE.md`).
  Each model keeps its maker's licence; the AGPL does not relicense it.
- **The printer catalogue** (names, sizes, checksums, licences) is from 3D Print Rig (MIT), credited in the app.
- **The failure watch** (`eye/`) follows Obico's AGPL-3.0 scoring and runs Obico's model, loaded from a file
  you point it at.

## Contributing

Issues and pull requests are welcome. By contributing you agree your change is under the same AGPL-3.0.
