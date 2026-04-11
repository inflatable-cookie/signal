# g09.015 - Platform Boundary Operator Views Closeout

Date: 2026-04-11  
Card: `docs/specs/batch-cards/060-g09-015-platform-boundary-operator-views.md`  
Status: complete

## Summary

Promoted the two remaining receipt-only platform boundary demos into rendered
browser-native operator companions. The macOS AU/CoreAudio and Linux LV2/backend
surfaces are now visually inspectable without reading raw receipts first, and
the platform pair no longer blocks operator-visible coverage breadth.

## Delivered

- added rendered companions to:
  - `demos/scripts/run_macos_au_coreaudio_boundary_demo.py`
  - `demos/scripts/run_linux_lv2_and_backend_boundary_demo.py`
- generated:
  - `demos/receipts/macos-au-coreaudio-boundary.view.html`
  - `demos/receipts/linux-lv2-backend-boundary.view.html`
- aligned manifests to the rendered views:
  - `demos/manifests/macos-au-coreaudio-boundary.demo.json`
  - `demos/manifests/linux-lv2-backend-boundary.demo.json`
- removed recursive Effigy lock conflicts by flattening the acceptance proof
  chains inside the platform demo scripts instead of invoking nested
  `effigy acceptance:*` tasks

## Validation Run

- `python3 demos/scripts/run_macos_au_coreaudio_boundary_demo.py`
- `python3 demos/scripts/run_linux_lv2_and_backend_boundary_demo.py`
- `effigy demo:macos-au-coreaudio-boundary`
- `effigy demo:linux-lv2-and-backend-boundary`
- `effigy qa:docs`
- `effigy qa:northstar`

## Result

- the remaining receipt-only platform demo surfaces are now rendered
- `g09.015` has reached the planned checkpoint for close-or-continue judgment

## Next Task

Re-enter planning at the `g09.015` checkpoint and decide whether the lane can
close on completed operator-visible coverage or whether one final deeper live
plugin-interaction tranche is still honestly required.
