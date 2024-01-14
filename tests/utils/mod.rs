use anyhow::bail;
use cargo_util::paths;
use std::fmt::Debug;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use strum::EnumIter;
use version_control_clean_check::check_version_control;
use version_control_clean_check::CheckOptions;
use version_control_clean_check::VCSError;
use version_control_clean_check::VCSResult;

mod git_commands;

pub(crate) fn test_check_version_control(
    opts: CheckOptions,
    test_dir: TestDir,
    expected: VCSResult<()>,
) {
    let path = test_dir.to_canonicalized_path();
    println!("Opts: {opts:#?}\ntest_dir: {test_dir:?}\nPath: {path:?}");
    let actual = check_version_control(path, &opts);
    match_results(actual, expected);
}

#[derive(Debug)]
pub(crate) struct TestError(VCSError);

impl From<VCSError> for TestError {
    fn from(value: VCSError) -> Self {
        Self(value)
    }
}

impl<T: Debug> TryFrom<VCSResult<T>> for TestError {
    type Error = anyhow::Error;

    fn try_from(value: VCSResult<T>) -> Result<Self, Self::Error> {
        if value.is_ok() {
            bail!("Value is not error. Found: {value:?}");
        }
        Ok(Self(value.unwrap_err()))
    }
}

impl PartialEq for TestError {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (VCSError::NoVCS, VCSError::NoVCS) => true,
            (
                VCSError::NotAllowedFilesFound {
                    dirty_files: l_dirty_files,
                    staged_files: l_staged_files,
                },
                VCSError::NotAllowedFilesFound {
                    dirty_files: r_dirty_files,
                    staged_files: r_staged_files,
                },
            ) => l_dirty_files == r_dirty_files && l_staged_files == r_staged_files,
            (VCSError::GitError(..), _) | (VCSError::Anyhow(..), _) => false, // Never equal if not one of our local errors during testing
            _ => core::mem::discriminant(&self.0) == core::mem::discriminant(&other.0),
        }
    }
}

#[derive(EnumIter, Debug)]
pub enum TestDir {
    NoVCS,
    Clean,
    StagedOnly,
    DirtyOnly,
    StagedAndDirty,
}

impl TestDir {
    pub(crate) const TEST_DIR_BASE: &str = "tests/test_folders/";
    pub(crate) fn to_path(&self) -> PathBuf {
        let base_test_folder = PathBuf::from(Self::TEST_DIR_BASE);
        let sub_folder = match self {
            TestDir::NoVCS => "no_vcs",
            TestDir::Clean => "clean",
            TestDir::StagedOnly => "staged_only",
            TestDir::DirtyOnly => "dirty_only",
            TestDir::StagedAndDirty => "staged_and_dirty",
        };
        base_test_folder.join(sub_folder)
    }

    pub(crate) fn to_canonicalized_path(&self) -> PathBuf {
        let result = self.to_path();
        assert!(result.exists(), "Path not found: {result:?}");
        result.canonicalize().unwrap()
    }
}

pub(crate) fn match_results(actual: VCSResult<()>, expected: VCSResult<()>) {
    match (&actual, &expected) {
        (Ok(_), Ok(_)) => (),
        (Ok(_), Err(_)) | (Err(_), Ok(_)) => {
            panic!("Actual and Expected do not match.\nactual: {actual:?}\nexpected: {expected:?}")
        }
        (Err(..), Err(..)) => {
            let actual_error = actual.unwrap_err();
            let expected_error = expected.unwrap_err();
            println!("---\nActual Error:\n{actual_error}\n");
            println!("---\nExpected Error:\n{expected_error}\n---");
            assert_eq!(TestError(actual_error), TestError(expected_error))
        }
    }
}

pub fn create_test_folder(test_dir: &TestDir) -> anyhow::Result<()> {
    // Skip if folder if it already exists (doesn't check that it is in the correct state)
    let path = test_dir.to_path();
    if path.exists() {
        return Ok(());
    }

    match test_dir {
        TestDir::NoVCS => {
            paths::create_dir_all(&path)?;
            create_abc(&path)?;
        }
        TestDir::Clean => {
            let repo = git_commands::init(&path)?;
            create_abc(&path)?;
            git_commands::add_all(&repo, &["a", "b", "c"])?;
            git_commands::commit_irrelevant_msg(&repo)?;
        }
        TestDir::StagedOnly => {
            let repo = git_commands::init(&path)?;
            create_abc(&path)?;
            git_commands::add_all(&repo, &["a", "b", "c"])?;
            git_commands::commit_irrelevant_msg(&repo)?;
            modify_files(&path, &["b", "c"])?;
            git_commands::add_all(&repo, &["b", "c"])?;
        }
        TestDir::DirtyOnly => {
            let repo = git_commands::init(&path)?;
            create_abc(&path)?;
            git_commands::add_all(&repo, &["a"])?;
            git_commands::commit_irrelevant_msg(&repo)?;
        }
        TestDir::StagedAndDirty => {
            let repo = git_commands::init(&path)?;
            create_abc(&path)?;
            git_commands::add_all(&repo, &["a", "b", "c"])?;
            git_commands::commit_irrelevant_msg(&repo)?;
            modify_files(&path, &["b", "c"])?;
            git_commands::add_all(&repo, &["b"])?;
        }
    }
    Ok(())
}

fn create_abc<P: AsRef<Path>>(path: P) -> anyhow::Result<()> {
    let path = path.as_ref();
    for name in ["a", "b", "c"] {
        let file_name = path.join(name);
        File::create(file_name)?;
    }
    Ok(())
}

fn modify_files<P: AsRef<Path>>(path: P, files: &[&str]) -> anyhow::Result<()> {
    let path = path.as_ref();
    for name in files {
        let file_name = path.join(name);
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(file_name)?;
        file.write_all(b"Some text\n")?;
    }
    Ok(())
}
