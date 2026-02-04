use std::path::PathBuf;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir = PathBuf::from(out_dir);

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_dir = PathBuf::from(manifest_dir);

    let minhook_src_dir = manifest_dir.join("minhook-detours").join("src");
    println!("cargo:rerun-if-changed={}", minhook_src_dir.display());

    let phnt_dir = minhook_src_dir.join("phnt");
    let slimdetours_dir = minhook_src_dir.join("SlimDetours");

    cc::Build::new()
        .include(&phnt_dir)
        .include(&minhook_src_dir)
        .file(minhook_src_dir.join("MinHook.c"))
        .file(slimdetours_dir.join("Trampoline.c"))
        .file(slimdetours_dir.join("Transaction.c"))
        .file(slimdetours_dir.join("Thread.c"))
        .file(slimdetours_dir.join("Memory.c"))
        .file(slimdetours_dir.join("Instruction.c"))
        .file(slimdetours_dir.join("InlineHook.c"))
        .file(slimdetours_dir.join("Disassembler.c"))
        .compile("MinHook");

    bindgen::Builder::default()
        .header(minhook_src_dir.join("MinHook.h").to_string_lossy())
        .allowlist_type("MH_.*")
        .allowlist_function("MH_.*")
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .merge_extern_blocks(true)
        .generate_comments(true)
        .clang_arg("-fparse-all-comments")
        // Re-export MH_STATUS::* so that users can access the constants directly
        // while also having the MH_STATUS namespace available.
        .raw_line("pub use MH_STATUS::*;")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
