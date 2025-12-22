## v1.8.3

Changes since v1.8.2:
* feat(ui): dynamic winnowing tab visibility based on conflicts
* chore(tools): update notify binary [skip ci]
* fix(notify): replace external curl with native reqwest client
* chore(tools): update notify binary [skip ci]
* fix(tools): set working dir for curl to avoid path issues
* chore(tools): update notify binary [skip ci]
* fix(tools): use absolute path for curl upload
* chore(tools): update notify binary [skip ci]
* fix(tools): capture stderr in curl wrapper
* webui: fix TELEGRAM_LINK
* [skip ci]ci: fix repeat build push on notify
* chore(tools): update notify binary [skip ci]
* fix(tools): detach notify from parent workspace
* chore(ci): migrate telegram notifier to rust binary
* Reapply "feat: Check CONFIG_TMPFS_XATTR=y runtime support for OverlayFS on tmpfs"
* 	modified:   .github/workflows/build.yml
* fix(overlay): remove ambiguous .as_ref() calls to fix compilation errors
* Revert "fix(build): remove Clippy check from Rust toolchain setup"
* Reapply "[skip ci]fix: ensure all branches trigger build on push"
* fix(overlay): use detached mounts for robust child mount restoration and add rollback
* fix: make clippy happy
* Revert "[skip ci]fix: ensure all branches trigger build on push"
* fix insets
* feat(webui): add telegram group button to info tab
* Revert "feat(hymofs): support merge rules protocol v6"
* fix(executor): repair SELinux context for overlayfs upperdir
* feat(hymofs): support merge rules protocol v6
* fix(cargo): remove unused cargo-features entry for edition 2024
* fix(core): import PathBuf and unix fs extensions to fix build errors
* fix(mount): explicit type annotation for path components to fix build
* Revert "feat: merge Tools-cx-app/meta-magic_mount#27 && bump edition to 2024"
* make cargo clippy happy  (#64)
* fix(build): remove Clippy check from Rust toolchain setup
* fix(mount): auto-detect and restore sub-mounts to prevent RIL/firmware lookup failure
* [skip ci]fix: ensure all branches trigger build on push
* feat: merge Tools-cx-app/meta-magic_mount#27 && bump edition to 2024