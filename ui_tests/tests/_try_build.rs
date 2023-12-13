use std::{fs, os::unix::ffi::OsStrExt, path};

#[test]
fn ui() {
    let t = trybuild::TestCases::new();

    let entries_in_tests_folder =
        fs::read_dir("./tests").expect("should be able to read the “tests” folder");

    let mut pass_cases: Vec<path::PathBuf> = Vec::new();
    for entry in entries_in_tests_folder {
        let entry = entry.expect("should be able to iterate entries in the “tests” folder");
        if !entry
            .file_type()
            .expect("should be able to get the file type")
            .is_file()
        {
            continue;
        }
        if entry.file_name().as_bytes().first() == Some(b'_').as_ref() {
            continue;
        }
        pass_cases.push(entry.path());
    }

    for case in pass_cases {
        t.pass(case);
    }

    t.compile_fail("./should-fail/*.rs");
}
