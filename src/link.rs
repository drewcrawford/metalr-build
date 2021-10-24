use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use buildkit::{Configuration, LinkStep};

pub struct MetalLinker {

}
impl LinkStep for MetalLinker {
    fn link_all(object_files: &[PathBuf], out_dir: &Path, lib_name: &str, configuration: &Configuration) -> PathBuf {
        //create path if needed
        if !out_dir.exists() {
            std::fs::create_dir_all(out_dir).unwrap();
        }
        let mut metallib_file = out_dir.to_owned();
        let metallib_nameonly = format!("{}.metallib",lib_name);
        metallib_file.push(metallib_nameonly);
        let mut cmd = Command::new("xcrun");
        cmd
            .arg("-sdk").arg("macosx")
            .arg("metal")
            .args(&["-target","air64-apple-macos11.3"]); //deployment target
        match configuration {
            Configuration::Debug => {
                cmd.arg("-MO"); //"Embed sources and driver options into output"
            }
            Configuration::Release => ()
        }
        let link_output = cmd.args(&["-o",metallib_file.to_str().unwrap()])
            .args(object_files)
            .stdout(Stdio::piped()).stderr(Stdio::piped())
            .spawn().unwrap().wait_with_output().unwrap();
        if !link_output.status.success() {
            panic!(
                r#"
Metal linker reported an error.
stdout: {}
stderr: {}
"#,
                String::from_utf8(link_output.stdout).unwrap(),
                String::from_utf8(link_output.stderr).unwrap()
            );
        }
        metallib_file
    }
}
