## v3.1.4

Changes since v3.1.3:
* fix(planner): correct module partition directory detection logic
* chore(deps): bump rollup## v3.1.2

Changes since v3.1.1:
* feat: add remote release step for KernelSU-Repo in workflow
* fix: correct typo in module summary
* chore(release): sync version v3.1.1 [skip ci]## v3.1.1

Changes since v3.1.0:
* daemon: using ksu's override.description api for description overriding
* chore(deps): bump ajv
* chore: remove unused fs import from mount.rs
* chore(tools): update notify binary [skip ci]
* feat: add branch name to Telegram notification and improve zip file detection
* feat: add webuiIcon parameter to module.prop generation
* feat: update launcher icon
* feat: add webui shortcut button
* chore(deps): update webui deps
* feat: removed useless read self mounts
* fix: fix typo
* chore: make cargo clippy happy
* docs: moved README to docs/
* refactor: abstract mount backend and storage implementations using traits
* fix: update refactored `fs` utility imports and function calls
* chore(deps): bump the crates group with 3 updates
* refactor: move fs module from utils to sys
* chore(release): sync version v3.1.0 [skip ci]## v3.1.0

Changes since v3.0.2:
* chore: add module.prop to gitignore
* fix: fix error handle logic
* feat: dropped folder check when umount
* chore: fmt
* feat: dropped zygisk check
* chore: removed module.prop
* feat: only use MNT_DETACH to umount
* workflow: fix workflow fmt
* refactor: ci&release logic
* feat: add overlay supported check
* chore: fmt && make cargo clippy happy (#215)
* fix: cargo fmt
* chore: fmt && make cargo clippy happy
* fix: fix nuke failed && fix ap no hidden ext4 loop
* feat(deps): bump && removed useless deps
* refactor: refactor mount_ext4
* fix: fix build
* feat: dropped poaceae subcommand
* chore(tools): update notify binary [skip ci]
* fix: fix build
* notify: make binary size less
* chore: fix gitignore
* refactor: refactor ignore partitions logic
* Change build command to use CI configuration
* Refactor:build_full function to handle CI and archs
* fix: fix error handling methods
* feat: dropped tmpfs setting
* opt: Optimize the logic of donnot umount
* feat: add ignore paths in umount
* feat: Make the errors of the fsopen operation more detailed
* fix: fix try umount flags
* feat: add umount successful log
* fix: correct modules name in some pathes
* chore(tools): update notify binary [skip ci]
* build: unify project naming to Hybrid-Mount and fix xtask syntax
* Revert "chore: rename package name in Cargo.toml"
* workflow: rename build.yml
* chore: rename package name in Cargo.toml
* xtask: refactor generate the module.prop
* chore(deps): bump tgbot from 0.41.0 to 0.42.0 in /tools/notify in the crates group (#212)
* chore(deps): update toml requirement from 0.9 to 1.0 in the crates group (#211)
* chore(release): bump version to v3.0.2 [skip ci]## v3.0.2

Changes since v3.0.14:
* chore: bump to v3.0.2
* build: inject release flag into webui constants and use it for logo display
* chore: fmt
* chore(release): bump version to v3.0.14 [skip ci]