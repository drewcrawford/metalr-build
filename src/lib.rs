/*!
# metalr-build

metalr-build is a way to build [Metal](https://developer.apple.com/metal/) GPU shaders from Rust/cargo `build.rs` scripts.

Based on the [buildkit](https://github.com/drewcrawford/buildkit) micro build system, metalr-build provides the following features:
1. debug and release builds that makes sense and integrate with cargo
2. skipping builds if no sourcefiles were changed.  Other features like incremental builds and parallel builds are planned upstream, see [buildkit](https://github.com/drewcrawford/buildkit)
3. One-line integration
4. Modern, xcode-like compile flags

# Use
1. Place your `.metal` files in `src/`
2. Call [BuildSystem::build_rs]

# See also
metalr-build plays well with
* [buildkit](https://github.com/drewcrawford/buildkit)
* [metalr](https://github.com/drewcrawford/metalr)
*/
mod compile;
mod link;


pub use buildkit::{BuildSettingsBuilder, SourceFileStrategy, Configuration, BuildSettings};

pub type BuildSystem = buildkit::BuildSystem<compile::MetalCompiler,link::MetalLinker>;

#[test]
fn t_build() {
    use buildkit::PathType;
    use std::path::PathBuf;
    use std::str::FromStr;
    use std::env::temp_dir;
    let mut intermediate_path = temp_dir();
    intermediate_path.push("t_build");
    use buildkit::CompileSettingsBuilder;
    let mut compile_settings = CompileSettingsBuilder::new();
    compile_settings
        .source_strategy(SourceFileStrategy::SearchFromManifest(vec![PathBuf::from_str("tests/").unwrap()]))
        .intermediate_path(PathType::Exact(intermediate_path.clone()));

    let mut settings = BuildSettingsBuilder::new();

    settings
        .product_path(PathType::Exact(intermediate_path));

    let mut debug_compile = compile_settings.clone();
    debug_compile.configuration(Configuration::Debug);
    let mut debug_settings = settings.clone();
    debug_settings.compile_settings(debug_compile.finish());
    BuildSystem::build(&debug_settings.finish());

    let mut release_compile = compile_settings.clone();
    release_compile.configuration(Configuration::Release);
    let mut release = settings.clone();
    release.compile_settings(release_compile.finish());
    BuildSystem::build(&release.finish());

}