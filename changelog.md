## v1.3.1-r1

Changes since v1.3.0-r1:
* fix: remove unused `try_umount.rs` to clean up compiler warnings
* [skipci]readme(fix): 写的什么傻逼readme迟早有一天给他重写了
* fix: propagate configured mount source to storage backend setup
* fix: deduplicate umount requests to prevent system instability
* style: standardize safe-area-inset handling and remove comments
* refactor(ui): visually treat unmounted modules as 'None' mode without extra buttons
* chore(release): bump version to v1.3.0-r1 [skip ci]