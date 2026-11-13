//! Unit tests for `deletion`.

use super::*;
use std::os::unix::fs::symlink;

/// A temporary tree; the `TempDir` is held so it lives as long as the paths.
struct Tree(#[allow(dead_code)] tempfile::TempDir, PathBuf);

impl Tree {
    fn new() -> Self {
        let dir = tempfile::TempDir::new().unwrap();
        let root = std::fs::canonicalize(dir.path()).unwrap();
        Self(dir, root)
    }

    fn p(&self, rel: &str) -> PathBuf {
        self.1.join(rel)
    }

    fn dir(&self, rel: &str) -> PathBuf {
        let p = self.p(rel);
        std::fs::create_dir_all(&p).unwrap();
        p
    }
}

fn survivor(path: PathBuf, what: &str, gone_with_source_of: Option<usize>) -> Survivor {
    Survivor {
        path,
        what: what.to_string(),
        gone_with_source_of,
    }
}

fn deletion(path: PathBuf, owner: usize, kind: DeletionKind) -> Deletion {
    Deletion { path, owner, kind }
}

/// A path reaches a root through a symlink inside it even when it ends up
/// elsewhere, and a path that does not exist yet is walked as far as it
/// does. What fails this: comparing where paths end up instead of what
/// they pass through.
#[test]
fn path_reaches_counts_the_links_on_the_way() {
    let t = Tree::new();
    let old = t.dir("envs/old");
    let store = t.dir("store/scuv");
    symlink(&store, old.join("bridge")).unwrap();
    symlink(old.join("bridge"), t.p("scuv-link")).unwrap();
    assert!(path_reaches(&t.p("scuv-link/virtualenvs/new"), &old).unwrap());
    assert!(!path_reaches(&store.join("virtualenvs/new"), &old).unwrap());
    assert!(!path_reaches(&t.p("envs/sibling"), &old).unwrap());
    assert!(!path_reaches(&old, &t.p("gone")).unwrap());
}

/// A plain `--delete-source` and a plain `--force` pass; a target reached
/// through a link inside the deleted source, and a source reached through
/// the replaced target, block. What fails this: dropping the walk, or
/// exempting everything an env owns.
#[test]
fn check_deletions_blocks_only_what_something_depends_on() {
    let t = Tree::new();
    let src = t.dir("envs/old");
    let dst = t.dir("scuv/virtualenvs/dst");
    let ok = check_deletions(
        &[
            deletion(src.clone(), 0, DeletionKind::Source),
            deletion(dst.clone(), 0, DeletionKind::Target),
        ],
        &[
            survivor(src.clone(), "old's source", Some(0)),
            survivor(t.p("scuv/virtualenvs"), "the env directory", None),
        ],
    );
    assert!(ok.is_empty(), "{ok:?}");

    symlink(t.dir("store"), src.join("bridge")).unwrap();
    symlink(src.join("bridge"), t.p("home-link")).unwrap();
    let blocked = check_deletions(
        &[deletion(src.clone(), 0, DeletionKind::Source)],
        &[survivor(
            t.p("home-link/virtualenvs"),
            "the env directory",
            None,
        )],
    );
    assert_eq!(
        blocked,
        [Blocked {
            deletion: 0,
            because: "the env directory".into()
        }]
    );

    // The source is exempt from its own source deletion, not from a target
    // replacement that would remove it.
    let inside = t.dir("scuv/virtualenvs/dst/old-src");
    let blocked = check_deletions(
        &[deletion(dst.clone(), 0, DeletionKind::Target)],
        &[survivor(inside, "old's source", Some(0))],
    );
    assert_eq!(blocked.len(), 1);
}

/// A deletion of something that does not exist blocks nothing. What fails
/// this: walking survivors against a missing root.
#[test]
fn check_deletions_ignores_a_missing_directory() {
    let t = Tree::new();
    let blocked = check_deletions(
        &[deletion(t.p("gone"), 0, DeletionKind::Target)],
        &[survivor(t.dir("a"), "a", None)],
    );
    assert!(blocked.is_empty());
}

/// Same directory and nesting both ways overlap, through a symlink alias
/// too; siblings do not. What fails this: comparing path strings.
#[test]
fn overlaps_is_identity_and_nesting() {
    let t = Tree::new();
    let a = t.dir("x/a");
    let inner = t.dir("x/a/inner");
    let sibling = t.dir("x/b");
    symlink(&a, t.p("alias")).unwrap();
    assert!(overlaps(&a, &a).unwrap());
    assert!(overlaps(&a, &inner).unwrap());
    assert!(overlaps(&inner, &a).unwrap());
    assert!(overlaps(&t.p("alias"), &a).unwrap());
    assert!(!overlaps(&a, &sibling).unwrap());
}

/// A target that does not exist yet would be inside the source when its
/// nearest existing ancestor is the source or under it; a sibling under a
/// shared parent would not. What fails this: comparing the parent with
/// overlap in both directions (the parent contains the source).
#[test]
fn would_be_within_uses_the_nearest_existing_ancestor() {
    let t = Tree::new();
    let parent = t.dir("virtualenvs");
    let old = t.dir("virtualenvs/old");
    assert!(!would_be_within(&parent.join("new"), &old).unwrap());
    assert!(would_be_within(&old.join("home/virtualenvs/new"), &old).unwrap());
}

/// An env's interpreter is its `bin/python` and the `home` its pyvenv.cfg
/// records. What fails this: dropping the home.
#[test]
fn interpreter_paths_reads_pyvenv_home() {
    let t = Tree::new();
    let env = t.dir("env");
    std::fs::write(
        env.join("pyvenv.cfg"),
        "include-system-site-packages = false\nhome = /opt/py/bin\n",
    )
    .unwrap();
    assert_eq!(
        interpreter_paths(&env),
        [env.join("bin/python"), PathBuf::from("/opt/py/bin")]
    );
    std::fs::write(env.join("pyvenv.cfg"), "home = relative\n").unwrap();
    assert_eq!(interpreter_paths(&env), [env.join("bin/python")]);
}
