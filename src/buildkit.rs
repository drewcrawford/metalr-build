//! This is a lightweight buildsystem that will eventually be extracted to its own crate.
//!
//!
//!
use std::path::{Path, PathBuf};
use std::marker::PhantomData;
use std::ffi::OsString;
use std::str::FromStr;
use std::fs::create_dir_all;

#[derive(Copy,Clone)]
pub enum Configuration {
    Debug,
    Release
}

pub trait CompileStep {
    ///The extension to scan for.  We will use this to create individual `CompileStep`.
    const SOURCE_FILE_EXTENSION: &'static str;

    ///Compile one file, placing the output in the intermediate dir.
    ///
    /// # Returns a path to the compiled object file, should be located in the intermediate dir.
    fn compile_one(path: &Path, intermediate_dir: &Path, configuration: &Configuration,dependency_path: &Path) -> PathBuf;
}

pub trait LinkStep {
    fn link_all(object_files: &[PathBuf], out_dir: &Path, lib_name: &str, configuration: &Configuration) -> PathBuf;
}

pub struct BuildSettings {
    ///Will scan this path for sourcefiles
    source_strategy: SourceFileStrategy,
    ///Path for output/intermediates
    intermediate_path: PathBuf,
    ///The "final path" for the product
    product_path: PathBuf,
    ///Whether debug/release
    configuration: Configuration,
    ///The product name, e.g. libname or similar
    product_name: String,
}

///How to find sourcefiles for compiling
#[derive(Clone)]
pub enum SourceFileStrategy {
    ///Use exactly the sourcefiles specified.
    SourceFiles(Vec<PathBuf>),
    ///Search recursively in this path, starting from the manifest directory. e.g. payload like "src/"
    ///
    /// Note that if this path is absolute, we will search the absolute path instead.
    SearchFromManifest(String)
}

impl SourceFileStrategy {
    fn resolve<C: CompileStep>(&self) -> Vec<PathBuf> {
        match self {
            SourceFileStrategy::SourceFiles(paths) => paths.into_iter().map(|f| f.clone()).collect(),
            SourceFileStrategy::SearchFromManifest(manifest_path) => {
                let manifest_string = std::env::var("CARGO_MANIFEST_DIR").unwrap();
                let mut m_path = PathBuf::from_str(&manifest_string).unwrap();
                m_path.push(manifest_path);
                let mut vec = Vec::new();
                dir_walk(&m_path, C::SOURCE_FILE_EXTENSION,&mut vec);
                vec
            }
        }
    }
}

impl BuildSettings {
    ///builder-pattern for `BuildSettings`
    pub fn build<'a>() -> BuildSettingsBuilder { BuildSettingsBuilder::new() }

    ///Automatically builds all build settings.
    pub fn auto() -> BuildSettings { BuildSettingsBuilder::new().finish() }
}

///Actual build system, specialized via compiler/linker
pub struct BuildSystem<Compiler,Linker> {
    compiler: PhantomData<Compiler>,
    linker: PhantomData<Linker>,
}

///Walks a directory, looking for .metal files
///
/// Returns its output in its argument, because it makes the memory
/// faster for recursion
fn dir_walk(base: &Path, extension: &str, output: &mut Vec<PathBuf>) {
    for item in std::fs::read_dir(base).expect(&format!("Problem reading dir at {:?}",base)) {
        let path = item.unwrap().path();
        if path.is_dir() {
            dir_walk(&path, extension, output);
        }
        else if path.is_file() { //I'm not 100% sure what other options there are, but ok
            if path.extension() == Some(&OsString::from_str(extension).unwrap()) {
                output.push(path);
            }
        }
    }
}

