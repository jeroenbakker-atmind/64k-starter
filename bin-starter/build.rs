use std::{
    env,
    ffi::OsStr,
    fs::{self, File},
    path::{Path, PathBuf},
    process::Command,
};

use walkdir::WalkDir;

const SHADER_MINIFIER_VERSION: &'static str = "1.5.0";

fn manifest_dir() -> PathBuf {
    Path::new(&env::var_os("CARGO_MANIFEST_DIR").unwrap()).to_path_buf()
}

/// Path to the downloaded Shader_Minifier binary. The released asset is a
/// single PE file that is also a valid Mono/.NET assembly, so the same file
/// runs natively on Windows and (via `mono`) on macOS.
fn shader_minifier_path() -> PathBuf {
    manifest_dir().join(format!(
        "target/shader_minifier_v{}.exe",
        SHADER_MINIFIER_VERSION
    ))
}

fn shader_sources() -> Vec<PathBuf> {
    WalkDir::new(manifest_dir())
        .into_iter()
        .filter_map(|x| x.ok())
        .filter(|x| {
            let extensions: &[&str] = &["glsl", "frag", "vert"];
            if let Some(extension) = x.path().extension().and_then(OsStr::to_str) {
                return extensions.iter().any(|x| x.eq_ignore_ascii_case(extension));
            }

            false
        })
        .map(|x| x.into_path())
        .collect()
}

fn ensure_shader_minifier_exists() {
    let path = shader_minifier_path();
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut response = reqwest::blocking::get(format!(
            "https://github.com/laurentlb/Shader_Minifier/releases/download/{}/shader_minifier.exe",
            SHADER_MINIFIER_VERSION
        ))
        .unwrap();

        let mut file = File::create(&path).unwrap();
        response.copy_to(&mut file).unwrap();
    }
}

/// Runs Shader_Minifier (directly on Windows, via `mono` on macOS) to produce
/// `src/glsl.rs`, so both platforms share the exact same minified output.
fn minify_shaders(sources: &[PathBuf]) {
    ensure_shader_minifier_exists();

    let path = shader_minifier_path();
    let output = Path::new(&manifest_dir()).join("src").join("glsl.rs");

    let mut cmd = Command::new(match env::var("CARGO_CFG_TARGET_OS").as_deref() {
        Ok("windows") => path.clone().into_os_string(),
        // Shader_Minifier is a Mono/.NET assembly; macOS runs it via mono.
        _ => "mono".into(),
    });
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        cmd.arg(&path);
    }
    cmd.args(["-o", output.to_str().unwrap()])
        .args(["--format", "rust"])
        .args(sources.iter().map(|x| x.as_os_str()))
        .output()
        .unwrap();
}

/// Regenerates `src/song.bin` by running the music composition from the `song`
/// library. This runs on the host every build, so the tune stays in sync with
/// any changes to the composition without a nested `cargo` process.
fn generate_song() {
    let out = manifest_dir().join("src").join("song.bin");
    let data = song::compose::compose();
    fs::write(&out, &data).unwrap();
    // Re-run whenever any of the composition inputs change.
    for entry in WalkDir::new(manifest_dir().join("../../lib-song/src")) {
        if let Ok(e) = entry {
            if e.file_type().is_file() {
                println!("cargo:rerun-if-changed={}", e.path().display());
            }
        }
    }
}

fn main() {
    generate_song();
    let sources = shader_sources();
    minify_shaders(&sources);

    for src in &sources {
        println!("cargo:rerun-if-changed={}", src.display());
    }

    match env::var("CARGO_CFG_TARGET_OS").as_deref() {
        Ok("windows") => {
            println!("cargo:rustc-link-arg-bins=/DEBUG:NONE");
            println!("cargo:rustc-link-arg-bins=/EMITPOGOPHASEINFO");
            println!("cargo:rustc-link-arg-bins=/MERGE:.pdata=.text");
            println!("cargo:rustc-link-arg-bins=/MERGE:.rdata=.text");
            println!("cargo:rustc-link-arg-bins=/NODEFAULTLIB");
        }
        Ok("macos") => {
            for framework in [
                "Cocoa",
                "OpenGL",
                "CoreGraphics",
                "CoreFoundation",
                "AudioToolbox",
                "CoreAudio",
            ] {
                println!("cargo:rustc-link-lib=framework={framework}");
            }
            println!("cargo:rustc-link-arg-bins=-Wl,-dead_strip");
            println!("cargo:rustc-link-arg-bins=-Wl,-x");
        }
        _ => {}
    }
}
