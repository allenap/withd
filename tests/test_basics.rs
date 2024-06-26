use tempfile::TempDir;

#[test]
fn test_run_in_existing_directory() {
    let target = TempDir::new().unwrap();
    let path_os = target.path().as_os_str();
    let output = withd([path_os, os("touch"), os("here")]);
    assert!(output.status.success());
    assert!(target.path().join("here").exists());
}

#[test]
fn test_run_in_non_existing_directory() {
    let target = TempDir::new().unwrap();
    let path = target.path().join("foo");
    let path_os = path.as_os_str();
    let output = withd([path_os, os("touch"), os("here")]);
    assert!(!output.status.success());
    assert!(!path.join("here").exists());
}

#[test]
fn test_run_with_create_in_non_existing_directory() {
    let target = TempDir::new().unwrap();
    let path = target.path().join("foo");
    let path_os = path.as_os_str();
    let output = withd([os("-c"), path_os, os("touch"), os("here")]);
    assert!(output.status.success());
    assert!(path.join("here").exists());
}

#[test]
fn test_run_temporary_in_existing_directory() {
    let target = TempDir::new().unwrap();
    let path = target.path().join("tmp.XXXXXX.dir");
    let path_os = path.as_os_str();
    let output = withd([os("-t"), path_os, os("touch"), os("../here")]);
    assert!(output.status.success());
    assert!(target.path().join("here").exists());
}

#[test]
fn test_run_temporary_in_non_existing_directory() {
    let target = TempDir::new().unwrap();
    let path = target.path().join("foo").join("tmp.XXXXXX.dir");
    let path_os = path.as_os_str();
    let output = withd([os("-t"), path_os, os("touch"), os("../here")]);
    assert!(!output.status.success());
    assert!(!target.path().join("foo").join("here").exists());
}

#[test]
fn test_run_temporary_with_create_in_non_existing_directory() {
    let target = TempDir::new().unwrap();
    let path = target.path().join("foo").join("tmp.XXXXXX.dir");
    let path_os = path.as_os_str();
    let output = withd([os("-tc"), path_os, os("touch"), os("../here")]);
    assert!(output.status.success());
    assert!(target.path().join("foo").join("here").exists());
}

#[test]
fn test_whence() {
    let target = TempDir::new().unwrap();
    let path = target.path().join("foo").join("tmp.XXXXXX.dir");
    let path_os = path.as_os_str();
    let output = withd([os("-tc"), path_os, os("printenv"), os("WHENCE")]);
    assert!(output.status.success());
    // Convert stdout to a string from bytes. We assume that the path is valid
    // UTF-8, but of course this is not guaranteed, so this test _may_ fail
    // here. However, it's really unlikely.
    let stdout = String::from_utf8(output.stdout).expect("Path is not valid UTF-8");
    assert_eq!(
        std::env::current_dir().expect("Could not get current directory"),
        std::path::PathBuf::from(stdout.trim())
    );
}

// -----------------------------------------------------------------------------

static WITHD_EXE: &str = env!("CARGO_BIN_EXE_withd");

fn withd<I, S>(args: I) -> std::process::Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    match std::process::Command::new(WITHD_EXE).args(args).output() {
        Err(error) => panic!("Failed to execute {WITHD_EXE}: {error}"),
        Ok(output) => output,
    }
}

#[inline]
fn os(s: &str) -> &std::ffi::OsStr {
    std::ffi::OsStr::new(s)
}
