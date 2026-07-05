//! Spec integrity suite.
//!
//! Guarantees the self-description layer can never drift from the binary:
//! 1. every CLI leaf command has a spec file, and every spec file has a CLI leaf
//! 2. every spec example actually runs and produces the documented output
//! 3. every upstream reference in a spec exists in the parity map, and the
//!    parity map marks it implemented

use agent_it_tools::meta;
use std::collections::BTreeSet;
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn every_tool_has_a_spec_and_vice_versa() {
    let cli: BTreeSet<String> = meta::leaf_paths().into_iter().collect();
    let specs: BTreeSet<String> = meta::load_specs().unwrap().into_keys().collect();

    let missing_spec: Vec<_> = cli.difference(&specs).collect();
    let orphan_spec: Vec<_> = specs.difference(&cli).collect();
    assert!(
        missing_spec.is_empty() && orphan_spec.is_empty(),
        "spec drift!\n  CLI leaves without a spec file: {missing_spec:?}\n  spec files without a CLI leaf: {orphan_spec:?}"
    );
}

#[test]
fn spec_examples_run_and_match() {
    let specs = meta::load_specs().unwrap();
    let bin = env!("CARGO_BIN_EXE_agent-it-tools");

    for (path, spec) in &specs {
        assert!(
            !spec.examples.is_empty(),
            "spec '{path}' has no examples - every tool must ship at least one verified example"
        );
        for (i, ex) in spec.examples.iter().enumerate() {
            let label = format!("{path} example #{i}");
            let expected_prefix: Vec<&str> = path.split(' ').collect();
            assert!(
                ex.argv.len() >= 2 && ex.argv[..2] == expected_prefix[..],
                "{label}: argv {:?} does not start with the tool path",
                ex.argv
            );
            assert!(
                ex.stdout.is_some() ^ ex.stdout_regex.is_some(),
                "{label}: exactly one of stdout / stdout_regex must be set"
            );

            let mut cmd = Command::new(bin);
            cmd.args(&ex.argv)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            let mut child = cmd.spawn().unwrap();
            if let Some(stdin_data) = &ex.stdin {
                child
                    .stdin
                    .as_mut()
                    .unwrap()
                    .write_all(stdin_data.as_bytes())
                    .unwrap();
            }
            drop(child.stdin.take());
            let out = child.wait_with_output().unwrap();
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stdout = stdout.trim_end_matches('\n');
            assert!(
                out.status.success(),
                "{label}: exited {:?}, stderr: {}",
                out.status.code(),
                String::from_utf8_lossy(&out.stderr)
            );

            if let Some(expected) = &ex.stdout {
                assert_eq!(
                    stdout,
                    expected.trim_end_matches('\n'),
                    "{label}: stdout mismatch"
                );
            }
            if let Some(pattern) = &ex.stdout_regex {
                let re = regex::Regex::new(pattern)
                    .unwrap_or_else(|e| panic!("{label}: bad stdout_regex: {e}"));
                assert!(
                    re.is_match(stdout),
                    "{label}: stdout does not match /{pattern}/\nstdout was: {stdout}"
                );
            }
        }
    }
}

#[test]
fn parity_map_is_consistent_with_specs() {
    let specs = meta::load_specs().unwrap();
    let parity = meta::load_parity().unwrap();

    let mut by_id = std::collections::BTreeMap::new();
    for e in &parity.upstream {
        assert!(
            ["implemented", "planned", "not-applicable"].contains(&e.status.as_str()),
            "parity entry '{}' has invalid status '{}'",
            e.id,
            e.status
        );
        assert!(
            by_id.insert(e.id.clone(), e).is_none(),
            "duplicate parity entry '{}'",
            e.id
        );
    }

    // Every upstream id referenced by a spec must exist and be implemented.
    for (path, spec) in &specs {
        for id in &spec.it_tools {
            let entry = by_id
                .get(id)
                .unwrap_or_else(|| panic!("spec '{path}' references unknown upstream id '{id}'"));
            assert_eq!(
                entry.status, "implemented",
                "spec '{path}' references '{id}' but parity status is '{}'",
                entry.status
            );
        }
    }

    // Every implemented parity entry must point at real tool paths.
    let cli: BTreeSet<String> = meta::leaf_paths().into_iter().collect();
    for e in parity.upstream.iter().filter(|e| e.status == "implemented") {
        let tool = e
            .tool
            .as_deref()
            .unwrap_or_else(|| panic!("implemented parity entry '{}' has no tool field", e.id));
        for t in tool.split(", ") {
            assert!(
                cli.contains(t),
                "parity entry '{}' points at unknown tool '{t}'",
                e.id
            );
        }
    }
}
