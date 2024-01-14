use anyhow::Context;
use rstest::{fixture, rstest};
use std::path::PathBuf;
use strum::IntoEnumIterator;
use utils::{create_test_folder, TestDir as TD};
use version_control_clean_check::{check_version_control, CheckOptions, VCSError, VCSResult};

mod utils;

#[test]
fn non_existent_folder() {
    let mut opts = CheckOptions::new();
    let non_existent_path = PathBuf::from("non_existent_path_bfEHgMV62y5S7LYn");
    assert!(!non_existent_path.exists());

    // Test is no vcs
    let actual = check_version_control(&non_existent_path, &opts);
    utils::match_results(actual, Err(VCSError::NoVCS));

    // Test passes if no vcs allowed
    opts.allow_no_vcs = true;
    let actual = check_version_control(&non_existent_path, &opts);
    utils::match_results(actual, Ok(()));
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
    let mut opts = CheckOptions::new();
    opts.allow_no_vcs = true;
    let expected = Ok(());
    utils::test_check_version_control(opts, test_dir, expected);
}

#[rstest]
// No Dirty, No Staged
#[case(false, false, TD::NoVCS, Err(VCSError::NoVCS))]
#[case(false, false, TD::Clean, Ok(()))]
#[case(false, false, TD::StagedOnly, Err(VCSError::NotAllowedFilesFound { dirty_files: vec![], staged_files: vec!["b".to_string(), "c".to_string()] }))]
#[case(false, false, TD::DirtyOnly, Err(VCSError::NotAllowedFilesFound { dirty_files: vec!["b".to_string(), "c".to_string()], staged_files: vec![] }))]
#[case(false, false, TD::StagedAndDirty, Err(VCSError::NotAllowedFilesFound { dirty_files: vec!["c".to_string()], staged_files: vec!["b".to_string()] }))]
// No Dirty, Yes Staged
#[case(false, true, TD::NoVCS, Err(VCSError::NoVCS))]
#[case(false, true, TD::Clean, Ok(()))]
#[case(false, true, TD::StagedOnly, Ok(()))]
#[case(false, true, TD::DirtyOnly, Err(VCSError::NotAllowedFilesFound { dirty_files: vec!["b".to_string(), "c".to_string()], staged_files: vec![] }))]
#[case(false, true, TD::StagedAndDirty, Err(VCSError::NotAllowedFilesFound { dirty_files: vec!["c".to_string()], staged_files: vec![] }))]
// Yes Dirty, No Staged
#[case(true, false, TD::NoVCS, Err(VCSError::NoVCS))]
#[case(true, false, TD::Clean, Ok(()))]
#[case(true, false, TD::StagedOnly, Err(VCSError::NotAllowedFilesFound { dirty_files: vec![], staged_files: vec!["b".to_string(), "c".to_string()] }))]
#[case(true, false, TD::DirtyOnly, Ok(()))]
#[case(true, false, TD::StagedAndDirty, Err(VCSError::NotAllowedFilesFound { dirty_files: vec![], staged_files: vec!["b".to_string()] }))]
// Yes Dirty, Yes Staged
#[case(true, true, TD::NoVCS, Err(VCSError::NoVCS))]
#[case(true, true, TD::Clean, Ok(()))]
#[case(true, true, TD::StagedOnly, Ok(()))]
#[case(true, true, TD::DirtyOnly, Ok(()))]
#[case(true, true, TD::StagedAndDirty, Ok(()))]
fn vcs_required(
    #[case] allow_dirty: bool,
    #[case] allow_staged: bool,
    #[case] test_dir: utils::TestDir,
    #[case] expected: VCSResult<()>,
    create_dirs: &anyhow::Result<()>,
) {
    assert!(create_dirs.is_ok(), "{create_dirs:?}");
    let mut opts = CheckOptions::new();
    // opts.allow_no_vcs = false; // Always false because it is tested in allow_no_vcs
    opts.allow_dirty = allow_dirty;
    opts.allow_staged = allow_staged;

    utils::test_check_version_control(opts, test_dir, expected);
}
