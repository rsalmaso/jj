// Copyright 2022 The Jujutsu Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::common::TestEnvironment;

#[test]
fn test_interdiff_basic() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    std::fs::write(repo_path.join("file1"), "foo\n").unwrap();
    test_env.run_jj_in(&repo_path, ["new"]).success();
    std::fs::write(repo_path.join("file2"), "foo\n").unwrap();
    test_env
        .run_jj_in(&repo_path, ["bookmark", "create", "-r@", "left"])
        .success();

    test_env.run_jj_in(&repo_path, ["new", "root()"]).success();
    std::fs::write(repo_path.join("file3"), "foo\n").unwrap();
    test_env.run_jj_in(&repo_path, ["new"]).success();
    std::fs::write(repo_path.join("file2"), "foo\nbar\n").unwrap();
    test_env
        .run_jj_in(&repo_path, ["bookmark", "create", "-r@", "right"])
        .success();

    // implicit --to
    let output = test_env.run_jj_in(&repo_path, ["interdiff", "--from", "left"]);
    insta::assert_snapshot!(output, @r"
    Modified regular file file2:
       1    1: foo
            2: bar
    [EOF]
    ");

    // explicit --to
    test_env.run_jj_in(&repo_path, ["new", "@-"]).success();
    let output = test_env.run_jj_in(&repo_path, ["interdiff", "--from", "left", "--to", "right"]);
    insta::assert_snapshot!(output, @r"
    Modified regular file file2:
       1    1: foo
            2: bar
    [EOF]
    ");
    test_env.run_jj_in(&repo_path, ["undo"]).success();

    // formats specifiers
    let output = test_env.run_jj_in(
        &repo_path,
        ["interdiff", "--from", "left", "--to", "right", "-s"],
    );
    insta::assert_snapshot!(output, @r"
    M file2
    [EOF]
    ");

    let output = test_env.run_jj_in(
        &repo_path,
        ["interdiff", "--from", "left", "--to", "right", "--git"],
    );
    insta::assert_snapshot!(output, @r"
    diff --git a/file2 b/file2
    index 257cc5642c..3bd1f0e297 100644
    --- a/file2
    +++ b/file2
    @@ -1,1 +1,2 @@
     foo
    +bar
    [EOF]
    ");
}

#[test]
fn test_interdiff_paths() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    std::fs::write(repo_path.join("file1"), "foo\n").unwrap();
    std::fs::write(repo_path.join("file2"), "foo\n").unwrap();
    test_env.run_jj_in(&repo_path, ["new"]).success();
    std::fs::write(repo_path.join("file1"), "bar\n").unwrap();
    std::fs::write(repo_path.join("file2"), "bar\n").unwrap();
    test_env
        .run_jj_in(&repo_path, ["bookmark", "create", "-r@", "left"])
        .success();

    test_env.run_jj_in(&repo_path, ["new", "root()"]).success();
    std::fs::write(repo_path.join("file1"), "foo\n").unwrap();
    std::fs::write(repo_path.join("file2"), "foo\n").unwrap();
    test_env.run_jj_in(&repo_path, ["new"]).success();
    std::fs::write(repo_path.join("file1"), "baz\n").unwrap();
    std::fs::write(repo_path.join("file2"), "baz\n").unwrap();
    test_env
        .run_jj_in(&repo_path, ["bookmark", "create", "-r@", "right"])
        .success();

    let output = test_env.run_jj_in(
        &repo_path,
        ["interdiff", "--from", "left", "--to", "right", "file1"],
    );
    insta::assert_snapshot!(output, @r"
    Modified regular file file1:
       1    1: barbaz
    [EOF]
    ");

    let output = test_env.run_jj_in(
        &repo_path,
        [
            "interdiff",
            "--from",
            "left",
            "--to",
            "right",
            "file1",
            "file2",
        ],
    );
    insta::assert_snapshot!(output, @r"
    Modified regular file file1:
       1    1: barbaz
    Modified regular file file2:
       1    1: barbaz
    [EOF]
    ");
}

#[test]
fn test_interdiff_conflicting() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    std::fs::write(repo_path.join("file"), "foo\n").unwrap();
    test_env.run_jj_in(&repo_path, ["new"]).success();
    std::fs::write(repo_path.join("file"), "bar\n").unwrap();
    test_env
        .run_jj_in(&repo_path, ["bookmark", "create", "-r@", "left"])
        .success();

    test_env.run_jj_in(&repo_path, ["new", "root()"]).success();
    std::fs::write(repo_path.join("file"), "abc\n").unwrap();
    test_env.run_jj_in(&repo_path, ["new"]).success();
    std::fs::write(repo_path.join("file"), "def\n").unwrap();
    test_env
        .run_jj_in(&repo_path, ["bookmark", "create", "-r@", "right"])
        .success();

    let output = test_env.run_jj_in(
        &repo_path,
        ["interdiff", "--from", "left", "--to", "right", "--git"],
    );
    insta::assert_snapshot!(output, @r"
    diff --git a/file b/file
    index 0000000000..24c5735c3e 100644
    --- a/file
    +++ b/file
    @@ -1,7 +1,1 @@
    -<<<<<<< Conflict 1 of 1
    -%%%%%%% Changes from base to side #1
    --foo
    -+abc
    -+++++++ Contents of side #2
    -bar
    ->>>>>>> Conflict 1 of 1 ends
    +def
    [EOF]
    ");
}
