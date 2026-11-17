# Stella Scope add-ons: WhatsApp alerts and the failure watch

Stella Scope works without either of these. Both use outside services or code that each owner sets up
for themselves, so the installers do not include them. This page is the source for the "Add-ons"
section of the Stella Scope page on 3d3d.ca.

## WhatsApp alerts (CallMeBot)

Stella Scope can send you a WhatsApp message when a print finishes, fails or is cancelled, on a probe
alarm, a stepper driver warning, Klipper stopping, or the printer dropping off mid-print. Messages go
through CallMeBot's free WhatsApp service, which is for personal use: every owner gets their own key.

1. Add **+34 694 23 41 84** to your phone's contacts.
2. On WhatsApp, send that contact: `I allow callmebot to send me messages`
3. CallMeBot replies with your API key.
4. In Stella Scope on the computer, open **Phone → WhatsApp alerts**, enter your WhatsApp number with
   country code and the key, and press **Save and turn on**.
5. Press **Send a test message**. It should arrive within a minute.

The key is stored only on your computer, readable by your user only, and never shown again. Messages
are spaced at least 20 seconds apart and the same message is not repeated within 10 minutes.

## Failure watch (spaghetti detection)

The failure watch looks at your printer's camera every few seconds while printing and warns you (on
screen and on WhatsApp, if set up) when it sees a print failing. It never stops a print by itself.

It uses the failure-detection model published by Obico (obico-server, AGPL-3.0). Because of that
licence it is not shipped in the installers. You build the small `stella-eye` program yourself and
the model downloads from Obico the first time you switch the watch on.

**What it needs:** a camera that sees the bed (a top-down camera cannot see a small part come loose
under the toolhead), about 600 MB of free memory while printing, and Rust (https://rustup.rs).

1. Get the Stella Scope source and build the watcher:
   ```
   cd stella-scope/eye
   cargo build --release
   ```
2. Copy the program into Stella Scope's data folder (the folder that holds `stella.sqlite3`):
   - Linux, installed app: `~/.local/share/ca.threedthreed.stellascope/`
   - Linux, recorder run as a service: the folder of `database` in its `stella-scope.json`
   - Windows: `%APPDATA%\ca.threedthreed.stellascope\` (copy `stella-eye.exe`)
   ```
   cp target/release/stella-eye ~/.local/share/ca.threedthreed.stellascope/
   ```
3. In Stella Scope open **View → Spaghetti watch** and switch it **On**. The first time, it downloads
   the 200 MB model beside your recordings; the watch starts when the download completes.

Switching it **Off** stops the watcher at once. The model stays on disk for next time.
