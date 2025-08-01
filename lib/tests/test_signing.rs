use jj_lib::backend::MillisSinceEpoch;
use jj_lib::backend::Signature;
use jj_lib::backend::Timestamp;
use jj_lib::config::ConfigLayer;
use jj_lib::config::ConfigSource;
use jj_lib::repo::Repo as _;
use jj_lib::settings::UserSettings;
use jj_lib::signing::SigStatus;
use jj_lib::signing::SignBehavior;
use jj_lib::signing::Signer;
use jj_lib::signing::Verification;
use jj_lib::test_signing_backend::TestSigningBackend;
use test_case::test_case;
use testutils::TestRepoBackend;
use testutils::TestWorkspace;
use testutils::create_random_commit;
use testutils::write_random_commit;

fn user_settings(behavior: SignBehavior) -> UserSettings {
    let mut config = testutils::base_user_config();
    config.add_layer(
        ConfigLayer::parse(
            ConfigSource::User,
            &format!(
                r#"
                signing.key = "impeccable"
                signing.behavior = "{}"
                "#,
                match behavior {
                    SignBehavior::Drop => "drop",
                    SignBehavior::Keep => "keep",
                    SignBehavior::Own => "own",
                    SignBehavior::Force => "force",
                }
            ),
        )
        .unwrap(),
    );
    UserSettings::from_config(config).unwrap()
}

fn someone_else() -> Signature {
    Signature {
        name: "Someone Else".to_string(),
        email: "someone-else@example.com".to_string(),
        timestamp: Timestamp {
            timestamp: MillisSinceEpoch(0),
            tz_offset: 0,
        },
    }
}

fn good_verification() -> Option<Verification> {
    Some(Verification {
        status: SigStatus::Good,
        key: Some("impeccable".to_owned()),
        display: Some("test-display".into()),
    })
}

#[test_case(TestRepoBackend::Simple ; "simple backend")]
#[test_case(TestRepoBackend::Git ; "git backend")]
fn manual(backend: TestRepoBackend) {
    let settings = user_settings(SignBehavior::Own);

    let signer = Signer::new(Some(Box::new(TestSigningBackend)), vec![]);
    let test_workspace = TestWorkspace::init_with_backend_and_signer(backend, signer, &settings);

    let repo = &test_workspace.repo;

    let repo = repo.clone();
    let mut tx = repo.start_transaction();
    let commit1 = create_random_commit(tx.repo_mut())
        .set_sign_behavior(SignBehavior::Own)
        .write()
        .unwrap();
    let commit2 = create_random_commit(tx.repo_mut())
        .set_sign_behavior(SignBehavior::Own)
        .set_author(someone_else())
        .write()
        .unwrap();
    tx.commit("test").unwrap();

    let commit1 = repo.store().get_commit(commit1.id()).unwrap();
    assert_eq!(commit1.verification().unwrap(), good_verification());

    let commit2 = repo.store().get_commit(commit2.id()).unwrap();
    assert_eq!(commit2.verification().unwrap(), None);
}

#[test_case(TestRepoBackend::Git ; "git backend")]
fn keep_on_rewrite(backend: TestRepoBackend) {
    let settings = user_settings(SignBehavior::Own);

    let signer = Signer::new(Some(Box::new(TestSigningBackend)), vec![]);
    let test_workspace = TestWorkspace::init_with_backend_and_signer(backend, signer, &settings);

    let repo = &test_workspace.repo;

    let repo = repo.clone();
    let mut tx = repo.start_transaction();
    let commit = create_random_commit(tx.repo_mut())
        .set_sign_behavior(SignBehavior::Own)
        .write()
        .unwrap();
    tx.commit("test").unwrap();

    let mut tx = repo.start_transaction();
    let mut_repo = tx.repo_mut();
    let rewritten = mut_repo.rewrite_commit(&commit).write().unwrap();

    let commit = repo.store().get_commit(rewritten.id()).unwrap();
    assert_eq!(commit.verification().unwrap(), good_verification());
}

#[test_case(TestRepoBackend::Git ; "git backend")]
fn manual_drop_on_rewrite(backend: TestRepoBackend) {
    let settings = user_settings(SignBehavior::Own);

    let signer = Signer::new(Some(Box::new(TestSigningBackend)), vec![]);
    let test_workspace = TestWorkspace::init_with_backend_and_signer(backend, signer, &settings);

    let repo = &test_workspace.repo;

    let repo = repo.clone();
    let mut tx = repo.start_transaction();
    let commit = create_random_commit(tx.repo_mut())
        .set_sign_behavior(SignBehavior::Own)
        .write()
        .unwrap();
    tx.commit("test").unwrap();

    let mut tx = repo.start_transaction();
    let mut_repo = tx.repo_mut();
    let rewritten = mut_repo
        .rewrite_commit(&commit)
        .set_sign_behavior(SignBehavior::Drop)
        .write()
        .unwrap();

    let commit = repo.store().get_commit(rewritten.id()).unwrap();
    assert_eq!(commit.verification().unwrap(), None);
}

#[test_case(TestRepoBackend::Git ; "git backend")]
fn forced(backend: TestRepoBackend) {
    let settings = user_settings(SignBehavior::Force);

    let signer = Signer::new(Some(Box::new(TestSigningBackend)), vec![]);
    let test_workspace = TestWorkspace::init_with_backend_and_signer(backend, signer, &settings);

    let repo = &test_workspace.repo;

    let repo = repo.clone();
    let mut tx = repo.start_transaction();
    let commit = create_random_commit(tx.repo_mut())
        .set_author(someone_else())
        .write()
        .unwrap();
    tx.commit("test").unwrap();

    let commit = repo.store().get_commit(commit.id()).unwrap();
    assert_eq!(commit.verification().unwrap(), good_verification());
}

#[test_case(TestRepoBackend::Git ; "git backend")]
fn configured(backend: TestRepoBackend) {
    let settings = user_settings(SignBehavior::Own);

    let signer = Signer::new(Some(Box::new(TestSigningBackend)), vec![]);
    let test_workspace = TestWorkspace::init_with_backend_and_signer(backend, signer, &settings);

    let repo = &test_workspace.repo;

    let repo = repo.clone();
    let mut tx = repo.start_transaction();
    let commit = write_random_commit(tx.repo_mut());
    tx.commit("test").unwrap();

    let commit = repo.store().get_commit(commit.id()).unwrap();
    assert_eq!(commit.verification().unwrap(), good_verification());
}

#[test_case(TestRepoBackend::Git ; "git backend")]
fn drop_behavior(backend: TestRepoBackend) {
    let settings = user_settings(SignBehavior::Drop);

    let signer = Signer::new(Some(Box::new(TestSigningBackend)), vec![]);
    let test_workspace = TestWorkspace::init_with_backend_and_signer(backend, signer, &settings);

    let repo = &test_workspace.repo;

    let repo = repo.clone();
    let mut tx = repo.start_transaction();
    let commit = create_random_commit(tx.repo_mut())
        .set_sign_behavior(SignBehavior::Own)
        .write()
        .unwrap();
    tx.commit("test").unwrap();

    let original_commit = repo.store().get_commit(commit.id()).unwrap();
    assert_eq!(original_commit.verification().unwrap(), good_verification());

    let mut tx = repo.start_transaction();
    let mut_repo = tx.repo_mut();
    let rewritten = mut_repo.rewrite_commit(&original_commit).write().unwrap();

    let rewritten_commit = repo.store().get_commit(rewritten.id()).unwrap();
    assert_eq!(rewritten_commit.verification().unwrap(), None);
}
