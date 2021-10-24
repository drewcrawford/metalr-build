
mod compile;
mod link;

pub use buildkit::{BuildSettingsBuilder,SourceFileStrategy,Configuration,BuildSettings};

pub type BuildSystem = buildkit::BuildSystem<compile::MetalCompiler,link::MetalLinker>;



#[test]
fn t_build() {
    use buildkit::PathType;

    use std::env::temp_dir;
    let mut intermediate_path = temp_dir();
    intermediate_path.push("t_build");

    let mut settings = BuildSettingsBuilder::new();

    settings
        .source_strategy(SourceFileStrategy::SearchFromManifest("tests/".to_string()))
        .intermediate_path(intermediate_path.clone())
        .product_path(PathType::Exact(intermediate_path));

    let mut debug_settings = settings.clone();
    debug_settings.configuration(Configuration::Debug);
    BuildSystem::build(&debug_settings.finish());

    let mut release_settings = settings.clone();
    release_settings.configuration(Configuration::Release);
    BuildSystem::build(&release_settings.finish());

}