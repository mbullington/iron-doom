use rust_shell::cmd;

fn main() {
    // Add Homebrew to the linker paths.
    println!("cargo:rustc-link-search=/opt/homebrew/lib");

    // Compile the Slang shaders.
    // Assumes slangc is in the PATH.

    compile_slangc("src/renderer/shaders/sector.slang", "vs_main");
    compile_slangc("src/renderer/shaders/sector.slang", "ps_main");

    compile_slangc("src/renderer/shaders/wall.slang", "vs_main");
    compile_slangc("src/renderer/shaders/wall.slang", "ps_main");
}

fn compile_slangc(input: &str, entry: &str) {
    let result = cmd!(&format!("slangc {} -target wgsl -entry {}", input, entry)).stdout_utf8();

    if result.is_ok() {
        let result = result.unwrap();

        // Replace all @align(4,8,16,32) with "".
        let result = result.replace("@align(4)", "");
        let result = result.replace("@align(8)", "");
        let result = result.replace("@align(16)", "");
        let result = result.replace("@align(32)", "");

        // Write the result to a file.
        let output = input.replace(".slang", &format!("_{}.wgsl", entry));
        std::fs::write(&output, result).unwrap();
    }
}
