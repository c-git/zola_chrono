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
    let mut cli = Cli::default();
    let non_existent_path = "non_existent_path_bfEHgMV62y5S7LYn";
    assert!(!PathBuf::from(non_existent_path).exists());
    cli.root_path = non_existent_path.to_string();

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
#[case(TD::NoVCS)]
#[case(TD::Clean)]
#[case(TD::StagedOnly)]
#[case(TD::DirtyOnly)]
#[case(TD::StagedAndDirty)]
fn allow_no_vcs(#[case] test_dir: utils::TestDir, create_dirs: &anyhow::Result<()>) {
    assert!(create_dirs.is_ok(), "{create_dirs:?}");
    let expected = IsOk;
    utils::test_run(
        Cli {
            unattended: true,
            ..Default::default()
        },
        test_dir,
        expected,
    );
}

#[rstest]
//
#[case(false, false, false, TD::NoVCS, IsErr)]
#[case(false, false, false, TD::Clean, IsErr)]
#[case(false, false, false, TD::StagedOnly, IsErr)]
#[case(false, false, false, TD::DirtyOnly, IsErr)]
#[case(false, false, false, TD::StagedAndDirty, IsErr)]
fn test_with_unattended(
    #[case] should_check_only: bool,
    #[case] allow_dirty: bool,
    #[case] allow_no_vcs: bool,
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
        allow_no_vcs,
        log_level: Default::default(),
    };
    utils::test_run(cli, test_dir, expected);
}
