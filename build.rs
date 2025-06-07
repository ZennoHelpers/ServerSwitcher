use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    rerun_if_changed(&["build.rs"]);

    set_linker_args(&[
        "/NOLOGO",
        "/ALLOWBIND:NO",
        "/INFERASANLIBS:NO",
        "/CGTHREADS:1",
        "/VERBOSE",
        "/WX",
    ]);

    if let Some(mt_path) = find_exe_in_path("mt.exe") {
        if let Some(rc_path) = find_exe_in_path("rc.exe") {
            let authors = env!("CARGO_PKG_AUTHORS").replace(':', ", ");

            let crate_toml_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

            let version = env!("CARGO_PKG_VERSION");
            let version_with_commas = version.replace('.', ",");

            let pkg_name = env!("CARGO_PKG_NAME");
            let pkg_description = env!("CARGO_PKG_DESCRIPTION");

            let manifest = make_manifest(pkg_name, version, pkg_description);

            let manifest_fullname = &format!("{pkg_name}.exe.manifest");

            write_file(&crate_toml_path, manifest_fullname, &manifest);

            run(
                &mt_path,
                &[
                    "/nologo",
                    "/validate_manifest",
                    //"/validate_file_hashes",
                    "/check_for_duplicates",
                    "/manifest",
                    manifest_fullname,
                ],
            );

            set_linker_args(&[
                "/MANIFEST:EMBED",                                  // requires for '/MANIFESTINPUT'
                &format!("/MANIFESTINPUT:{pkg_name}.exe.manifest"), // set default 'requestedExecutionLevel' to tell UAC doesn't emulate paths like for the old application
                "/MANIFESTUAC:level='requireAdministrator'", // set custom 'requestedExecutionLevel' in manifest
            ]);

            let rc_data = make_rcdata(
                &authors,
                pkg_description,
                pkg_name,
                &version_with_commas,
                version,
            );

            let rc_fullname = "res.rc";

            write_file(&crate_toml_path, rc_fullname, &rc_data);

            run(&rc_path, &["/nologo", "/v", rc_fullname]);

            let mut res_path = crate_toml_path.clone();
            res_path.push("res.res");

            println!(r#"cargo:rustc-link-arg={}"#, res_path.to_str().unwrap());
        } else {
            sdk_not_set_warn()
        }
    } else {
        sdk_not_set_warn()
    }
}

fn sdk_not_set_warn() {
    println!(
        r"cargo:warning='C:\Program Files (x86)\Windows Kits\10\bin\**ver**\x64' not installed or not specified in PATH."
    )
}

fn rerun_if_changed(list: &[&str]) {
    for item in list {
        println!("cargo:rerun-if-changed={item}");
    }
}

fn set_linker_args(args: &[&str]) {
    for arg in args {
        println!("cargo:rustc-link-arg={arg}");
    }
}

fn write_file<P: AsRef<Path>>(path_to_dir: P, file_fullname: &str, data: &str) {
    let mut file_path = PathBuf::from(path_to_dir.as_ref());
    file_path.push(file_fullname);

    let mut file = File::options()
        .write(true)
        .truncate(true)
        .create(true)
        .open(file_path)
        .unwrap();
    file.write_all(data.as_bytes()).unwrap();
}

fn find_exe_in_path<P: AsRef<Path>>(exe_name: P) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .filter_map(|dir| {
                let full_path = dir.join(&exe_name);
                if full_path.is_file() {
                    Some(full_path)
                } else {
                    None
                }
            })
            .next()
    })
}

fn run<P: AsRef<Path>>(exe_path: P, args: &[&str]) {
    let rc_proc = Command::new(exe_path.as_ref()).args(args).spawn().unwrap();

    let result = rc_proc.wait_with_output().unwrap();
    if !result.status.success() {
        panic!("{}", String::from_utf8_lossy(result.stdout.as_slice()))
    }
}

fn make_manifest(pkg_name: &str, version: &str, pkg_description: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly manifestVersion="1.0" xmlns="urn:schemas-microsoft-com:asm.v1" xmlns:asmv3="urn:schemas-microsoft-com:asm.v3">
<assemblyIdentity
    name="{pkg_name}.exe"
    version="{version}.0"
    type="win32"
    processorArchitecture="amd64"
/>
<description>{pkg_description}</description>
</assembly>"#
    )

    // <!-- <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    //     <security>
    //         <requestedPrivileges> -->
    //             <!-- 'requestedExecutionLevel' conflict with '/MANIFESTINPUT' and requires same '/MANIFESTUAC' or remove 'requestedExecutionLevel' block -->
    //             <!-- <requestedExecutionLevel
    //                 level="requireAdministrator"
    //                 uiAccess="false"
    //             />
    //         </requestedPrivileges>
    //     </security>
    // </trustInfo> -->
}

fn make_rcdata(
    authors: &str,
    pkg_description: &str,
    pkg_name: &str,
    version_with_commas: &str,
    version: &str,
) -> String {
    format!(
        r#"#define VER_COMPANYNAME_STR         "{authors}\0"
#define VER_FILEDESCRIPTION_STR     "{pkg_description}\0"
#define VER_INTERNALNAME_STR        "{pkg_name}\0"

#define VER_ORIGINALFILENAME_STR   "{pkg_name}.exe\0"
#define VER_PRODUCTNAME_STR        "Server Switcher\0"

#define VERSION                     {version_with_commas}
#define VERSION_STR                 "{version}\0"

1 ICON "CodeCreator.ico"

1 VERSIONINFO
FILEVERSION     VERSION
PRODUCTVERSION  VERSION
BEGIN
BLOCK "StringFileInfo"
BEGIN
    // https://learn.microsoft.com/en-us/windows/win32/menurc/versioninfo-resource
    BLOCK "041904B0" // Ru
    BEGIN
        VALUE "CompanyName",      VER_COMPANYNAME_STR
        VALUE "FileDescription",  VER_FILEDESCRIPTION_STR
        VALUE "FileVersion",      VERSION_STR
        VALUE "InternalName",     VER_INTERNALNAME_STR
        VALUE "OriginalFilename", VER_ORIGINALFILENAME_STR
        VALUE "ProductName",      VER_PRODUCTNAME_STR
        VALUE "ProductVersion",   VERSION_STR
    END
END
BLOCK "VarFileInfo"
BEGIN
    // https://learn.microsoft.com/en-us/windows/win32/menurc/versioninfo-resource
    // VALUE "Translation", 0x409, 1200 // En
    VALUE "Translation", 0x419, 1200 // Ru
END
END"#
    )
}
