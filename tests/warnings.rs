use std::env::temp_dir;
use std::path::PathBuf;
use std::str::FromStr;
use buildkit::{BuildSettingsBuilder, CompileSettingsBuilder, Configuration, PathType, SourceFileStrategy};

#[test] fn warnings() {
    let temp_path = temp_dir();
    let mut intermediate_path = temp_path.clone();
    intermediate_path.push("intermediates");
    let mut product_path = temp_path;
    product_path.push("products");
    let compile_settings = CompileSettingsBuilder::new()
        .source_strategy(SourceFileStrategy::SourceFiles(vec![PathBuf::from_str("tests/warning.metal").unwrap()]))
        .intermediate_path(PathType::Exact(intermediate_path))
        .configuration(Configuration::Debug)
        .finish();
    let settings = BuildSettingsBuilder::new()
        .compile_settings(compile_settings)
        .product_path(PathType::Exact(product_path))
        .finish();
    let _builder = metalr_build::BuildSystem::build(&settings);

}