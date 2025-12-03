# **Meta-Hybrid Mount**

![Language](https://img.shields.io/badge/Language-Rust-orange?style=flat-square&logo=rust)
![License](https://img.shields.io/badge/License-GPL--3.0-blue?style=flat-square)

> A Hybrid Mount metamodule for KernelSU/Magisk, implementing both OverlayFS and Magic Mount logic via a native Rust binary.

---

<div align="center">

**[ 🇨🇳 中文 (Chinese) ](README_ZH.md)**

</div>

---

## **Core Architecture**

* **True Hybrid Engine**:
  * **Logic**: Written in Rust using `rustix` for direct syscalls, ensuring high performance and safety.
  * **Mechanism**: Intelligently mixes **OverlayFS** and **Magic Mount**. It prioritizes OverlayFS for performance but automatically falls back to Magic Mount for specific modules or partitions if needed.

* **Smart Sync (New)**:
  * **Performance**: Implements an incremental synchronization strategy on boot. Instead of wiping and re-copying everything, it checks for changes in `module.prop`.
  * **Speed**: Only modified or new modules are synced, drastically reducing boot time I/O overhead.

* **Smart Storage**:
  * **Priority**: Prioritizes **Tmpfs** (memory-backed filesystem) for maximum speed and stealth.
  * **Fallback**: Automatically detects if Tmpfs supports XATTR (required for SELinux). If not, it safely falls back to mounting a 2GB ext4 loop image (`modules.img`).