// Copyright 2026 Hybrid Mount Developers
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{env, fs, path::Path, process::Command};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use fs_extra::{dir, file};
use serde::Deserialize;
use zip::{CompressionMethod, write::FileOptions};

mod zip_ext;
use crate::zip_ext::zip_create_from_directory_with_options;

#[derive(Deserialize)]
struct Package {
    version: String,
}

#[derive(Deserialize)]
struct CargoConfig {
    package: Package,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
enum Arch {
    #[value(name = "arm64")]
    Arm64,
    #[value(name = "arm")]
    Arm,
    #[value(name = "x86_64")]
    X86_64,
}

impl Arch {
    fn target(&self) -> &'static str {
        match self {
            Arch::Arm64 => "arm64-v8a",
            Arch::Arm => "armeabi-v7a",
            Arch::X86_64 => "x86_64",
        }
    }
    fn android_abi(&self) -> &'static str {
        match self {
            Arch::Arm64 => "aarch64-linux-android",
            Arch::Arm => "armv7-linux-androideabi",
            Arch::X86_64 => "x86_64-linux-android",
        }
    }
}

#[derive(Parser)]
#[command(name = "xtask")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Build {
        #[arg(long)]
        release: bool,
        #[arg(long)]
        skip_webui: bool,
        #[arg(long, value_enum)]
        arch: Option<Arch>,
    },
    Lint,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Build {
            release,
            skip_webui,
            arch,
        } => {
            build_full(release, skip_webui, arch)?;
        }
        Commands::Lint => {
            run_clippy()?;
        }
    }
    Ok(())
}

fn run_clippy() -> Result<()> {
    println!(":: Running Clippy...");

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());

    let status = Command::new(cargo)
        .args([
            "clippy",
            "--workspace",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ])
        .status()
        .context("Failed to run cargo clippy")?;

    if !status.success() {
        anyhow::bail!("Clippy found issues! Please fix them before committing.");
    }

    println!(":: Clippy checks passed!");
    Ok(())
}

fn build_full(release: bool, skip_webui: bool, target_arch: Option<Arch>) -> Result<()> {
    let output_dir = Path::new("output");
    let stage_dir = output_dir.join("staging");
    if output_dir.exists() {
        fs::remove_dir_all(output_dir)?;
    }
    fs::create_dir_all(&stage_dir)?;
    let version = get_version()?;
    if !skip_webui {
        println!(":: Building WebUI...");
        build_webui(&version)?;
    }

    let archs_to_build = if let Some(selected) = target_arch {
        vec![selected]
    } else {
        vec![Arch::Arm64, Arch::Arm, Arch::X86_64]
    };

    for arch in archs_to_build {
        println!(":: Compiling Core for {:?}...", arch);
        compile_core(release, arch)?;
        let bin_name = "meta-hybrid";
        let profile = if release { "release" } else { "debug" };
        let src_bin = Path::new("target")
            .join(arch.android_abi())
            .join(profile)
            .join(bin_name);
        let stage_bin_dir = stage_dir.join("binaries").join(arch.target());
        fs::create_dir_all(&stage_bin_dir)?;
        if src_bin.exists() {
            file::copy(
                &src_bin,
                stage_bin_dir.join(bin_name),
                &file::CopyOptions::new().overwrite(true),
            )?;
        } else {
            println!("Warning: Binary not found at {}", src_bin.display());
        }
    }
    println!(":: Copying module scripts...");
    let module_src = Path::new("module");
    let options = dir::CopyOptions::new().overwrite(true).content_only(true);
    dir::copy(module_src, &stage_dir, &options)?;
    let gitignore = stage_dir.join(".gitignore");
    if gitignore.exists() {
        fs::remove_file(gitignore)?;
    }
    println!(":: Injecting version: {}", version);
    update_module_prop(&stage_dir.join("module.prop"), &version)?;
    println!(":: Creating Zip...");
    let zip_file = output_dir.join(format!("Meta-Hybrid-{}.zip", version));
    let zip_options = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(9));
    zip_create_from_directory_with_options(&zip_file, &stage_dir, |_| zip_options)?;
    println!(":: Build Complete: {}", zip_file.display());

    Ok(())
}

fn build_webui(version: &str) -> Result<()> {
    generate_webui_constants(version)?;
    let webui_dir = Path::new("webui");
    let pnpm = if cfg!(windows) { "pnpm.cmd" } else { "pnpm" };
    let status = Command::new(pnpm)
        .current_dir(webui_dir)
        .arg("install")
        .status()?;
    if !status.success() {
        anyhow::bail!("pnpm install failed");
    }
    let status = Command::new(pnpm)
        .current_dir(webui_dir)
        .args(["run", "build"])
        .status()?;
    if !status.success() {
        anyhow::bail!("pnpm run build failed");
    }
    Ok(())
}

fn generate_webui_constants(version: &str) -> Result<()> {
    let path = Path::new("webui/src/lib/constants_gen.ts");
    let content = format!(
        r#"
export const APP_VERSION = "{version}";
export const RUST_PATHS = {{
  CONFIG: "/data/adb/meta-hybrid/config.toml",
  MODE_CONFIG: "/data/adb/meta-hybrid/module_mode.conf",
  IMAGE_MNT: "/data/adb/meta-hybrid/mnt",
  DAEMON_STATE: "/data/adb/meta-hybrid/run/daemon_state.json",
  DAEMON_LOG: "/data/adb/meta-hybrid/daemon.log",
}} as const;
export const BUILTIN_PARTITIONS = ["system", "vendor", "product", "system_ext", "odm", "oem", "apex"] as const;
"#
    );
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    let old_path = Path::new("webui/src/lib/constants_gen.js");
    if old_path.exists() {
        let _ = fs::remove_file(old_path);
    }
    Ok(())
}

fn compile_core(release: bool, arch: Arch) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args([
        "ndk",
        "--platform",
        "31",
        "-t",
        arch.target(),
        "build",
        "-Z",
        "build-std",
    ])
    .env("RUSTFLAGS", "-C default-linker-libraries");
    if release {
        cmd.arg("-r");
    }
    let mut ret = cmd.spawn()?;
    let status = ret.wait()?;
    if !status.success() {
        anyhow::bail!("Compilation failed for {}", arch.target());
    }
    Ok(())
}

fn get_version() -> Result<String> {
    let toml = fs::read_to_string("Cargo.toml")?;
    let data: CargoConfig = toml::from_str(&toml)?;
    Ok(format!("{}-{}", data.package.version, cal_git_code()?))
}

fn cal_git_code() -> Result<i32> {
    Ok(String::from_utf8(
        Command::new("git")
            .args(["rev-list", "--count", "HEAD"])
            .output()?
            .stdout,
    )?
    .trim()
    .parse::<i32>()?)
}

fn update_module_prop(path: &Path, version: &str) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let content = fs::read_to_string(path)?;
    let mut new_lines = Vec::new();
    let code = if let Ok(env_code) = env::var("META_HYBRID_CODE") {
        env_code
    } else {
        use std::{
            collections::hash_map::DefaultHasher,
            hash::{Hash, Hasher},
        };
        let mut hasher = DefaultHasher::new();
        version.hash(&mut hasher);
        ((hasher.finish() % 100000) as u32).to_string()
    };
    for line in content.lines() {
        if line.starts_with("version=") {
            new_lines.push(format!("version={}", version));
        } else if line.starts_with("versionCode=") {
            new_lines.push(format!("versionCode={}", code));
        } else {
            new_lines.push(line.to_string());
        }
    }
    fs::write(path, new_lines.join("\n"))?;
    Ok(())
}
