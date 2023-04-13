use slint_build::CompilerConfiguration;

fn main() {
    let config = CompilerConfiguration::new().with_style("material".into());

    slint_build::compile_with_config("ui/window.slint", config)
        .expect("Failed to build ui modules");
}
