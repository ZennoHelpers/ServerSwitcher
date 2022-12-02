use std::fs::File;
use std::io::Write;
use std::path::{PathBuf};
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
    
    // r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64"
    let path_with_exe = PathBuf::from(env!("WIN_KITS_BIN_X64_PATH"));
    
    if path_with_exe.exists() && path_with_exe.is_dir() {
        let authors = env!("CARGO_PKG_AUTHORS").replace(':', ", ");
    
        let crate_toml_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    
        let version = env!("CARGO_PKG_VERSION");
        let version_with_commas = version.replace('.', ",");
    
        let pkg_name = env!("CARGO_PKG_NAME");
        let pkg_description = env!("CARGO_PKG_DESCRIPTION");
    
        let manifest = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly manifestVersion="1.0" xmlns="urn:schemas-microsoft-com:asm.v1" xmlns:asmv3="urn:schemas-microsoft-com:asm.v3">
    <assemblyIdentity
        name="{pkg_name}.exe"
        version="{version}.0"
        type="win32"
        processorArchitecture="amd64"
    />
    <description>{pkg_description}</description>
    <!-- <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
        <security>
            <requestedPrivileges> -->
                <!-- 'requestedExecutionLevel' conflict with '/MANIFESTINPUT' and requires same '/MANIFESTUAC' or remove 'requestedExecutionLevel' block -->
                <!-- <requestedExecutionLevel
                    level="requireAdministrator"
                    uiAccess="false"
                />   
            </requestedPrivileges>
        </security>
    </trustInfo> -->
</assembly>"#);
        
        let manifest_fullname = &format!("{pkg_name}.exe.manifest");
    
        create_and_write_file(&crate_toml_path, manifest_fullname, &manifest);
        
        run(&path_with_exe, "mt.exe", &["/nologo", "/validate_manifest", "/check_for_duplicates", "/manifest", manifest_fullname]);
    
        set_linker_args(&[
            "/MANIFEST:EMBED", // requires for '/MANIFESTINPUT'
            &format!("/MANIFESTINPUT:{pkg_name}.exe.manifest"), // set default 'requestedExecutionLevel' to tell UAC doesn't emulate paths like for the old application
            "/MANIFESTUAC:level='requireAdministrator'", // set custom 'requestedExecutionLevel' in manifest
        ]);
    
        let rc_data = format!(
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
        );
        
        let rc_fullname = "res.rc";
    
        create_and_write_file(&crate_toml_path, rc_fullname, &rc_data);
    
        run(&path_with_exe, "rc.exe", &["/nologo", "/v", rc_fullname]);
        
        let mut res_path = crate_toml_path.clone();
        res_path.push("res.res");
    
        println!(r#"cargo:rustc-link-arg={}"#, res_path.to_str().unwrap());
    } else {
        println!(r"cargo:warning=Path to 'C:\Program Files (x86)\Windows Kits\10\bin\**ver**\x64' not specified")
    }
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

fn create_and_write_file(path_to_dir: &PathBuf, file_fullname: &str, data: &str) {
    let mut file_path = path_to_dir.clone();
    file_path.push(file_fullname);
    
    let mut file = File::options()
        .write(true)
        .truncate(true)
        .create(true)
        .open(file_path)
        .unwrap();
    file.write_all(data.as_bytes()).unwrap();
}

fn run(path_with_exe: &PathBuf, exe_fullname: &str, args: &[&str]) {
    let mut exe_path = path_with_exe.clone();
    exe_path.push(exe_fullname);
    
    let rc_proc = Command::new(exe_path)
        .args(args)
        .spawn()
        .unwrap();
    
    let result = rc_proc.wait_with_output().unwrap();
    if !result.status.success() {
        panic!("{}", String::from_utf8_lossy(result.stdout.as_slice()))
    }
}