impl<Compiler: CompileStep,Linker: LinkStep> BuildSystem<Compiler,Linker> {
    ///Compiles/links using the settings specified.
    ///
    /// Returns a path to the final product.
    pub fn build(settings: &BuildSettings) -> PathBuf {
        let source_files = settings.source_strategy.resolve::<Compiler>();
        if source_files.is_empty() { panic!("Nothing to compile!") }
        //todo: multithread this?
        //todo: Incremental compiles?
        //create intermediate path if it does not exist
        create_dir_all(&settings.intermediate_path).unwrap();
        let mut dependency_path = settings.intermediate_path.clone();
        dependency_path.push("dependency");
        let mut object_files = Vec::new();
        for source_file in source_files {
            let object_file = Compiler::compile_one(&source_file, &settings.intermediate_path,  &settings.configuration, &dependency_path);
            object_files.push(object_file);
        }
        //todo: Do all compilers write dependency files in the same way?
        super::dependency_parser::tell_cargo_about_dependencies(dependency_path);

        Linker::link_all(&object_files, &settings.product_path,&settings.product_name,  &settings.configuration)
    }

    ///Build using no special settings.  Usually the entrypoint from `build.rs`
    pub fn build_rs(exe_path: PathBuf) -> PathBuf {
        let settings = BuildSettingsBuilder::new().product_path(PathType::EXERelative(exe_path)).finish();
        Self::build(&settings)
    }
}

#[non_exhaustive]
#[derive(Clone)]
pub enum PathType {
    ///Path will take on some path relative to exe in target directory as part of a build process
    EXERelative(PathBuf),
    ///Path will be as specified
    Exact(PathBuf),
}
impl PathType {
    fn path(&self) -> PathBuf {
        match self {
            PathType::EXERelative(relative) => {
                let out_dir = std::env::var("OUT_DIR").expect("Must set `OUT_DIR` if not setting product_path or using PathType::EXERelative");
                let mut product_path = PathBuf::from_str(&out_dir).unwrap();
                product_path.pop(); //out
                product_path.pop(); //target_name
                product_path.pop(); //'build'
                product_path.push(relative);
                product_path
            }
            PathType::Exact(exact) => exact.to_path_buf(),
        }
    }
}

///Builder pattern for [BuildSettings]
///
/// https://doc.rust-lang.org/1.0.0/style/ownership/builders.html
#[derive(Clone)]
pub struct BuildSettingsBuilder{
    ///Will scan this path for sourcefiles
    source_strategy: Option<SourceFileStrategy>,
    intermediate_path: Option<PathBuf>,
    product_path: Option<PathType>,
    configuration: Option<Configuration>,
    //todo: Allow other types to be set
}


impl BuildSettingsBuilder {
    pub fn new() -> Self {
        BuildSettingsBuilder{ source_strategy: None,configuration:None,intermediate_path: None, product_path: None}
    }
    pub fn source_strategy(&mut self,strategy: SourceFileStrategy) -> &mut BuildSettingsBuilder {
        self.source_strategy = Some(strategy);
        self
    }
    pub fn configuration(&mut self, configuration: Configuration) -> &mut BuildSettingsBuilder {
        self.configuration = Some(configuration);
        self
    }
    pub fn intermediate_path(&mut self, path: PathBuf) -> &mut BuildSettingsBuilder {
        self.intermediate_path = Some(path);
        self
    }
    ///Specify where products are stored
    pub fn product_path(&mut self, path: PathType) -> &mut BuildSettingsBuilder {
        self.product_path = Some(path);
        self
    }
    pub fn finish(&self) -> BuildSettings {
        let source_strategy = match &self.source_strategy {
            None => {
                SourceFileStrategy::SearchFromManifest("src".to_owned())
            }
            Some(strategy) => strategy.clone()
        };

        let intermediate_path = match &self.intermediate_path {
            Some(path) => path.clone(),
            None => {
                let out_dir = std::env::var("OUT_DIR").expect("Must set `OUT_DIR` environment variable, or call `.intermediate_path()`");
                PathBuf::from_str(&out_dir).unwrap()
            }
        };

        let product_path: PathBuf = match &self.product_path {
            Some(path) => path.path().to_path_buf(),
            None => {
                PathType::EXERelative(PathBuf::new()).path().to_path_buf()
            }
        };

        let configuration = match self.configuration {
            Some(config) => config,
            None => if std::env::var("DEBUG").expect("Must set DEBUG environment variable or call .configuration()") == "1" { Configuration::Debug } else { Configuration::Release }
        };

        let product_name = std::env::var("CARGO_PKG_NAME").unwrap();
        BuildSettings {
            source_strategy,
            intermediate_path,
            product_path,
            product_name,
            configuration,

        }
    }
}