# metalr-build

metalr-build is a way to build [Metal](https://developer.apple.com/metal/) GPU shaders from Rust/cargo `build.rs` scripts.

Based on the [buildkit](https://github.com/drewcrawford/buildkit) micro build system, metalr-build provides the following features:
* debug and release builds that makes sense and integrate with cargo
* skipping builds if no sourcefiles were changed.  Other features like incremental builds and parallel builds are planned upstream, see [buildkit](https://github.com/drewcrawford/buildkit)
* One-line integration
* Modern, xcode-like compile flags
* Free for noncommercial and "small commercial" use

# Use
1. Place your `.metal` files in `src/`
2. Call `BuildSystem::build_rs`

# See also
metalr-build plays well with
* [buildkit](https://github.com/drewcrawford/buildkit)
* [metalr](https://github.com/drewcrawford/metalr)

