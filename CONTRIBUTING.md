# Contributing to Stella Scope

Thank you for wanting to help. Stella Scope is a small project run by [3D3D](https://3d3d.ca), and it is better for every bug report, fix and idea that comes in. This guide explains exactly how we would like contributions to arrive, so that your time is not wasted and ours is not either.

If you only read one section, read [The short version](#the-short-version).

## Contents

1. [The short version](#the-short-version)
2. [Ground rules](#ground-rules)
3. [Ways to contribute](#ways-to-contribute)
4. [Reporting a bug](#reporting-a-bug)
5. [Suggesting a feature](#suggesting-a-feature)
6. [Asking a question](#asking-a-question)
7. [Reporting a security problem](#reporting-a-security-problem)
8. [Getting set up](#getting-set-up)
9. [Building](#building)
10. [Testing](#testing)
11. [How the code is organised](#how-the-code-is-organised)
12. [Code and writing style](#code-and-writing-style)
13. [Rules for anything that touches a real printer](#rules-for-anything-that-touches-a-real-printer)
14. [Adding support for a printer](#adding-support-for-a-printer)
15. [3D models](#3d-models)
16. [Dependencies](#dependencies)
17. [Changing the web interface](#changing-the-web-interface)
18. [Documentation](#documentation)
19. [Submitting a pull request, step by step](#submitting-a-pull-request-step-by-step)
20. [Commit messages and the pull request title](#commit-messages-and-the-pull-request-title)
21. [What makes a pull request easy to review](#what-makes-a-pull-request-easy-to-review)
22. [What happens after you open it](#what-happens-after-you-open-it)
23. [Using AI tools](#using-ai-tools)
24. [Licence of your contribution](#licence-of-your-contribution)
25. [Conduct](#conduct)

## The short version

1. **Search first.** Look through [open and closed issues](https://github.com/3d3dcanada/stella-scope-community/issues) and pull requests before you file anything.
2. **Open an issue before a big change.** A bug fix of a few lines can go straight to a pull request. Anything larger (a new feature, a refactor, a new printer, a change to how something looks or behaves) starts as an issue, so we can agree on the approach before you write the code.
3. **Fork, branch from `main`, make one change.** One pull request does one thing.
4. **Check your work locally.** `cargo fmt`, `cargo test --locked -j 2`, and the interface tests (see [Testing](#testing)). If you changed something a person sees, look at it in a browser and include a screenshot.
5. **Open the pull request and fill in the template.** CI must be green.
6. **Expect a review.** We will ask questions and often ask for changes. That is normal and it is not a rejection.

## Ground rules

These apply to everything below.

- **Be kind and be specific.** Read the [code of conduct](CODE_OF_CONDUCT.md). Critique code, not people.
- **Safety first.** Printers have heaters, motors and mains power. A change that could make a printer do something its owner did not ask for gets the strictest review we have. See [Rules for anything that touches a real printer](#rules-for-anything-that-touches-a-real-printer).
- **No telemetry, no accounts, no cloud.** Stella Scope runs on the user's own computer and talks to the user's own printers. We will not merge anything that phones home, tracks use, or needs an account. Downloads happen only when the person presses a button and asks for them.
- **Honest data.** Recorded data that is missing is shown as unknown. We do not interpolate, smooth over or invent values that were never recorded.
- **Small and dependable beats clever.** We would rather have a boring change that is easy to read at 3 am than a clever one.
- **Maintainers decide.** The maintainers have the final say on what is merged, and on scope. A closed pull request is not a judgement of you or your work; sometimes a good change does not belong in this project.

## Ways to contribute

You do not have to write code.

- **Test on your printer.** Run Stella Scope against a printer we do not own and report what works and what does not. This is among the most valuable things you can do. We run a Voron 2.4 R2 350 (Klipper), a Prusa CORE One L (PrusaLink) and a FlashForge Adventurer 5M daily. Everything else is less tested.
- **Report bugs** with enough detail to reproduce them (below).
- **Improve the documentation**: the [user guide](docs/USER-GUIDE.md), the README, comments that explain why.
- **Fix a bug** or **add a feature** from the issue tracker. Issues labelled `good first issue` and `help wanted` are the best places to start, when there are any.
- **Add support for a printer** (see [Adding support for a printer](#adding-support-for-a-printer)).
- **Translate or simplify wording.** Interface text should be plain words a beginner understands.
- **Share a recording.** A sanitised recording of a failure (see below) lets us fix things we cannot reproduce.

## Reporting a bug

Open an issue using the **Bug report** template. A report that can be reproduced is a report that gets fixed. Please include:

- **What you expected** and **what happened instead**, in plain words.
- **Steps to reproduce**, numbered, starting from a fresh start if you can.
- **Your version.** Open **More, About** in the app, or visit `http://127.0.0.1:7420/api/meta`. Say whether it was the `.deb`, a source build (and the commit), or the demo.
- **Your computer**: operating system and version, and the browser if the problem is in the web interface.
- **Your printer**: the kind (Klipper, Prusa, FlashForge), the model, and the firmware version (Klipper and Moonraker versions for a Klipper machine).
- **Evidence.** A screenshot for anything visual. The text of any error message. Output from the service's terminal or `journalctl -u stella-scope` if you run it as a service.
- **Whether it happens in the demo** (`stella-scope --demo`). If a problem appears in the demo, it is not about your printer, and we can almost always reproduce it.

Before you paste anything, **remove secrets**: Moonraker API keys, PrusaLink passwords, FlashForge check codes, your phone pairing key, your public IP address, and the contents of any `*.env` or password file. Local addresses such as `192.168.1.20` are fine.

If a recording would help, say so and we will tell you how to share only the part we need. Recordings can contain your G-code file names and printer settings, so never attach one publicly without checking it.

## Suggesting a feature

Open an issue using the **Feature request** template and describe **the problem first**, then the solution you imagine. "I run six printers and cannot tell which one needs attention" is a better start than "add a red badge". Tell us:

- who it helps and how often it comes up;
- what you do today instead;
- whether it can be done without touching the printer (reading and showing is much easier to accept than controlling);
- whether you would like to build it yourself.

We say no to features more often than we would like. The project stays small on purpose, and every feature has to be maintained. A short note on why we declined is always given.

## Asking a question

Use the **Question** option in the issue chooser, after reading the [user guide](docs/USER-GUIDE.md) and searching existing issues. Questions are welcome. Answers often become documentation.

## Reporting a security problem

**Do not open a public issue for a vulnerability.** Follow [SECURITY.md](SECURITY.md): use GitHub's private vulnerability reporting on this repository, or email info@3d3d.ca. We will acknowledge your report and work with you on a fix and on crediting you.

## Getting set up

You need:

| Tool | Version | Used for |
|---|---|---|
| Rust | 1.91 or newer, stable | The recorder, drivers and web server (`rustup` is the easiest way). |
| Git | any recent | Source control. |
| A C compiler | `gcc` or `clang` | SQLite is built from source. |
| Node.js | 22 | The interface tests and smoke tests under `scripts/`. |
| WebKitGTK 4.1 and libsoup 3 development files | Linux only | Only for the desktop app in `desktop/`. |
| `cargo-tauri` | 2.x | Only to build the desktop installer. |

Then:

```sh
# 1. Fork the repository on GitHub, then clone your fork
git clone https://github.com/<your-name>/stella-scope-community.git
cd stella-scope-community

# 2. Keep a link to the original so you can stay up to date
git remote add upstream https://github.com/3d3dcanada/stella-scope-community.git

# 3. Branch from the current main
git fetch upstream
git switch -c fix/short-description upstream/main
```

Name branches by what they do: `fix/queue-priority-sort`, `feat/prusa-mk4-bed-mesh`, `docs/flashforge-lan-mode`.

## Building

The recorder and web interface are one Rust binary:

```sh
cargo build --release --locked
./target/release/stella-scope --demo        # nine simulated printers, no real printer touched
```

Always pass `--locked`. `Cargo.lock` is committed, and CI builds with `--locked`.

Other parts are separate builds:

```sh
# The phone link (a sidecar the desktop app starts)
cd link && cargo build --release --locked

# The desktop installer (Linux .deb). The sidecars must be copied in first, named with the target triple:
mkdir -p desktop/src-tauri/binaries
cp target/release/stella-scope        desktop/src-tauri/binaries/stella-scope-x86_64-unknown-linux-gnu
cp link/target/release/stella-link    desktop/src-tauri/binaries/stella-link-x86_64-unknown-linux-gnu
cd desktop/src-tauri && cargo tauri build --bundles deb
```

The Android app in `mobile/` needs the Android SDK and NDK and is not covered here. If you want to work on it, open an issue and we will help you get started.

## Testing

Run these before you push. CI runs the same checks, so a failure here will be a failure there.

```sh
# 1. Formatting. CI fails on unformatted code.
cargo fmt --check          # or just `cargo fmt` to fix it

# 2. Rust tests (several minutes; -j 2 keeps the machine usable)
cargo test --locked -j 2

# 3. Interface tests: plain Node scripts that exercise the web interface code
for t in scripts/*-tests.mjs; do node "$t" || break; done

# 4. Smoke tests: start the real binary against mock printers (needs the release build above)
node scripts/service-http-smoke.mjs
node scripts/workspace-http-smoke.mjs
node scripts/investigation-smoke.mjs
```

The smoke tests use `target/release/stella-scope` unless you set `STELLA_BIN` to another binary.

Some rules for tests:

- **A bug fix comes with a test that fails without the fix.** If it genuinely cannot be tested automatically, say why in the pull request.
- **A new feature comes with tests for what it does and for what it refuses to do.**
- **Tests never contact a real printer.** Use the fakes in `src/fakes.rs`. To try the program against a fake by hand: `cargo build --release --example fake-printers`, then `target/release/examples/fake-printers moonraker 7126` (also `prusalink` and `flashforge`).
- **Tests are deterministic.** No sleeping for "long enough", no dependence on the clock or the network. If time matters, inject it.
- **Do not weaken a test to make it pass.** If a test is wrong, fix it in its own commit and explain why in the message.

For changes to what people see, also look at the result in a browser. Check a desktop window (1366 px wide and wider) and a phone width (about 390 px). The pages have to work on both. Attach screenshots to the pull request.

## How the code is organised

| Where | What lives there |
|---|---|
| `src/main.rs`, `src/config.rs` | Start-up, the command line and the settings file. |
| `src/server.rs`, `src/api.rs`, `src/api/` | The built-in web server and the JSON API the pages use. |
| `src/driver.rs`, `src/drivers/`, `src/moonraker.rs` | The printer drivers. `driver.rs` defines the `Driver` trait; Klipper (Moonraker), Prusa (PrusaLink) and FlashForge each implement it. |
| `src/collector.rs`, `src/store.rs` | Reading from printers and the SQLite recording. |
| `src/farm.rs` | The farm: all printers, their lifecycle and their settings. |
| `src/commands.rs` | Everything that can change a printer, with its safety checks. |
| `src/queue.rs`, `src/queue_flow.rs` | The job queue and slicer upload. |
| `src/printer_models.rs` | The 3D model catalogue, downloads and notices. |
| `src/fakes.rs`, `src/demo*.rs` | Fake printers for tests, and the demo farm. |
| `web/` | The web interface: plain JavaScript modules, CSS and a vendored three.js. It is embedded into the binary at build time. |
| `desktop/`, `mobile/`, `link/`, `eye/` | The desktop app, Android app, phone link and failure watch. |
| `tests/fixtures/`, `scripts/` | Test data, interface tests and smoke tests. |

`src/farm_tests.rs` holds the big farm-level tests, and each module has its own `#[cfg(test)]` tests beside the code.

## Code and writing style

We follow the style of the file you are editing. In general:

**Rust**

- **Format with `cargo fmt`** and nothing else. There is no custom `rustfmt.toml`.
- **Synchronous and blocking.** There is no async runtime in this project. Please do not add one. Threads and channels are fine.
- **No `unsafe`** without a written reason in the pull request and a comment on the line. There are a few existing uses; do not add more casually.
- **No `unwrap()` or `expect()` on anything that comes from a printer, the network, the disk or a user.** Return an error with a message a person can act on. Panicking is acceptable only for broken internal invariants.
- **Errors are written for the person using the app**, not the programmer: say what happened, what to check, and whether anything was changed.
- **Comments say why**, not what. Do not leave commented-out code. Do not put names, work-order numbers or private paths in comments.
- Prefer `cargo clippy` clean on the code you touch. We do not fail CI on it, but we notice.

**JavaScript (`web/`)**

- Plain ES modules, no framework, no build step, no bundler, **no CDN or third-party requests at run time**. Everything the page needs ships inside the binary.
- Colour carries meaning: hot, cold, warning, error. Do not use colour as decoration.
- Every control is a real button or link with a visible name, reachable by keyboard, with a title that says what it does. Do not make the thing that looks like a button a `div`.
- Text is plain words. Say "heat the nozzle", not "set extruder target". Numbers carry their units.
- The interface must work on a phone-width screen as well as a desktop one.

**Documentation and messages**

- Plain, short sentences. British or Canadian spelling is fine; be consistent within a file.
- No marketing language, no exclamation marks, no emoji in code, messages or docs.

## Rules for anything that touches a real printer

This is the part we are strictest about.

1. **Reading is the default.** Anything that only reads and records is easy to accept. Anything that sends a command needs a clear reason.
2. **All commands go through `src/commands.rs`.** Do not send a printer a command from anywhere else. Commands are checked for what they are, whether the printer is in a state where they make sense, and limits (for example heat limits).
3. **Show the exact command first.** The person sees what will be sent, and it goes out once, on their action. No command is ever sent in the background, on a timer, on page load or as a side effect of viewing something.
4. **Controls stay off in replay and demo**, and when the web interface is reachable from anywhere but the local machine.
5. **Never start a print by itself.** Queue upload puts a file in the queue. A person starts it.
6. **Heaters and motion get hard limits** that cannot be bypassed by a typed G-code line or a macro.
7. **Failure is safe.** If the state of the printer is unknown or stale, refuse and say so.
8. **No secrets in the main settings file.** Passwords and access codes live in their own owner-only files.
9. **Prove it with the fakes.** A new command needs tests that show it is sent when allowed and refused when not, using `src/fakes.rs`.

If your change adds or loosens any command, say so at the top of the pull request description. It gets a closer look and may be asked to wait for hardware testing.

## Adding support for a printer

New printers are the most useful contribution after bug reports, but a driver is only as good as the hardware it was tested on. Before you write code, open an issue with:

- the printer and firmware, and a link to its API documentation;
- what you want to read (state, temperatures, job, camera) and what, if anything, you want to control;
- **sanitised sample responses** from the real printer for each call the driver would make;
- confirmation that **you can test on the real machine**, and whether you can test several firmware versions.

A driver pull request then needs:

- the driver in `src/drivers/` implementing the `Driver` trait, following `prusalink.rs` or `flashforge.rs` as examples;
- a fake in `src/fakes.rs` built from your sample responses, and tests that use it;
- fixtures in `tests/fixtures/` (sanitised: no names, addresses, serial numbers or keys);
- an entry in the user guide for how to connect it, including anything the person must switch on at the printer;
- a note in the pull request saying exactly which hardware and firmware you tested on, and what you did **not** test.

Printers whose protocol is closed, undocumented or only reachable by reverse engineering against the owner's wishes are not something we will merge. Bambu Lab support, for example, would need to use an interface the maker supports.

## 3D models

**Do not add printer model files (`.glb`, `.stl`, `.step` and so on) to this repository.** The program ships no models. People download them on request, from the maker's own address, with the licence shown first, and each file is checked against a pinned SHA-256.

If you want a machine to have a 3D model:

- if it already exists in the [3D Print Rig](https://github.com/Psych0h3ad/3d-print-rig) catalogue, it should appear automatically when the catalogue is refreshed; open an issue if it does not;
- if the model is published elsewhere under an open licence, open an issue with the link, the maker, the licence and a statement of what you checked;
- never propose a model whose licence forbids redistribution or whose terms are unclear. We cannot take it and neither should you.

## Dependencies

The dependency list is short on purpose, and every crate we add is code we have to trust and keep updated.

- **Prefer the standard library and what is already in `Cargo.toml`.** Reach for a new crate only when writing the code ourselves would be clearly worse.
- A new dependency needs a justification in the pull request: what it does, why the existing ones do not, its licence, its size, who maintains it and when it was last released.
- Licences must be compatible with the AGPL-3.0 (MIT, Apache-2.0, BSD, ISC, Zlib, MPL and similar permissive licences). No dependency with a licence that restricts use, redistribution or commercial use.
- Commit the updated `Cargo.lock`. Do not run a blanket `cargo update`; update the one crate you need.
- Anything you copy into the tree (code, fonts, data) must be listed with its licence in [docs/THIRD_PARTY.md](docs/THIRD_PARTY.md).
- JavaScript libraries are vendored under `web/vendor/` with their licence and are not loaded from a CDN. We add these very rarely.

## Changing the web interface

- `web/` is embedded into the binary at build time. **A change to a page needs `cargo build` before you see it.** There is no development server. (`STELLA_WEB_DIR` can point the build at a different folder, and `scripts/minify-web.sh` makes the smaller copy a release build uses.)
- Keep changes to one page or one component where you can. Large rewrites of existing pages need an issue first.
- Test with the demo farm (`--demo`) for how it looks and behaves, and with the fakes for anything that needs particular printer states.
- Include screenshots in the pull request at desktop width and at about 390 px. Before and after for a change to something that already exists.
- Do not add anything that makes a network request to a third party.

## Documentation

Documentation changes are welcome and easy to review. The user guide is [docs/USER-GUIDE.md](docs/USER-GUIDE.md) and its pictures are in `docs/guide/`; the guide is also built into the program (Help), so a picture that is missing from the build, or one the guide never uses, fails a test. Keep pictures small and free of real names, addresses and keys. If you change behaviour, change the guide in the same pull request.

## Submitting a pull request, step by step

1. **Agree the change** in an issue if it is not a small fix.
2. **Sync and branch.**
   ```sh
   git fetch upstream
   git switch -c fix/short-description upstream/main
   ```
3. **Make your change.** Keep it to one thing. Do not reformat files you are not changing, rename things for taste, or fix unrelated issues in the same branch; send those separately.
4. **Test it** as described in [Testing](#testing).
5. **Commit** with a clear message (see below). Several small commits while you work are fine; we squash when merging.
6. **Update from `main`** before you open the pull request, so it applies cleanly:
   ```sh
   git fetch upstream
   git rebase upstream/main
   ```
   Re-run the tests if anything came in.
7. **Push to your fork** and **open a pull request against `main`** of `3d3dcanada/stella-scope-community`.
8. **Fill in the template.** It asks what the change does and why, how you tested it, which printers and firmware you tested on, whether it touches printer commands, and for screenshots. Unfilled templates slow down review.
9. **Mark it as a draft** if you want early feedback on an approach. Take it out of draft when it is ready.
10. **Respond to review** by pushing more commits to the same branch. Please do not force-push over review comments once a review has started; it makes it hard to see what changed. If we ask you to rebase, do.
11. **Keep it alive.** If you cannot finish, say so and we will either help or close it kindly. A pull request with no reply from the author for 30 days may be closed; it can always be reopened.

## Commit messages and the pull request title

We merge pull requests with **squash and merge**, so **the pull request title and description become the commit message** on `main`. Write them well.

- **Title**: one line, 72 characters or fewer, in the imperative mood, saying what the change does. Good: `Queue: send a file only to printers whose nozzle fits`. Poor: `fixes`, `Update queue.rs`, `Changes for #12`.
- **Description**: say **why** first (what was wrong or missing), then **what** changed, then **how you checked it**. Link the issue with `Fixes #123` or `Refs #123`.
- Mention **anything surprising**: a behaviour that changed, a limit that moved, something you chose not to do.
- Do not paste secrets, private addresses or file names into commit messages. They live in history for ever.

## What makes a pull request easy to review

- **Small.** A few hundred changed lines is comfortable. Thousands is a conversation to have first.
- **One purpose.** If the description needs the word "also", it is probably two pull requests.
- **Self-explanatory.** The description tells us what to look at and what to ignore.
- **Tested** and says how.
- **Honest about what it did not test** (hardware, firmware, Windows, phone width).
- **No drive-by changes**: no mass formatting, renames or dependency updates mixed in.
- **Screenshots** for anything visual.

## What happens after you open it

1. **CI runs automatically** (formatting, Rust tests, release build, interface tests, smoke tests). A pull request with a failing check will not be reviewed until it is fixed. If you believe a failure is not your change, say so and point at it.
2. **A maintainer looks at it.** We are a small team and review in our working hours (Atlantic time), so please be patient. We aim to give a first reply within about a week. If you have heard nothing after ten days, a polite comment on the pull request is welcome.
3. **Review.** We read the code, run it, and often ask for changes or explain a different direction. Every review comment is meant to improve the change, not to grade you.
4. **Approval.** When the change is correct, tested and in scope, a maintainer approves it. Changes that touch printer commands or the safety rules above may also wait for a test on real hardware.
5. **Merge.** A maintainer squash-merges it. You are credited as the author of the commit. Release notes name contributors unless you ask us not to.

A pull request may be closed without merging if it is out of scope, duplicates other work, cannot be tested, or has gone quiet. We will always say why.

## Using AI tools

You may use AI tools to help write code, tests or documentation. The rules do not change:

- **You are the author and you are responsible** for every line you submit. You must have read it, understood it and run it.
- Say in the pull request if a substantial part was generated.
- Generated code must still be licensed so we can use it under the AGPL-3.0, and must not copy code you do not have the right to share.
- We will close large machine-generated pull requests that were clearly not tested or understood, and unreviewed bulk changes.
- Never paste printer addresses, keys, recordings or private files into a tool that sends them elsewhere.

## Licence of your contribution

Stella Scope is licensed under the [GNU Affero General Public License v3.0](LICENSE). When you submit a pull request, you agree that your contribution is licensed under the same licence, and you confirm that you wrote it or have the right to submit it under that licence. There is no contributor licence agreement to sign and no separate sign-off line is required.

You keep the copyright in what you write. Anything you copy in from elsewhere must be compatible with the AGPL-3.0 and credited in [docs/THIRD_PARTY.md](docs/THIRD_PARTY.md).

## Conduct

Everyone taking part in this project, in issues, pull requests and anywhere else it is discussed, is expected to follow the [code of conduct](CODE_OF_CONDUCT.md). Report a problem to info@3d3d.ca.

---

Thank you again. A clear bug report with a recording, a one-line fix, a test on a printer we do not own: all of it makes Stella Scope better for everyone who runs one.
