// build.rs
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // 设置SQLite编译标志以启用FTS5
    std::env::set_var("LIBSQLITE3_FLAGS", "-DSQLITE_ENABLE_FTS5");

    // 告诉链接器连接信号tokenizer的入口点
    println!("cargo:rustc-link-arg=-Wl,-u,_signal_fts5_tokenizer_init");
    //let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let signal_fts5_dir = PathBuf::from("../signal-fts5");

    println!("cargo:rerun-if-changed={}", signal_fts5_dir.display());

    // 编译Signal FTS5扩展为静态库
    let output = Command::new("cargo")
        .args([
            "rustc",
            "--manifest-path",
            &signal_fts5_dir.join("Cargo.toml").to_string_lossy(),
            "--features",
            "extension",
            "--release",
            "--crate-type=staticlib",
        ])
        .output()
        .expect("Failed to compile Signal FTS5 extension");

    if !output.status.success() {
        panic!(
            "Failed to compile Signal FTS5 extension: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // 获取编译输出的静态库路径，优先使用 CARGO_TARGET_DIR，否则使用 workspace 根目录下的 target
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let profile = env::var("PROFILE").unwrap();
    let target_dir = env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| manifest_dir.parent().unwrap().join("target"))
        .join(&profile)
        .canonicalize()
        .expect("Failed to canonicalize target directory");

    // 检查静态库是否存在
    let lib_name = if cfg!(target_os = "windows") {
        "signal_tokenizer.lib"
    } else {
        "libsignal_tokenizer.a"
    };

    let lib_path = target_dir.join(lib_name);
    if !lib_path.exists() {
        panic!("Static library not found at: {}", lib_path.display());
    }

    // 告诉链接器库的位置
    println!("cargo:rustc-link-search=native={}", target_dir.display());
    println!("cargo:rustc-link-lib=static=signal_tokenizer");

    // Windows特定的链接设置
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=dylib=ws2_32");
        println!("cargo:rustc-link-lib=dylib=userenv");
    }
}
