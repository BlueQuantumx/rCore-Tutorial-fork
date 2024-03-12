use std::fs::{read_dir, File};
use std::io::{Result, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use lazy_static::lazy_static;

lazy_static! {
    static ref TARGET_PATH: String = {
        let mut path = PathBuf::from(std::env::var("OUT_DIR").unwrap());
        path.pop();
        path.pop();
        path.pop();
        String::from(path.to_str().unwrap())
    };
}
static CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

fn main() {
    println!(
        "cargo:rustc-link-arg=-T{}/src/linker-qemu.ld",
        CARGO_MANIFEST_DIR
    );
    println!("cargo:rerun-if-changed=../user_apps/");
    println!("cargo:rerun-if-changed=../user_lib/");
    insert_app_data().unwrap();
}

fn insert_app_data() -> Result<()> {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("link_apps.S");
    let mut f = File::create(&dest_path).unwrap();
    let mut apps: Vec<_> = read_dir("../user_apps/src/bin")?
        .into_iter()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
            name_with_ext
        })
        .collect();
    apps.sort();

    writeln!(
        f,
        r#"
    .align 3
    .section .data
    .global _num_app
_num_app:
    .quad {}"#,
        apps.len()
    )?;

    for i in 0..apps.len() {
        writeln!(f, r#"    .quad app_{}_start"#, i)?;
    }
    writeln!(f, r#"    .quad app_{}_end"#, apps.len() - 1)?;

    for (idx, app) in apps.iter().enumerate() {
        println!("app_{}: {}", idx, app);
        Command::new("rust-objcopy")
            .arg("--binary-architecture=riscv64")
            .arg(format!("{}/{}", TARGET_PATH.as_str(), app))
            .args(["--strip-all", "-O", "binary"])
            .arg(format!("{}/{}.bin", TARGET_PATH.as_str(), app))
            .output()
            .unwrap();
        writeln!(
            f,
            r#"
    .section .data
    .global app_{0}_start
    .global app_{0}_end
app_{0}_start:
    .incbin "{2}/{1}.bin"
app_{0}_end:"#,
            idx,
            app,
            TARGET_PATH.as_str()
        )?;
    }
    Ok(())
}
