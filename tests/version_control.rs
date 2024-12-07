use anyhow::Context;
use rstest::{fixture, rstest};
use std::path::PathBuf;
use strum::IntoEnumIterator;
use utils::{
    create_test_folder,
    ResultExpected::{self, IsErr, IsOk},
    TestDir as TD,
};
use zola_chrono::{run, Cli};

mod utils;

#[test]
fn non_existent_folder() {
    let non_existent_path = "non_existent_path_bfEHgMV62y5S7LYn";
    assert!(!PathBuf::from(non_existent_path).exists());
    let cli = Cli {
        root_path: non_existent_path.to_string(),
        ..Default::default()
    };

    // Ensure run fails if folder doesn't exist
    let actual = run(&cli);
    assert!(actual.is_err());
}

#[fixture]
#[once]
fn create_dirs() -> anyhow::Result<()> {
    for test_dir in TD::iter() {
        create_test_folder(&test_dir)
            .with_context(|| format!("Failed to create directory {test_dir:?}"))?;
    }
    Ok(())
}

#[rstest]
// false, false
#[case(false, false, TD::NoVCS, IsErr)]
#[case(false, false, TD::Clean, IsOk)]
#[case(false, false, TD::StagedOnly, IsOk)]
#[case(false, false, TD::DirtyOnly, IsErr)]
#[case(false, false, TD::StagedAndDirty, IsErr)]
// false, true
#[case(false, true, TD::NoVCS, IsErr)]
#[case(false, true, TD::Clean, IsOk)]
#[case(false, true, TD::StagedOnly, IsOk)]
#[case(false, true, TD::DirtyOnly, IsOk)]
#[case(false, true, TD::StagedAndDirty, IsOk)]
// true, false
#[case(true, false, TD::NoVCS, IsErr)]
#[case(true, false, TD::Clean, IsOk)]
#[case(true, false, TD::StagedOnly, IsOk)]
#[case(true, false, TD::DirtyOnly, IsOk)]
#[case(true, false, TD::StagedAndDirty, IsOk)]
// true, true
#[case(true, true, TD::NoVCS, IsErr)]
#[case(true, true, TD::Clean, IsOk)]
#[case(true, true, TD::StagedOnly, IsOk)]
#[case(true, true, TD::DirtyOnly, IsOk)]
#[case(true, true, TD::StagedAndDirty, IsOk)]
fn test_with_unattended(
    #[case] should_check_only: bool,
    #[case] allow_dirty: bool,
    #[case] test_dir: utils::TestDir,
    #[case] expected: ResultExpected,
    create_dirs: &anyhow::Result<()>,
) {
    assert!(create_dirs.is_ok(), "{create_dirs:?}");
    let cli = Cli {
        root_path: Default::default(),
        unattended: true,
        should_check_only,
        allow_dirty,
    };
    utils::test_run(cli, test_dir, expected);
}
