use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn test_list_command() {
    let mut cmd = cargo_bin_cmd!("oxd");
    cmd.arg("-d").arg("design/docs").arg("list");

    cmd.assert().success().stdout(predicate::str::contains("Design Documents"));
}

#[test]
fn test_show_nonexistent() {
    let mut cmd = cargo_bin_cmd!("oxd");
    cmd.arg("-d").arg("design/docs").arg("show").arg("9999");

    cmd.assert().failure().stderr(predicate::str::contains("not found"));
}

#[test]
fn test_validate_command() {
    let mut cmd = cargo_bin_cmd!("oxd");
    cmd.arg("-d").arg("design/docs").arg("validate");

    cmd.assert().success();
}

#[test]
fn test_help() {
    let mut cmd = cargo_bin_cmd!("oxd");
    cmd.arg("--help");

    cmd.assert().success().stdout(predicate::str::contains("Oxur Design Documentation Manager"));
}
