# **Meta-Hybrid Mount**

![Language](https://img.shields.io/badge/Language-Rust-orange?style=flat-square&logo=rust)
![License](https://img.shields.io/badge/License-GPL--3.0-blue?style=flat-square)

> 这是一个用于 KernelSU/Magisk 的混合挂载 (Hybrid Mount) 元模块，通过原生 Rust 二进制文件实现了 OverlayFS 和 Magic Mount 逻辑。
> 基于俺寻思和 Vibe Coding。

---

<div align="center">

**[ 🇺🇸 English ](README.md)**

</div>

---

## **核心架构**

* **真·混合引擎**:
  * **逻辑**: 使用 Rust 编写，利用 `rustix` 进行直接系统调用，安全且高效。
  * **机制**: 智能结合 **OverlayFS** 和 **Magic Mount**。优先使用 OverlayFS 以保证性能，对于不支持的情况或特定模块配置，自动回退到 Magic Mount。

* **智能同步 (Smart Sync)**:
  * **性能**: 引入了开机增量同步机制。不再每次开机都清空重写，而是通过对比 `module.prop` 检测变化。
  * **速度**: 仅同步更新或新增的模块，大幅减少 I/O 占用，显著提升开机速度。

* **智能存储**:
  * **优先级**: 优先使用 **Tmpfs**（内存文件系统），实现最快的速度和最好的隐蔽性。
  * **回退机制**: 自动检测 Tmpfs 是否支持 XATTR（SELinux 必须）。如果不支持，则自动回退到挂载 2GB 的 ext4 loop 镜像 (`modules.img`)。

