use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use buildkit::{Configuration, CompileStep};

pub struct MetalCompiler;

impl CompileStep for MetalCompiler {
    const SOURCE_FILE_EXTENSION: &'static str = "metal";

    fn compile_one(path: &Path, intermediate_dir: &Path, configuration: &Configuration, dependency_path: &Path) -> PathBuf {
        let file_with_extension = path.file_name().unwrap();
        let file_base_name = path.file_stem().unwrap();
        let mut new_name = file_base_name.to_owned();
        new_name.push(".air");
        let mut new_file = PathBuf::from(intermediate_dir);
        new_file.push(new_name);
        let mut cmd = Command::new("xcrun");
        let compile_1 = cmd.arg("-sdk").arg("macosx")    //sdk
            .arg("metal")
            .arg("-c")                      //compile
            .args(&["-target","air64-apple-macos12.1"]); //deployment target?
        match configuration {
            Configuration::Debug => {
                compile_1.arg("-gline-tables-only") //"Emit debug line number tables only"
                    .arg("-MO") //"Embed sources and driver options into output"

                ;
            }
            Configuration::Release => (),
        };

        let compile_output = compile_1
            //-isysroot-isysroot /Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX12.1.sdk
            .arg("-ffast-math") //Allow aggressive, lossy floating-point optimizations
            //-serialize-diagnostics <file>
            .args(&["-o",new_file.to_str().unwrap()])
            //-index-store-path <file>
            .arg("-MMD") //write a depfile containing user headers
            .args(&["-MT",file_with_extension.to_str().unwrap()]) //Specify name of main file output in depfile
            .args(&["-MF",dependency_path.to_str().unwrap()]) //Write depfile output from -MMD, -MD, -MM, or -M to <file>
            .arg(path) //input file
            .stdout(Stdio::piped()).stderr(Stdio::piped())
            .spawn().unwrap().wait_with_output().unwrap();
        if !compile_output.status.success() {
            panic!(
                r#"
Metal compiler reported an error.
stdout: {}
stderr: {}
"#,
                String::from_utf8(compile_output.stdout).unwrap(),
                String::from_utf8(compile_output.stderr).unwrap()
            );
        }
        new_file
    }
}

