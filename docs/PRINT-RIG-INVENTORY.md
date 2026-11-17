# 3D Print Rig inventory (2026-10-18)

Everything in Stella Scope's catalogue (`web/printer-models.json`) that comes from 3D Print Rig, with the file, the
motion profile, the licence class and the source repository of each entry. The tables at the end are written by
`tools/printer-models-index.rs` from the project's own registrations, so they cannot drift from the catalogue.
Written for polish board WO-01 (the owner, 2026-10-18: "Get the new models, they have the toolchangers, all the
accessories, all the add-ons, all the parts").

## What the sources hold

3D Print Rig is `github.com/Psych0h3ad/3d-print-rig` (viewer code MIT) plus eight companion repositories that
hold its CAD. Mirrored on 2026-10-18 to `/media/drive/stella-scope-sources/3d-print-rig-2026-10-18/`
(git `--mirror` of every repo, 2.7 GB, and every release asset except the wallpapers, 13 GB). The 90 release
files the mirror holds were each checked against the SHA-256 GitHub states for them (0 differ); the 64 assets it
does not hold are the superseded `viewer-vNN` model bundles of 10-01 to 10-07, which the 10-12 mirror did not hold either.

| repository | last push | what it holds | in the 10-12 mirror |
|---|---|---|---|
| `3d-print-rig` | main `190637d`, 2026-10-12 | viewer, registrations, notices; release `viewer-trident-deck-103` (the model bundle, 851 MB) | yes |
| `3d-print-rig-community-models` | 2026-10-10 | community machines (Ender-3, SV08, Satsuma, TicTac, ...) | yes |
| `3d-print-rig-large-voron-models` | 2026-10-11 | 500/1000 Voron, internal spool, requested mods | yes |
| `3d-print-rig-toolhead-models` | 2026-10-07 | Trinity heads | no, mirrored now |
| `3d-print-rig-extra-models` | 2026-10-04 | Annex K1 to K3, ReMorph | no, mirrored now |
| `3d-print-rig-crossant-models` | 2026-10-04 | Crossant 235 | no, mirrored now |
| `3d-print-rig-positron-models` | 2026-10-05 | Positron V3.2.2 | no, mirrored now |
| `3d-print-rig-247zero-models` | 2026-10-10 | 247Zero beta 2 | no, mirrored now |
| `3d-print-rig-ratrig-models` | 2026-10-02 | 18 Rat Rig V-Core 4 | no, mirrored now |

**Nothing in the project changed after 10-12.** Its `main` is the same commit as in the 10-12 mirror
(`190637d`); the 2026-10-15 push was a new branch, `codex/model-loading-recovery`, pointing at that same commit;
community-models and large-voron-models have identical refs; no release is newer than `voron500-assy-v105`
(2026-10-09). Running the catalogue tool on the new mirror gives byte-identical `printer-models.json` machines,
`printer-motion.json` and `printer-thumbs.json`. What the catalogue lacked on 10-12 was not new models but the
**options**: it listed the 67 machines only, while the project's toolchangers, toolheads, accessories and
add-ons live in registrations of their own. This work adds them.

## What was added to the catalogue

- **Options.** 209 options in three kinds: 61 toolchangers, 61 toolheads, 87 add-ons. Each is a set of **files**
  (278 parts, one GLB each, 667 MB packed in all), each part pinned by its stored and its unpacked SHA-256, with
  its own licence class and the address its author publishes it at (`psych0h3ad.github.io/...`, https).
- **Where an option goes.** For a machine an option fits, the catalogue carries the place the project's own
  registrations give (millimetres in the machine's CAD frame). The tool bank and the INDX bank are the project's
  `bankPlan` and `installedHeadPlan` carried into the catalogue; `evidence/WO-01/bank-truth.mjs` runs the project's
  own functions on its data and `bank-compose.mjs` checks the catalogue against them: 33 states (V2.4 250, 300, 350 LDO
  and SIBOOR V2.4 350 with both banks; Trident 250, 300 and 350 with INDX; one tool, all docks with the first
  active, all docks with the last active), 453 placed parts, all equal within 1e-6 mm. `src/printer_models.rs` has the
  same numbers as a test.
- **Licence rule.** The class of an option is the strictest of its files. A file's class comes from the licence
  its own metadata states, and from the project's `docs/THIRD_PARTY_NOTICES.md` for the same source commit; no
  statement means `unlicensed`. NC, mixed, maker-licence and unlicensed options are never carried by a build:
  they are fetched on request with the licence shown first. For example the StealthChanger bank is `nc`
  because its Xol head is CC BY-NC-SA 4.0 (Armchair Heavy Industries), although its docks are GPL-3.0.
- **One machine added.** `e3ng_toolchanger`, the E3NG hotend changer with its gantry (CC BY-NC-SA 4.0, the
  project says "not the complete printer"). It was in the model bundle but not among the 67 machines of the
  project's `index.json`.
- **Upstream address.** The 15 machines whose files are in the model bundle now say the address the project
  publishes them at (`psych0h3ad.github.io/3d-print-rig/machines/<id>/model.glb.gz`) instead of a prose note, so
  "fetch from the author" works for them.

### The three kinds

| kind | what | where it comes from | drawn by Stella on a printer |
|---|---|---|---|
| toolchanger | frame tool banks (StealthChanger on V2.4 250/300/350 and SIBOOR V2.4; INDX on V2.4 and Trident), 59 standalone changer systems (StealthChanger standard 6/9 and Monolith, TapChanger, MadMax plates, YUDX, fixed-gantry StealthChanger) | `TOOLCHANGER_BANK.json`, `TOOLCHANGER_CONFIGURATIONS.json`, `TOOLHEAD_CONFIGURATIONS.json`, `MACHINE_HEAD_REGISTRATIONS.json` | the two banks, yes: docks, fasteners, parked heads, the tool on the gantry, the machine's own head hidden |
| toolhead | 22 head families (Stealthburner, Xol, JabberWocky, INDX, Sphinx, Dragon and Rapid Burner, Yavoth, A4T, Archetype, CrownCooler, Lineux, Trinity, EVA ...) with the machines the project's own installation rules (`machineHeadVariants`) accept them on; 39 hotends, extruders, carriages and electronics from the component lists | `TOOLHEAD_CONFIGURATIONS.json`, `HEAD_ADDITIONS.json`, `COMPONENT_LIBRARY.json`, `COMPONENT_ADDITIONS.json` | only as the tool of a bank. A head on a machine is 1291 registered variants in the project; they are listed per family with their licence and files, not placed one by one |
| add-on | Sturdy Handles, Stealth Skirts (two), Disco on a Stick, Trident A/B stepper fans, Trident internal spool holder (the machine-fitted ones); V0 mods, Trinity, CocobustorZero, Ducky0, Kirigami and other library parts shown on their own | `MACHINE_MODS.json`, `REQUESTED_MOD_INSTALLATIONS.json`, `TRIDENT_INTERNAL_SPOOL.json`, the component lists | the six machine-fitted ones, yes (the machine's stock parts they replace are hidden) |

### What is listed but not drawn, and why

- 179 standalone options (changer systems, library parts): the project shows them on their own and registers no
  installation on a printer. They carry their placements for a future "look at it" view.
- 22 head families: see above. Installed heads follow the machine's motion in the project; Stella has registered
  motion for 9 machines only (below), so a head would not move with the printer.
- The project's installed V0 mods (`V0_INSTALLATIONS.json`, seven slots with `requires` and `conflicts`), the EVA
  installations on Rat Rig machines (`EVA_INSTALLATIONS.json`), Trinity alpha hosts, the Monolith gantries
  (`GANTRY_CONFIGURATIONS.json`, `MONOLITH_MACHINE_REGISTRATIONS.json`) and the Ender-3 mods
  (`COMMUNITY_INSTALLATIONS.json`) each carry their own selection logic (part visibility, replaced stock parts,
  rotations). Their files that are separate GLBs are in the list above as standalone options; their per-machine
  selection logic is not ported.
- The E3NG changer has the project's selector (one of two gantries, one to six tools); Stella shows the whole file
  with every alternate at once and says so on the machine.
- STEP downloads, wallpapers and the viewer itself are not part of Stella Scope.

### Motion profiles

Registered motion (which parts move with X, Y and Z) exists for 9 machines, all from the project's
`site/viewer/motion-profiles/` (MIT): TicTac 2.1, THE 100, Rook MK2, Satsuma180, SOVOL SV08, Annex K1, K2, K3 and
FYSETC V2.4 Pro 250 (plus the CORE One L from Prusa's own CAD, in `printer-motion-extra.json`). The Voron models
have none yet: they are shown as built. The tool on a bank's gantry is placed in the motion group `head`, so it
moves with the head as soon as a machine has a profile that names one.


## The tables

Generated by `cargo run --release --example printer-models-index -- RIG_SRC MIRROR web/printer-models.json TABLES.md` (the fourth argument) and pasted here. Licence classes: gpl, mit, cc-by, cc-by-sa, cc0 may travel with a build with their notice; nc (non-commercial), mixed, ocl (maker licence) and unlicensed never do.

## Machines (68)

| id | label | family | bed mm | licence class | motion | model file | options that fit |
|---|---|---|---|---|---|---|---|
| `doomcube2_350_reference` | DoomCube 2 · 350 / Native CAD | doomcube | 350 | gpl | no | `3d-print-rig-community-models/doomcube-v103/doomcube2_350_reference/model.glb.gz` | 0 |
| `tictac_21_120` | TicTac 2.1 · 120 / Native CAD | tictac | 120 | nc | yes | `3d-print-rig-community-models/requested-v1/tictac/model.glb.gz` | 0 |
| `the100_v11_165` | THE 100 v1.1 · 165 / Native CAD | the100 | 165 | nc | yes | `3d-print-rig-community-models/requested-v1/the100/model.glb.gz` | 0 |
| `rook_mk2_120` | Rook MK2 · 120 / Native CAD | rook | 120 | nc | yes | `3d-print-rig-community-models/requested-v1/rook/model.glb.gz` | 0 |
| `satsuma180_v10` | Satsuma 180 v1.0 / Native CAD | satsuma | 180 | gpl | yes | `3d-print-rig-community-models/requested-v1/satsuma/model.glb.gz` | 0 |
| `sovol_sv08_350` | SOVOL SV08 · 350 / Official CAD | sovol_sv08 | 350 | gpl | yes | `3d-print-rig-community-models/sv08-v2/model.glb.gz` | 0 |
| `lh_stinger_200` | LH Stinger 1.0 · 200 / Orbiter V2.0 / Dragon HF | lh_stinger | 200 | nc | no | `3d-print-rig-community-models/stinger-v3/model.glb.gz` | 0 |
| `antithesis_aether_mk11` | Antithesis Aether MK1.1 · Aluminum / Conch / Galileo 2 | aether | 130 | gpl | no | `3d-print-rig-community-models/aether/model.glb.gz` | 0 |
| `ratrig_vminion_180` | Rat Rig V-Minion 1.0 · 180 / EVA / LGX Lite | vminion | 180 | nc | no | `3d-print-rig-community-models/vminion/model.glb.gz` | 0 |
| `snakeoil_xy_180` | SnakeOil XY · 180 / Mosquito / Sherpa Mini | snakeoil_xy | 180 | mixed | no | `3d-print-rig-community-models/snake180/model.glb.gz` | 0 |
| `snakeoil_xy_idex` | SnakeOil XY · IDEX / Dual Mosquito / Sherpa Mini | snakeoil_xy | 250 | mixed | no | `3d-print-rig-community-models/snakeidex/model.glb.gz` | 0 |
| `snakeoil_3s_kp3s_180` | SnakeOil XY-3S · KP3S / V6 / Sherpa Mini | snakeoil_3s | 180 | mixed | no | `3d-print-rig-community-models/snake3s/model.glb.gz` | 0 |
| `snakeoil_proosaxy` | ProosaXY · Triple Z / V6 / Sherpa Mini | proosaxy | 250 | mixed | no | `3d-print-rig-community-models/proosa/model.glb.gz` | 0 |
| `mercury_one1_235` | Mercury One.1 · Ender-5 / 235 CAD | mercury | 235 | nc | no | `3d-print-rig-community-models/mercury235/model.glb.gz` | 0 |
| `mercury_one1_370` | Mercury One.1 · Ender-5 Plus / 370 CAD | mercury | 370 | nc | no | `3d-print-rig-community-models/mercury370/model.glb.gz` | 0 |
| `vzbot_330_printed` | VzBot 330 · Printed AWD / Goliath / Vz-HextrudORT | vzbot | 330 | nc | no | `3d-print-rig-community-models/vz330/model.glb.gz` | 0 |
| `ender3_stock_220` | Ender-3 · Creality / Original | ender3 | 220 | gpl | no | `3d-print-rig-community-models/ender3/model.glb.gz` | 0 |
| `siboor_sboom_220` | SIBOOR S-BOOM · January 2026 / CoreXZ | sboom | 220 | gpl | no | `3d-print-rig-community-models/sboom/model.glb.gz` | 0 |
| `positron_v322` | Positron V3.2.2 · LDO / 180 | positron | 180 | cc-by-sa | no | `3d-print-rig-positron-models/v3.2.2-native-2/model.glb.gz` | 0 |
| `annex_k1_assembly` | Annex K1 · Release 3.0 · assembly CAD | annex_k1 | 250 | mixed | yes | `3d-print-rig-extra-models/machines/annex_k1_assembly/model.glb.gz` | 0 |
| `annex_k2_assembly` | Annex K2 · Release 4.0 · assembly CAD | annex_k2 | 350 | mixed | yes | `3d-print-rig-extra-models/machines/annex_k2_assembly/model.glb.gz` | 0 |
| `annex_k3_assembly` | Annex K3 · Release 1.1 · 2022-11-24 · assembly CAD | annex_k3 | 180 | mixed | yes | `3d-print-rig-extra-models/machines/annex_k3_assembly/model.glb.gz` | 0 |
| `remorph_beta1_307` | Remorph Beta 1 · 307 / LGX Pro / Rapido UHF / Beacon Rev. H | remorph | 307 | mixed | no | `3d-print-rig-extra-models/machines/remorph_beta1_307/model.glb.gz` | 0 |
| `siboor_trident_300` | SIBOOR Trident 300 · CNC AWD | trident | 300 | unlicensed | no | `3d-print-rig-community-models/siboor-trident-300/model.glb.gz` | 0 |
| `siboor_trident_350` | SIBOOR Trident 350 · CNC AWD | trident | 350 | unlicensed | no | `no model file published` | 2 |
| `voron_trident_250` | VORON Trident 250 · standard printed assembly | trident | 250 | gpl | no | `viewer-models.zip machines/voron_trident_250/model.glb.gz` | 14 |
| `voron_trident_300` | VORON Trident 300 · standard printed assembly | trident | 300 | gpl | no | `viewer-models.zip machines/voron_trident_300/model.glb.gz` | 15 |
| `voron_trident_350` | VORON Trident 350 · standard printed assembly | trident | 350 | gpl | no | `viewer-models.zip machines/voron_trident_350/model.glb.gz` | 15 |
| `voron_trident_500_custom` | VORON Trident 500 × 500 × 250 · Custom | trident | 500 | gpl | no | `3d-print-rig-community-models/trident-deck-v103/voron_trident_500_custom/model.glb.gz` | 0 |
| `voron_trident_1000_custom` | VORON Trident 1000 × 1000 × 994 · Custom | trident | 1000 | gpl | no | `3d-print-rig-community-models/trident-deck-v103/voron_trident_1000_custom/model.glb.gz` | 0 |
| `voron_trident_350_half_z` | VORON Trident 350 × 350 × 125 · Half Z | trident | 350 | gpl | no | `3d-print-rig-community-models/trident-deck-v103/voron_trident_350_half_z/model.glb.gz` | 0 |
| `siboor_v24_350` | VORON V2.4 350 · mod comparisons | v24 | 350 | unlicensed | no | `viewer-models.zip machines/siboor_v24_350/model.glb.gz` | 12 |
| `voron_v24_250_printed` | VORON V2.4 250 · R2standard printed assembly | v24 | 250 | gpl | no | `viewer-models.zip machines/voron_v24_250_printed/model.glb.gz` | 10 |
| `voron_v24_250_ldo_cnc` | VORON V2.4 250 · LDO CNC AWD reference | v24 | 250 | gpl | no | `viewer-models.zip machines/voron_v24_250_ldo_cnc/model.glb.gz` | 10 |
| `voron_v24_300_printed` | VORON V2.4 300 · R2standard printed assembly | v24 | 300 | gpl | no | `viewer-models.zip machines/voron_v24_300_printed/model.glb.gz` | 10 |
| `voron_v24_300_ldo_cnc` | VORON V2.4 300 · LDO CNC AWD reference | v24 | 300 | gpl | no | `viewer-models.zip machines/voron_v24_300_ldo_cnc/model.glb.gz` | 10 |
| `voron_v24_350_printed` | VORON V2.4 350 · R2standard printed assembly | v24 | 350 | gpl | no | `viewer-models.zip machines/voron_v24_350_printed/model.glb.gz` | 10 |
| `voron_v24_500_custom` | VORON V2.4 500 × 500 × 469 · Custom | v24 | 500 | gpl | no | `3d-print-rig-community-models/custom-voron-v3/voron_v24_500_custom/model.glb.gz` | 0 |
| `voron_v24_1000_custom` | VORON V2.4 1000 × 1000 × 969 · Custom | v24 | 1000 | gpl | no | `3d-print-rig-community-models/voron1000-v1/voron_v24_1000_custom/model.glb.gz` | 0 |
| `voron_v24_350_half_z` | VORON V2.4 350 × 350 × 165 · Half Z | v24 | 350 | gpl | no | `3d-print-rig-community-models/custom-voron-v3/voron_v24_350_half_z/model.glb.gz` | 0 |
| `voron_v24_350_ldo_cnc` | VORON V2.4 350 · LDO CNC AWD reference | v24 | 350 | gpl | no | `viewer-models.zip machines/voron_v24_350_ldo_cnc/model.glb.gz` | 10 |
| `voron_v02r1_120` | VORON V0.2r1 120 · Mini Stealthburner | v0 | 120 | gpl | no | `viewer-models.zip machines/voron_v02r1_120/model.glb.gz` | 0 |
| `voron_v02_120` | VORON V0.2 120 · Mini Stealthburner | v0 | 120 | gpl | no | `viewer-models.zip machines/voron_v02_120/model.glb.gz` | 0 |
| `micron_r1_120` | Micron 120 · R1 / AntHead / WWG2 / Revo Voron | micron | 120 | gpl | no | `viewer-models.zip machines/micron_r1_120/model.glb.gz` | 0 |
| `micron_plus_r1_180` | Micron Plus 180 · R1 RC8 / AntHead / WWG2 / Revo Voron | micron | 180 | gpl | no | `viewer-models.zip machines/micron_plus_r1_180/model.glb.gz` | 0 |
| `fysetc_v24_250_pro` | FYSETC V2.4 R2 Pro · official CAD / 250 | v24 | 250 | unlicensed | yes | `viewer-models.zip machines/fysetc_v24_250_pro/model.glb.gz` | 0 |
| `crossant_235_v06_leadscrew` | Crossant-235 · Leadscrew Z / Sherpa Mini / Goliath | crossant | 220 | nc | no | `3d-print-rig-crossant-models/` | 0 |
| `ratrig_vcore_40_300_corexy` | Rat Rig V-Core 4.0 300 · CoreXY | vcore4 | 300 | nc | no | `3d-print-rig-ratrig-models/machines/ratrig_vcore_40_300_corexy/model.glb.gz` | 0 |
| `ratrig_vcore_40_300_hybrid` | Rat Rig V-Core 4.0 300 · Hybrid | vcore4 | 300 | nc | no | `3d-print-rig-ratrig-models/machines/ratrig_vcore_40_300_hybrid/model.glb.gz` | 0 |
| `ratrig_vcore_40_300_idex` | Rat Rig V-Core 4.0 300 · IDEX | vcore4 | 300 | nc | no | `3d-print-rig-ratrig-models/machines/ratrig_vcore_40_300_idex/model.glb.gz` | 0 |
| `ratrig_vcore_40_400_corexy` | Rat Rig V-Core 4.0 400 · CoreXY | vcore4 | 400 | nc | no | `3d-print-rig-ratrig-models/machines/ratrig_vcore_40_400_corexy/model.glb.gz` | 0 |
| `ratrig_vcore_40_400_hybrid` | Rat Rig V-Core 4.0 400 · Hybrid | vcore4 | 400 | nc | no | `3d-print-rig-ratrig-models/machines/ratrig_vcore_40_400_hybrid/model.glb.gz` | 0 |
| `ratrig_vcore_40_400_idex` | Rat Rig V-Core 4.0 400 · IDEX | vcore4 | 400 | nc | no | `3d-print-rig-ratrig-models/machines/ratrig_vcore_40_400_idex/model.glb.gz` | 0 |
| `ratrig_vcore_40_500_corexy` | Rat Rig V-Core 4.0 500 · CoreXY | vcore4 | 500 | nc | no | `3d-print-rig-ratrig-models/machines/ratrig_vcore_40_500_corexy/model.glb.gz` | 0 |
| `ratrig_vcore_40_500_hybrid` | Rat Rig V-Core 4.0 500 · Hybrid | vcore4 | 500 | nc | no | `3d-print-rig-ratrig-models/machines/ratrig_vcore_40_500_hybrid/model.glb.gz` | 0 |
| `ratrig_vcore_40_500_idex` | Rat Rig V-Core 4.0 500 · IDEX | vcore4 | 500 | nc | no | `3d-print-rig-ratrig-models/machines/ratrig_vcore_40_500_idex/model.glb.gz` | 0 |
| `ratrig_vcore_41_300_corexy` | Rat Rig V-Core 4.1 300 · CoreXY | vcore4 | 300 | nc | no | `3d-print-rig-ratrig-models/machines/ratrig_vcore_41_300_corexy/model.glb.gz` | 0 |
| `ratrig_vcore_41_300_hybrid` | Rat Rig V-Core 4.1 300 · Hybrid | vcore4 | 300 | nc | no | `3d-print-rig-ratrig-models/machines/ratrig_vcore_41_300_hybrid/model.glb.gz` | 0 |
| `ratrig_vcore_41_300_idex` | Rat Rig V-Core 4.1 300 · IDEX | vcore4 | 300 | nc | no | `3d-print-rig-ratrig-models/machines/ratrig_vcore_41_300_idex/model.glb.gz` | 0 |
| `ratrig_vcore_41_400_corexy` | Rat Rig V-Core 4.1 400 · CoreXY | vcore4 | 400 | nc | no | `3d-print-rig-ratrig-models/machines/ratrig_vcore_41_400_corexy/model.glb.gz` | 0 |
| `ratrig_vcore_41_400_hybrid` | Rat Rig V-Core 4.1 400 · Hybrid | vcore4 | 400 | nc | no | `3d-print-rig-ratrig-models/machines/ratrig_vcore_41_400_hybrid/model.glb.gz` | 0 |
| `ratrig_vcore_41_400_idex` | Rat Rig V-Core 4.1 400 · IDEX | vcore4 | 400 | nc | no | `3d-print-rig-ratrig-models/machines/ratrig_vcore_41_400_idex/model.glb.gz` | 0 |
| `ratrig_vcore_41_500_corexy` | Rat Rig V-Core 4.1 500 · CoreXY | vcore4 | 500 | nc | no | `3d-print-rig-ratrig-models/machines/ratrig_vcore_41_500_corexy/model.glb.gz` | 0 |
| `ratrig_vcore_41_500_hybrid` | Rat Rig V-Core 4.1 500 · Hybrid | vcore4 | 500 | nc | no | `3d-print-rig-ratrig-models/machines/ratrig_vcore_41_500_hybrid/model.glb.gz` | 0 |
| `ratrig_vcore_41_500_idex` | Rat Rig V-Core 4.1 500 · IDEX | vcore4 | 500 | nc | no | `3d-print-rig-ratrig-models/machines/ratrig_vcore_41_500_idex/model.glb.gz` | 0 |
| `247zero_beta2_120` | 247zero BETA2 · 120 / Crazy Volcano / M4 Bowden | 247zero | 120 | unlicensed | no | `3d-print-rig-community-models/247zero/model.glb.gz` | 0 |
| `ratrig_vcore31_300` | Rat Rig V-Core 3.1 · 300 / EVA 3 / LGX Lite | vcore3 | 300 | nc | no | `3d-print-rig-large-voron-models/eva-installed-v1/vcore31/model.glb.gz` | 0 |
| `e3ng_toolchanger` | E3NG hotend changer · 230 / Native CAD (changer and gantry, not the complete printer) | e3ng | 230 | nc | no | `viewer-models.zip machines/e3ng_toolchanger/model.glb.gz` | 0 |

## Toolchangers (61)

| id | what | category | licence class | files | MB | fits (machines) | drawn by Stella |
|---|---|---|---|---|---|---|---|
| `stealthchanger_bank` | StealthChanger tool bank (frame-mounted docks, parked heads) | tool bank | nc | 8 | 14.4 | 7 | yes |
| `indx_bank` | Bondtech INDX tool bank (passive tools on 2020 docks) | tool bank | gpl | 8 | 3.8 | 10 | yes |
| `stealthchanger_standard_6_a4t` | StealthChanger · standard 6 · A4T · Dual 4010 / 2510 | changer system | gpl | 3 | 0.7 | 0 | no |
| `stealthchanger_standard_6_anthead` | StealthChanger · standard 6 · Anthead | changer system | gpl | 3 | 0.8 | 0 | no |
| `stealthchanger_standard_6_blackbird` | StealthChanger · standard 6 · Archetype · BlackBird / dual 4010 | changer system | gpl | 3 | 0.8 | 0 | no |
| `stealthchanger_standard_6_dragonburner` | StealthChanger · standard 6 · Dragonburner | changer system | gpl | 3 | 0.8 | 0 | no |
| `stealthchanger_standard_6_jabberwocky` | StealthChanger · standard 6 · JabberWocky · Rev82 | changer system | gpl | 3 | 0.9 | 0 | no |
| `stealthchanger_standard_6_rapidburner` | StealthChanger · standard 6 · Rapidburner | changer system | gpl | 2 | 0.4 | 0 | no |
| `stealthchanger_standard_6_stealthburner` | StealthChanger · standard 6 · Stealthburner (SB) | changer system | gpl | 3 | 0.7 | 0 | no |
| `stealthchanger_standard_6_xol` | StealthChanger · standard 6 · Xol | changer system | gpl | 3 | 0.7 | 0 | no |
| `stealthchanger_standard_6_yavoth` | StealthChanger · standard 6 · Yavoth · dual 4010 | changer system | gpl | 3 | 0.8 | 0 | no |
| `stealthchanger_standard_9_a4t` | StealthChanger · standard 9 · A4T · Dual 4010 / 2510 | changer system | gpl | 3 | 0.7 | 0 | no |
| `stealthchanger_standard_9_anthead` | StealthChanger · standard 9 · Anthead | changer system | gpl | 3 | 0.9 | 0 | no |
| `stealthchanger_standard_9_blackbird` | StealthChanger · standard 9 · Archetype · BlackBird / dual 4010 | changer system | gpl | 3 | 0.9 | 0 | no |
| `stealthchanger_standard_9_dragonburner` | StealthChanger · standard 9 · Dragonburner | changer system | gpl | 3 | 0.9 | 0 | no |
| `stealthchanger_standard_9_jabberwocky` | StealthChanger · standard 9 · JabberWocky · Rev82 | changer system | gpl | 3 | 0.9 | 0 | no |
| `stealthchanger_standard_9_rapidburner` | StealthChanger · standard 9 · Rapidburner | changer system | gpl | 2 | 0.4 | 0 | no |
| `stealthchanger_standard_9_stealthburner` | StealthChanger · standard 9 · Stealthburner (SB) | changer system | gpl | 3 | 0.7 | 0 | no |
| `stealthchanger_standard_9_xol` | StealthChanger · standard 9 · Xol | changer system | gpl | 3 | 0.7 | 0 | no |
| `stealthchanger_standard_9_yavoth` | StealthChanger · standard 9 · Yavoth · dual 4010 | changer system | gpl | 3 | 0.9 | 0 | no |
| `stealthchanger_monolith_6_a4t` | StealthChanger · monolith 6 · A4T · Dual 4010 / 2510 | changer system | gpl | 3 | 0.6 | 0 | no |
| `stealthchanger_monolith_6_anthead` | StealthChanger · monolith 6 · Anthead | changer system | gpl | 3 | 0.8 | 0 | no |
| `stealthchanger_monolith_6_blackbird` | StealthChanger · monolith 6 · Archetype · BlackBird / dual 4010 | changer system | gpl | 3 | 0.8 | 0 | no |
| `stealthchanger_monolith_6_dragonburner` | StealthChanger · monolith 6 · Dragonburner | changer system | gpl | 3 | 0.8 | 0 | no |
| `stealthchanger_monolith_6_jabberwocky` | StealthChanger · monolith 6 · JabberWocky · Rev82 | changer system | gpl | 3 | 0.8 | 0 | no |
| `stealthchanger_monolith_6_rapidburner` | StealthChanger · monolith 6 · Rapidburner | changer system | gpl | 2 | 0.3 | 0 | no |
| `stealthchanger_monolith_6_stealthburner` | StealthChanger · monolith 6 · Stealthburner (SB) | changer system | gpl | 3 | 0.6 | 0 | no |
| `stealthchanger_monolith_6_xol` | StealthChanger · monolith 6 · Xol | changer system | gpl | 3 | 0.6 | 0 | no |
| `stealthchanger_monolith_6_yavoth` | StealthChanger · monolith 6 · Yavoth · dual 4010 | changer system | gpl | 3 | 0.8 | 0 | no |
| `stealthchanger_monolith_9_a4t` | StealthChanger · monolith 9 · A4T · Dual 4010 / 2510 | changer system | gpl | 3 | 0.6 | 0 | no |
| `stealthchanger_monolith_9_anthead` | StealthChanger · monolith 9 · Anthead | changer system | gpl | 3 | 0.8 | 0 | no |
| `stealthchanger_monolith_9_blackbird` | StealthChanger · monolith 9 · Archetype · BlackBird / dual 4010 | changer system | gpl | 3 | 0.8 | 0 | no |
| `stealthchanger_monolith_9_dragonburner` | StealthChanger · monolith 9 · Dragonburner | changer system | gpl | 3 | 0.8 | 0 | no |
| `stealthchanger_monolith_9_jabberwocky` | StealthChanger · monolith 9 · JabberWocky · Rev82 | changer system | gpl | 3 | 0.8 | 0 | no |
| `stealthchanger_monolith_9_rapidburner` | StealthChanger · monolith 9 · Rapidburner | changer system | gpl | 2 | 0.3 | 0 | no |
| `stealthchanger_monolith_9_stealthburner` | StealthChanger · monolith 9 · Stealthburner (SB) | changer system | gpl | 3 | 0.6 | 0 | no |
| `stealthchanger_monolith_9_xol` | StealthChanger · monolith 9 · Xol | changer system | gpl | 3 | 0.6 | 0 | no |
| `stealthchanger_monolith_9_yavoth` | StealthChanger · monolith 9 · Yavoth · dual 4010 | changer system | gpl | 3 | 0.8 | 0 | no |
| `tapchanger_sb_rods` | TapChanger · source · Sb rods | changer system | gpl | 1 | 3.2 | 0 | no |
| `tapchanger_minisb_rods` | TapChanger · source · Minisb rods | changer system | gpl | 1 | 1.8 | 0 | no |
| `tapchanger_minisb_plates` | TapChanger · source · Minisb plates | changer system | gpl | 1 | 2.1 | 0 | no |
| `tapchanger_dragon_revo` | TapChanger · source · Dragon revo | changer system | gpl | 1 | 0.8 | 0 | no |
| `tapchanger_dragon_dragon` | TapChanger · source · Dragon dragon | changer system | gpl | 1 | 1.3 | 0 | no |
| `tapchanger_xol_experimental` | TapChanger · source · Xol experimental | changer system | gpl | 1 | 0.5 | 0 | no |
| `tapchanger_liftbar_v24` | TapChanger · source · Liftbar v24 | changer system | gpl | 1 | 9.1 | 0 | no |
| `madmax_mgn9_db_ah` | madmax · mgn9 6 · Db ah plate | changer system | gpl | 2 | 0.5 | 0 | no |
| `madmax_mgn12_db_ah` | madmax · mgn12 6 · Db ah plate | changer system | gpl | 2 | 0.5 | 0 | no |
| `madmax_mgn12_xol_a4t` | madmax · mgn12 6 · Xol a4t plate | changer system | gpl | 6 | 0.8 | 0 | no |
| `yudx_assembly` | YUDX · yudx mgn12 · Yudx assembly | changer system | gpl | 3 | 1.7 | 0 | no |
| `yudx_head` | YUDX · yudx mgn12 · Yudx head | changer system | gpl | 2 | 1.6 | 0 | no |
| `yudx_extruder` | YUDX · yudx mgn12 · Yudx extruder | changer system | gpl | 1 | 1.3 | 0 | no |
| `yudx_tool` | YUDX · yudx mgn12 · Yudx tool | changer system | gpl | 1 | 0.4 | 0 | no |
| `yudx_dock` | YUDX · yudx mgn12 · Yudx dock | changer system | gpl | 1 | 0.1 | 0 | no |
| `fixed_sc_dock_anthead_left` | Fixed-gantry StealthChanger · fixed sc left · Anthead | changer system | gpl | 1 | 0.2 | 0 | no |
| `fixed_sc_dock_a4t_left` | Fixed-gantry StealthChanger · fixed sc left · A4T · Dual 4010 / 2510 | changer system | gpl | 1 | 0.2 | 0 | no |
| `fixed_sc_dock_anthead_right` | Fixed-gantry StealthChanger · fixed sc right · Anthead | changer system | gpl | 1 | 0.2 | 0 | no |
| `fixed_sc_dock_a4t_right` | Fixed-gantry StealthChanger · fixed sc right · A4T · Dual 4010 / 2510 | changer system | gpl | 1 | 0.2 | 0 | no |
| `fixed_sc_shover_ldo` | Fixed-gantry StealthChanger · fixed sc shover · Fixed sc ldo | changer system | gpl | 1 | 0.1 | 0 | no |
| `fixed_sc_shover_fysetc` | Fixed-gantry StealthChanger · fixed sc shover · Fixed sc fysetc | changer system | gpl | 1 | 0.1 | 0 | no |
| `fixed_sc_anthead_sf_ducts` | Fixed-gantry StealthChanger · fixed sc head · Anthead | changer system | gpl | 1 | 0.1 | 0 | no |
| `fixed_sc_a4t_low_rider` | Fixed-gantry StealthChanger · fixed sc head · A4T · Dual 4010 / 2510 | changer system | gpl | 1 | 0.2 | 0 | no |

## Toolheads (61)

| id | what | category | licence class | files | MB | fits (machines) | drawn by Stella |
|---|---|---|---|---|---|---|---|
| `component_dragon_ace_hf` | Dragon ACE HF | hotend | unlicensed | 1 | 0.5 | 0 | no |
| `component_dragon_ace_volcano` | Dragon ACE Volcano | hotend | unlicensed | 1 | 0.3 | 0 | no |
| `component_rapido_ace_hf` | Rapido ACE HF | hotend | unlicensed | 1 | 0.6 | 0 | no |
| `component_rapido_ace_uhf` | Rapido ACE UHF / MZE | hotend | unlicensed | 1 | 0.4 | 0 | no |
| `component_bms6` | Dragonfly BMS · 6 fins | hotend | unlicensed | 1 | 0.3 | 0 | no |
| `component_bms7` | Dragonfly BMS · 7 fins | hotend | unlicensed | 1 | 0.6 | 0 | no |
| `component_copperhead_groove` | Copperhead · Groove mount | hotend | unlicensed | 1 | 0.1 | 0 | no |
| `component_copperhead_screw` | Copperhead · Screw mount | hotend | unlicensed | 1 | 0.1 | 0 | no |
| `component_revo_voron` | Revo Voron | hotend | nc | 1 | 0.1 | 0 | no |
| `component_revo_voron_pz` | Revo Voron PZ | hotend | unlicensed | 1 | 0.0 | 0 | no |
| `component_revo_pz_controller` | Revo PZ Controller | electronics | unlicensed | 1 | 0.1 | 0 | no |
| `component_revo_micro` | Revo Micro · Reference CAD | hotend | nc | 1 | 0.0 | 0 | no |
| `component_lgx_lite` | LGX Lite · Simplified V6 | extruder | unlicensed | 1 | 0.0 | 0 | no |
| `component_sherpa2_short` | Sherpa Mini R2 · Short idler | extruder | mixed | 1 | 1.9 | 0 | no |
| `component_sherpa2_long` | Sherpa Mini R2 · Long idler | extruder | mixed | 1 | 1.9 | 0 | no |
| `component_galileo2` | Galileo 2 · G2E | extruder | gpl | 1 | 1.3 | 0 | no |
| `component_additional_metal_tap_wired` | MetalTap | carriage | gpl | 1 | 0.5 | 0 | no |
| `component_additional_metal_tap_opto` | MetalTap · OptoTap PCB v4 | carriage | gpl | 1 | 0.5 | 0 | no |
| `component_additional_lightweight_sb` | Vitalii Lightweight · SB | carriage | gpl | 1 | 0.1 | 0 | no |
| `component_additional_lightweight_xol` | Vitalii Lightweight · Xol | carriage | gpl | 1 | 0.1 | 0 | no |
| `component_additional_monolith_milled_clamp` | Monolith · Milled belt clamp | carriage | gpl | 1 | 0.1 | 0 | no |
| `component_additional_monolith_slm_clamp` | Monolith · SLM belt clamp | carriage | gpl | 1 | 0.3 | 0 | no |
| `component_head_lineux_board_orbiter2_ebb36` | BTT EBB36 CAN V1.1 | electronics | unlicensed | 1 | 2.1 | 0 | no |
| `component_head_lineux_board_orbiter2_h36_v13` | FYSETC H36 V1.3 | electronics | unlicensed | 1 | 1.3 | 0 | no |
| `component_head_lineux_board_orbiter2_h36_v2` | FYSETC H36 V2.0 | electronics | unlicensed | 1 | 1.4 | 0 | no |
| `component_head_lineux_board_orbiter2_nh36` | Nitehawk-36 V1.4 | electronics | unlicensed | 1 | 1.3 | 0 | no |
| `component_indx_link` | INDX Link 1.0 | electronics | gpl | 1 | 0.3 | 0 | no |
| `component_ahi_reference_escapement` | Escapement · Galileo 2 | extruder | gpl | 1 | 0.7 | 0 | no |
| `component_ahi_reference_sharketype` | Sharketype · BMG | extruder | nc | 1 | 0.4 | 0 | no |
| `component_ahi_a4t_wwbmg` | A4T WW-BMG main body · Sherpa Mini spacing | extruder | nc | 1 | 0.1 | 0 | no |
| `component_chube_compact` | Chube Compact | hotend | unlicensed | 1 | 0.3 | 0 | no |
| `component_chube_air_conduction_reference` | Chube Air / Conduction | hotend | unlicensed | 1 | 0.2 | 0 | no |
| `component_tricorn_long` | Tricorn Long | hotend | cc0 | 1 | 0.5 | 0 | no |
| `component_tricorn_short` | Tricorn Short | hotend | cc0 | 1 | 0.5 | 0 | no |
| `component_goliath_air` | Goliath · Mellow · Air | hotend | nc | 1 | 0.1 | 0 | no |
| `component_goliath_water` | Goliath · Mellow · Water | hotend | nc | 1 | 0.2 | 0 | no |
| `component_goliath_short_wc` | Goliath Short + WC Vz-HextrudORT | hotend | nc | 1 | 2.1 | 0 | no |
| `component_rapido_x` | Rapido X | hotend | unlicensed | 1 | 0.4 | 0 | no |
| `component_v0mod_cocobustorzero_m4max` | CocobustorZero · M4max · source parts | extruder | gpl | 1 | 0.5 | 0 | no |
| `head_a4t` | A4T · Dual 4010 / 2510 toolhead | head family | nc | 2 | 7.3 | 10 | no |
| `head_anthead_papilio` | AntHead · Papilio Lite toolhead | head family | gpl | 2 | 9.0 | 0 | no |
| `head_atrocity` | Archetype · Atrocity toolhead | head family | nc | 1 | 0.2 | 0 | no |
| `head_blackbird` | Archetype · BlackBird / dual 4010 toolhead | head family | nc | 1 | 0.2 | 0 | no |
| `head_breakneck` | Archetype · Breakneck / CPAP toolhead | head family | nc | 1 | 0.2 | 0 | no |
| `head_crowncooler` | CrownCooler · CPAP / Orbiter toolhead | head family | gpl | 1 | 3.2 | 10 | no |
| `head_dragon_burner` | Dragon Burner · v8 toolhead | head family | unlicensed | 18 | 21.0 | 0 | no |
| `head_eva_2` | EVA 2 toolhead | head family | unlicensed | 3 | 16.4 | 0 | no |
| `head_eva_3` | EVA 3 toolhead | head family | unlicensed | 3 | 55.7 | 0 | no |
| `head_indx` | Bondtech INDX toolhead | head family | gpl | 4 | 9.3 | 10 | no |
| `head_jabberwocky` | JabberWocky · Rev82 toolhead | head family | cc-by-sa | 1 | 4.9 | 10 | no |
| `head_lineux_tiga` | Lineux · Tiga toolhead | head family | gpl | 1 | 1.6 | 0 | no |
| `head_mini_stealthburner` | Mini Stealthburner toolhead | head family | gpl | 2 | 3.8 | 0 | no |
| `head_mjolnir` | Archetype · Mjolnir toolhead | head family | nc | 1 | 0.2 | 0 | no |
| `head_rapid_burner` | Rapid Burner · v8 toolhead | head family | gpl | 4 | 0.5 | 0 | no |
| `head_sphinx` | Sphinx toolhead | head family | nc | 16 | 5.2 | 10 | no |
| `head_stealthburner` | Stealthburner (SB) toolhead | head family | unlicensed | 4 | 11.4 | 10 | no |
| `head_trinity` | Trinity toolhead | head family | mixed | 4 | 68.5 | 0 | no |
| `head_trinity_crossbow` | Trinity · Crossbow toolhead | head family | mixed | 10 | 167.1 | 0 | no |
| `head_xol` | Xol toolhead | head family | nc | 1 | 8.2 | 10 | no |
| `head_yavoth` | Yavoth · dual 4010 toolhead | head family | gpl | 18 | 1.2 | 10 | no |
| `head_zephyr` | Archetype · Zephyr toolhead | head family | nc | 1 | 0.2 | 0 | no |

## Add-ons (87)

| id | what | category | licence class | files | MB | fits (machines) | drawn by Stella |
|---|---|---|---|---|---|---|---|
| `disco_on_a_stick` | Disco on a Stick (LED bar) | add-on | gpl | 1 | 1.9 | 5 | yes |
| `stealth_skirts_cheese_grater` | Stealth Skirts · Cheese Grater | add-on | gpl | 9 | 15.8 | 3 | yes |
| `stealth_skirts_honeycomb` | Stealth Skirts · Honeycomb | add-on | gpl | 9 | 8.7 | 3 | yes |
| `sturdy_handles_m5` | Sturdy Handles · M5 | add-on | gpl | 3 | 0.1 | 5 | yes |
| `trident_ab_stepper_fan_491404_v2` | Trident A/B stepper fans · FOX V2 | add-on | nc | 3 | 10.8 | 3 | yes |
| `trident_internal_spool` | Trident internal spool holder (elcrni) | add-on | gpl | 2 | 16.3 | 2 | yes |
| `component_additional_doublet_v24` | DoubleT / Vitalii · V2.4 Aluminum Gantry | gantry | gpl | 1 | 1.8 | 0 | no |
| `component_v0mod_kirigami` | Kirigami V0.2 bed | mod | gpl | 1 | 0.1 | 0 | no |
| `component_v0mod_zeroclick` | ZeroClick probe and three docks | mod | gpl | 1 | 0.3 | 0 | no |
| `component_v0mod_stealth_handles` | V0 Stealth Handles | mod | gpl | 1 | 0.0 | 0 | no |
| `component_v0mod_lift_off_tophat` | Lift-Off Tophat Hinges | mod | gpl | 1 | 0.1 | 0 | no |
| `component_v0mod_tip_tophat` | TipTophat | mod | gpl | 1 | 0.4 | 0 | no |
| `component_v0mod_tophat_cat_flap` | Tophat Cat Flap | mod | gpl | 1 | 0.8 | 0 | no |
| `component_v0mod_xcarriage_inserts` | V0.2 X carriage inserts | mod | gpl | 1 | 0.1 | 0 | no |
| `component_v0mod_mini_fridge` | MiniFridge left/right and 3mm foam doors | mod | gpl | 1 | 0.1 | 0 | no |
| `component_v0mod_umbilical` | V0 Umbilical PCB and V0.2 mount | mod | gpl | 1 | 0.2 | 0 | no |
| `component_v0mod_picobilical` | LDO Picobilical V0.2 toolhead mount | mod | gpl | 1 | 0.0 | 0 | no |
| `component_v0mod_nozzle_wiper_v2` | Servo NozzleWiper v2 | mod | gpl | 1 | 0.6 | 0 | no |
| `component_v0mod_official_hotend_mounts` | Official MiniSB Standard/Fan Saver mounts | mod | gpl | 1 | 1.0 | 0 | no |
| `component_v0mod_official_adxl_mounts` | Official MiniSB ADXL mounts | mod | gpl | 1 | 0.5 | 0 | no |
| `component_v0mod_dragon_burner_v8` | Dragon Burner v8 | mod | gpl | 1 | 2.4 | 0 | no |
| `component_v0mod_rapid_burner_v8` | Rapid Burner v8 | mod | gpl | 1 | 1.2 | 0 | no |
| `component_v0mod_mailbox_v5` | Mailbox 5015 v5 | mod | gpl | 1 | 0.5 | 0 | no |
| `component_v0mod_tulip_standard` | Tulip V0.2r1 standard gantry | mod | gpl | 1 | 5.7 | 0 | no |
| `component_v0mod_official_bowden` | Official MiniSB Bowden toolhead | mod | gpl | 1 | 1.2 | 0 | no |
| `component_ahi_reference_trailhead_xy_joints` | Trailhead XY Joints · 1515/2020 / 6 | gantry | gpl | 1 | 6.9 | 0 | no |
| `component_ahi_reference_adxl_mounts` | AHI ADXL / KUSBA | mod | gpl | 1 | 0.3 | 0 | no |
| `component_ahi_a4t_cowl` | A4T Cowling · Dragon/Rapido · Xol carriage | mod | nc | 1 | 0.5 | 0 | no |
| `component_ahi_a4t_duct` | A4T Rapido hotend fan duct | mod | nc | 1 | 0.1 | 0 | no |
| `component_ahi_a4t_adapter` | A4T WW-BMG adapter · Xol carriage | mod | nc | 1 | 0.0 | 0 | no |
| `component_v21__filamatrix_depressor` | FilamATrix · Beefy Depressor | mod | gpl | 1 | 0.1 | 0 | no |
| `component_v21__madmax_xol_plate` | MadMax · Xol-A4T plate / dock bracket | mod | gpl | 1 | 0.1 | 0 | no |
| `component_v0mod_fysetc_cnc_stealth_front` | V0 · FYSETC CNC Bed Stealth Front | mod | gpl | 1 | 0.0 | 0 | no |
| `component_v0mod_fysetc_cnc_bed_fans` | V0 · FYSETC CNC Bed Fans & Connection Mount | mod | gpl | 1 | 0.0 | 0 | no |
| `component_hydra_ender3_parts` | HYDRA · Ender-3 source parts | mod | nc | 1 | 3.9 | 0 | no |
| `component_trinity_v05` | Trinity · Monolith / 6 mm / Chube Compact / No cutter | mod | mixed | 1 | 17.3 | 0 | no |
| `component_trinity_v06` | Trinity · Monolith / 6 mm / Chube Compact / Crossbow | mod | mixed | 1 | 17.8 | 0 | no |
| `component_trinity_v07` | Trinity · Monolith / 9–10 mm / Chube Compact / No cutter | mod | mixed | 1 | 14.9 | 0 | no |
| `component_trinity_v08` | Trinity · Monolith / 9–10 mm / Chube Compact / Crossbow | mod | mixed | 1 | 15.3 | 0 | no |
| `component_trinity_v09` | Trinity · Monolith / 6 mm / Dragon ACE Volcano / Crossbow | mod | mixed | 1 | 18.8 | 0 | no |
| `component_trinity_v10` | Trinity · Monolith / 9–10 mm / Dragon ACE Volcano / Crossbow | mod | mixed | 1 | 18.8 | 0 | no |
| `component_trinity_v11` | Trinity · Voron / 6 mm / Chube Compact / Crossbow | mod | mixed | 1 | 15.5 | 0 | no |
| `component_trinity_v12` | Trinity · Voron / 9–10 mm / Chube Compact / Crossbow | mod | mixed | 1 | 15.5 | 0 | no |
| `component_trinity_v13` | Trinity · Voron / 6 mm / Dragon ACE Volcano / Crossbow | mod | mixed | 1 | 16.5 | 0 | no |
| `component_trinity_v14` | Trinity · Voron / 9–10 mm / Dragon ACE Volcano / Crossbow | mod | mixed | 1 | 16.6 | 0 | no |
| `component_trinity_accessories` | Trinity · belt clips / depressor | mod | mixed | 1 | 0.4 | 0 | no |
| `component_trinity_native_n00` | Trinity · Klicky dock mount / HVFDesigns | mod | gpl | 1 | 0.0 | 0 | no |
| `component_trinity_native_n01` | Trinity · Klicky mount / HVFDesigns | mod | gpl | 1 | 0.2 | 0 | no |
| `component_trinity_native_n02` | Trinity · Klicky00 plate / Prooda | mod | gpl | 1 | 0.1 | 0 | no |
| `component_trinity_native_n03` | Trinity · 2020 belt pretensioner / Prooda | mod | gpl | 1 | 0.5 | 0 | no |
| `component_trinity_native_n04` | Trinity · Xol / SHT36 V2 board mounts / Prooda | mod | gpl | 1 | 5.3 | 0 | no |
| `component_trinity_native_n05` | Trinity v1.41 · MGN9 / 6 mm | mod | mixed | 1 | 15.8 | 0 | no |
| `component_trinity_native_n06` | Trinity v1.41 · MGN9 / 9 mm | mod | mixed | 1 | 15.8 | 0 | no |
| `component_trinity_native_n07` | Trinity · Apex clip / 6 mm / 2.8 mm | mod | gpl | 1 | 0.0 | 0 | no |
| `component_trinity_native_n08` | Trinity · Apex clip / 6 mm / 2.9 mm | mod | gpl | 1 | 0.0 | 0 | no |
| `component_trinity_native_n09` | Trinity · Apex clip / 6 mm / 3.1 mm | mod | gpl | 1 | 0.0 | 0 | no |
| `component_trinity_native_n10` | Trinity · Apex clip / 6 mm / 3 mm | mod | gpl | 1 | 0.0 | 0 | no |
| `component_trinity_native_n11` | Trinity · Apex clip / 9 mm / 2.8 mm | mod | gpl | 1 | 0.0 | 0 | no |
| `component_trinity_native_n12` | Trinity · Apex clip / 9 mm / 2.9 mm | mod | gpl | 1 | 0.0 | 0 | no |
| `component_trinity_native_n13` | Trinity · Apex clip / 9 mm / 3.1 mm | mod | gpl | 1 | 0.0 | 0 | no |
| `component_trinity_native_n14` | Trinity · Apex clip / 9 mm / 3 mm | mod | gpl | 1 | 0.0 | 0 | no |
| `component_trinity_native_n15` | Trinity · Heatcore 4 UHF parts / cpp0xc0ffeeee | mod | gpl | 1 | 1.3 | 0 | no |
| `component_trinity_native_n16` | Trinity · Chube Compact / MGN12 / 6 mm / Prooda | mod | gpl | 1 | 4.8 | 0 | no |
| `component_trinity_native_n17` | Trinity · Chube Compact / MGN12 / 9 mm / Prooda | mod | gpl | 1 | 2.9 | 0 | no |
| `component_trinity_native_n18` | Trinity · Chube Compact / MGN9 / 6 mm / Prooda | mod | gpl | 1 | 3.0 | 0 | no |
| `component_trinity_native_n19` | Trinity v1.54 · MGN12 / 6 mm | mod | mixed | 1 | 18.5 | 0 | no |
| `component_trinity_native_n20` | Trinity v1.54 · MGN12 / 9–10 mm | mod | mixed | 1 | 18.5 | 0 | no |
| `component_trinity_native_n21` | Trinity · Chube Compact / Magneto / Zerodegree79 | mod | nc | 1 | 12.5 | 0 | no |
| `component_trinity_native_n22` | Trinity · Dragon T-Volcano / front Magneto / Zerodegree79 | mod | nc | 1 | 9.6 | 0 | no |
| `component_trinity_native_n23` | Trinity · Dragon UHF / rear Magneto / Zerodegree79 | mod | nc | 1 | 7.5 | 0 | no |
| `component_trinity_native_n24` | Trinity · Dragon UHF / front Magneto / Zerodegree79 | mod | nc | 1 | 6.6 | 0 | no |
| `component_v0mod_tall_skirts_841565` | V0.2 tall skirts · source variants | mod | gpl | 1 | 5.4 | 0 | no |
| `component_x1c_nozzle_wiper_546970` | X1C/P1P nozzle wiper · beta mount | mod | cc-by-sa | 1 | 0.1 | 0 | no |
| `component_voron_v24_pg7_filter_1230133` | V2.4 PG7 rear exhaust · v2 source parts | mod | gpl | 1 | 13.1 | 0 | no |
| `component_trident_stepper_decoupler_430168` | Trident stepper decoupler · original printed parts | mod | nc | 1 | 1.4 | 0 | no |
| `component_trident_ab_stepper_fan_491404_v2` | Trident A/B stepper fan · V2 | mod | nc | 1 | 0.1 | 0 | no |
| `component_trident_ab_stepper_fan_491404_v1` | Trident A/B stepper fan · V1 | mod | nc | 1 | 0.4 | 0 | no |
| `component_trident_stepper_spacer_534736` | Trident stepper spacer | mod | nc | 1 | 0.0 | 0 | no |
| `component_tanaes_trident_kinematic_left_STEP_62268ed` | Trident kinematic bed · original left mount | mod | gpl | 1 | 0.1 | 0 | no |
| `component_requested_mod_681594_head_source_preview_v5` | EVA for Trident · author CAD subset | mod | cc-by | 1 | 33.6 | 0 | no |
| `component_v0mod_cocobustorzero_awd_coconut_v2` | CocobustorZero · Coconut V2 AWD · source parts | mod | gpl | 1 | 1.1 | 0 | no |
| `component_v0mod_cocobustorzero_ulw_x` | CocobustorZero · ULW X gantry · source parts | mod | gpl | 1 | 0.6 | 0 | no |
| `component_v0mod_cocobustorzero_belted_z` | CocobustorZero · Thick belted Z · source parts | mod | gpl | 1 | 1.0 | 0 | no |
| `component_v0mod_cocobustorzero_air_ducts` | CocobustorZero · Coconut air ducts · source parts | mod | gpl | 1 | 1.9 | 0 | no |
| `component_v0mod_cocobustorzero_accessories` | CocobustorZero · Mounts, front idlers and spool holder · source parts | mod | gpl | 1 | 0.4 | 0 | no |
| `component_v0mod_ducky0_motor_cooling` | Ducky0 · 12032 motor cooling brackets · source parts | mod | gpl | 1 | 0.1 | 0 | no |
| `component_v0mod_ducky0_electronics_mounts` | Ducky0 · Screen and USB hub mounts · source parts | mod | gpl | 1 | 0.1 | 0 | no |

## New or changed since the 2026-10-12 catalogue

- CHANGED `voron_trident_250`: upstream address.
- CHANGED `voron_trident_300`: upstream address.
- CHANGED `voron_trident_350`: upstream address.
- CHANGED `siboor_v24_350`: upstream address.
- CHANGED `voron_v24_250_printed`: upstream address.
- CHANGED `voron_v24_250_ldo_cnc`: upstream address.
- CHANGED `voron_v24_300_printed`: upstream address.
- CHANGED `voron_v24_300_ldo_cnc`: upstream address.
- CHANGED `voron_v24_350_printed`: upstream address.
- CHANGED `voron_v24_350_ldo_cnc`: upstream address.
- CHANGED `voron_v02r1_120`: upstream address.
- CHANGED `voron_v02_120`: upstream address.
- CHANGED `micron_r1_120`: upstream address.
- CHANGED `micron_plus_r1_180`: upstream address.
- CHANGED `fysetc_v24_250_pro`: upstream address.
- NEW machine `e3ng_toolchanger` (E3NG hotend changer · 230 / Native CAD (changer and gantry, not the complete printer), licence class nc).
- 209 options and 278 parts are new: the earlier catalogue carried machines only.
