<!-- Thank you for the pull request. Please read CONTRIBUTING.md if you have not. Delete any section that does not apply. -->

## What does this change, and why?

<!-- The problem first, then what you changed. Link the issue: Fixes #123 -->

## How did you test it?

<!-- Commands you ran, and what you saw. -->

- [ ] `cargo fmt --check`
- [ ] `cargo test --locked -j 2`
- [ ] interface tests (`for t in scripts/*-tests.mjs; do node "$t" || break; done`)
- [ ] smoke tests, if you changed the server or a driver
- [ ] added or updated a test that fails without this change (or explained below why not)

## Which printers and firmware did you test on?

<!-- Model and firmware (and Klipper/Moonraker versions). Or "demo and fakes only". Say what you did NOT test. -->

## Does it touch what a printer is told to do?

<!-- Tick one. If you add or loosen a command, read "Rules for anything that touches a real printer" in CONTRIBUTING.md. -->

- [ ] No. It only reads, records, shows or documents.
- [ ] Yes. It adds or changes a command, a limit, or a safety check (describe below).

## Screenshots

<!-- For anything people see: desktop width and about 390 px wide. Before and after if it changes something that exists. Remove addresses, names and keys. -->

## Checklist

- [ ] One change in this pull request.
- [ ] No printer model files, secrets, recordings or private addresses added.
- [ ] New dependencies (if any) are justified here and their licences are AGPL-compatible.
- [ ] The user guide is updated if behaviour changed.
- [ ] I wrote this or have the right to submit it, and I agree it is licensed under the AGPL-3.0.
