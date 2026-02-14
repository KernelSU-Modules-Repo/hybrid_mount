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
        #[arg(long)]
        ci: bool,
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
            ci,
        } => {
            let (cargo_release, webui_release, target_archs) = if ci {
                (true, false, vec![Arch::Arm64])
            } else {
                let archs = if let Some(selected) = arch {
                    vec![selected]
                } else {
                    vec![Arch::Arm64, Arch::Arm, Arch::X86_64]
                };
                (release, release, archs)
            };

            build_full(cargo_release, webui_release, skip_webui, target_archs)?;
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

fn build_full(
    cargo_release: bool,
    webui_release: bool,
    skip_webui: bool,
    target_archs: Vec<Arch>,
) -> Result<()> {
    let output_dir = Path::new("output");
    let stage_dir = output_dir.join("staging");
    if output_dir.exists() {
        fs::remove_dir_all(output_dir)?;
    }
    fs::create_dir_all(&stage_dir)?;
    let version = get_version()?;
    if !skip_webui {
        println!(":: Building WebUI...");
        build_webui(&version, webui_release)?;
    }

    for arch in target_archs {
        println!(":: Compiling Core for {:?}...", arch);
        compile_core(cargo_release, arch)?;
        let bin_name = "hybrid-mount";
        let profile = if cargo_release { "release" } else { "debug" };
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
    println!(":: Creating Zip...");
    let zip_file = output_dir.join(format!("Hybrid-Mount-{}.zip", version));
    let zip_options = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(9));
    zip_create_from_directory_with_options(&zip_file, &stage_dir, |_| zip_options)?;
    println!(":: Build Complete: {}", zip_file.display());

    Ok(())
}

fn build_webui(version: &str, is_release: bool) -> Result<()> {
    generate_webui_constants(version, is_release)?;
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

fn generate_webui_constants(version: &str, is_release: bool) -> Result<()> {
    let path = Path::new("webui/src/lib/constants_gen.ts");
    let content = format!(
        r#"
export const APP_VERSION = "{version}";
export const IS_RELEASE = {is_release};
export const RUST_PATHS = {{
  CONFIG: "/data/adb/hybrid-mount/config.toml",
  MODE_CONFIG: "/data/adb/hybrid-mount/module_mode.conf",
  IMAGE_MNT: "/data/adb/hybrid-mount/mnt",
  DAEMON_STATE: "/data/adb/hybrid-mount/run/daemon_state.json",
  DAEMON_LOG: "/data/adb/hybrid-mount/daemon.log",
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
