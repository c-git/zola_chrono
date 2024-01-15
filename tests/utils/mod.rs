use std::env;
use std::fmt::Debug;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;
use strum::EnumIter;
use zola_chrono::run;
use zola_chrono::Cli;

mod git_commands;

pub enum ResultExpected {
    IsOk,
    IsErr,
}

pub(crate) fn test_run(mut cli: Cli, test_dir: TestDir, expected: ResultExpected) {
    let path = test_dir.to_canonicalized_path();
    cli.root_path = path.to_string_lossy().to_string();
    println!("Cli: {cli:#?}\ntest_dir: {test_dir:?}\nPath: {path:?}");
    let actual = run(&cli);
    println!("run result: {actual:?}");
    match expected {
        ResultExpected::IsOk => assert!(actual.is_ok()),
        ResultExpected::IsErr => assert!(actual.is_err()),
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
    pub(crate) const TEST_DIR_BASE: &'static str = "tests/test_folders/";
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
        // Need to capture and lock in value of original directory because `run` changes the current working directory if it runs successfully and following tests fail
        static ORG_WORKING_DIR: OnceLock<PathBuf> = OnceLock::new();
        let org_dir =
            ORG_WORKING_DIR.get_or_init(|| env::current_dir().expect("Failed to get current_dir"));
        let result = org_dir.join(self.to_path());
        assert!(result.exists(), "Path not found: {result:?}");
        result.canonicalize().unwrap()
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
            cargo_util::paths::create_dir_all(&path)?;
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
