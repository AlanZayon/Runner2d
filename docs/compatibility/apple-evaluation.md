# iOS / macOS — evaluation checklist (not implemented)

Phase 11 documents acceptance criteria only. Apple targets need a separate graphics ADR (Metal vs MoltenVK) before coding.

## Prerequisites

- Xcode + Apple developer account (device deploy)
- Decision: **MoltenVK** (keep Vulkan shaders) vs native **Metal** backend
- Rust targets: `aarch64-apple-ios` / `aarch64-apple-darwin` as applicable

## Acceptance

1. Window/surface + lifecycle (background/resume) documented and tested.
2. Touch gestures reuse host `GestureRecognizer` mapping.
3. Portrait iPhone/iPad smoke; Mac windowed smoke if macOS claimed.
4. Asset bundle + Application Support save path.
5. Matrix row updated only after on-device (iOS) or local (macOS) run evidence.

## Out of scope until then

- App Store submission, Game Center, multi-backend without ADR.
