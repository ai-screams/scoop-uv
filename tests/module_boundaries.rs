//! Which top-level modules may not use which. Each rule names a dependency
//! that would close a cycle between modules; `python_version` may use none.
//!
//! The check parses the sources with `syn`, so strings and comments cannot
//! hide or fake a path. It starts at each module's root file and follows its
//! `mod` declarations, inline ones included. Items under `#[cfg(test)]`
//! (a test module and the files below it, a function, an impl method) are
//! skipped, since test code may reach anywhere. Every path is resolved to the
//! top-level module it names: grouped (`crate::{cli::Cli}`), relative
//! (`super::super::cli`) and re-exported (`crate::Result`, which `lib.rs`
//! takes from `error`) paths count the same as `crate::cli`. Paths inside
//! macro calls are read from the token tree, string literals left out.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use proc_macro2::{TokenStream, TokenTree};
use syn::visit::Visit;

/// The target of a path that reaches the crate root itself; forbidden in
/// every guarded module.
const CRATE_ROOT: &str = "<crate root>";

/// Forbidden: these top-level modules, or every module but its own.
enum Forbidden {
    Modules(&'static [&'static str]),
    AllOthers,
}

/// (root module, what it must not use, why)
const RULES: &[(&str, Forbidden, &str)] = &[
    (
        "shell",
        Forbidden::Modules(&["cli"]),
        "cli uses shell, so shell owns ShellType",
    ),
    ("paths", Forbidden::Modules(&["core"]), "core uses paths"),
    (
        "uv",
        Forbidden::Modules(&["core", "cli"]),
        "core uses uv, and uv uses PythonVersion",
    ),
    ("core", Forbidden::Modules(&["cli"]), "cli uses core"),
    (
        "python_version",
        Forbidden::AllOthers,
        "uv, core and cli all use it, so it uses no crate module",
    ),
];

/// `#[cfg(test)]` or `#[cfg(all(test, ...))]`.
fn is_test_only(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("cfg")
            && a.meta.require_list().is_ok_and(|l| {
                let t = l.tokens.to_string();
                t == "test" || t.starts_with("all (test")
            })
    })
}

/// The file of `mod name;` declared in `dir`, the directory a file's child
/// modules live in (inline modules already joined).
fn child_file(dir: &Path, name: &str) -> PathBuf {
    let flat = dir.join(format!("{name}.rs"));
    if flat.exists() {
        flat
    } else {
        dir.join(name).join("mod.rs")
    }
}

/// The directory `file`'s `mod x;` children live in.
fn children_dir(file: &Path) -> PathBuf {
    let stem = file.file_stem().unwrap().to_string_lossy();
    if stem == "mod" || stem == "lib" {
        file.parent().unwrap().to_path_buf()
    } else {
        file.parent().unwrap().join(&*stem)
    }
}

/// The crate root: its top-level modules and the names it re-exports,
/// each mapped to the module it comes from.
struct Root {
    modules: Vec<String>,
    reexports: HashMap<String, String>,
}

impl Root {
    fn parse(text: &str) -> Self {
        let ast = syn::parse_file(text).unwrap();
        let mut modules = Vec::new();
        let mut reexports = HashMap::new();
        for item in &ast.items {
            match item {
                syn::Item::Mod(m) => modules.push(m.ident.to_string()),
                syn::Item::Use(u) => {
                    fn walk(
                        t: &syn::UseTree,
                        from: Option<&str>,
                        out: &mut HashMap<String, String>,
                    ) {
                        match t {
                            syn::UseTree::Path(p) => {
                                let head = p.ident.to_string();
                                // `crate::cli::X` and `self::cli::X` come from `cli`.
                                if from.is_none() && (head == "crate" || head == "self") {
                                    return walk(&p.tree, None, out);
                                }
                                let from = from.map(str::to_string).unwrap_or(head);
                                walk(&p.tree, Some(&from), out);
                            }
                            syn::UseTree::Group(g) => {
                                g.items.iter().for_each(|i| walk(i, from, out));
                            }
                            syn::UseTree::Name(n) => {
                                if let Some(f) = from {
                                    out.insert(n.ident.to_string(), f.to_string());
                                }
                            }
                            syn::UseTree::Rename(r) => {
                                if let Some(f) = from {
                                    out.insert(r.rename.to_string(), f.to_string());
                                }
                            }
                            syn::UseTree::Glob(_) => {}
                        }
                    }
                    walk(&u.tree, None, &mut reexports);
                }
                _ => {}
            }
        }
        Self { modules, reexports }
    }

    /// The top-level module `name`, written right after `crate::`, stands for.
    fn module_of(&self, name: &str) -> String {
        self.reexports
            .get(name)
            .cloned()
            .unwrap_or_else(|| name.to_string())
    }
}

/// Collects the top-level module every path in one file refers to, and the
/// `mod x;` files it declares.
struct Uses<'a> {
    root: &'a Root,
    /// The module the code sits in, from the crate root (`["shell", "common"]`).
    module: Vec<String>,
    /// Where `mod x;` children of the current (possibly inline) module live.
    dir: PathBuf,
    file: &'a Path,
    found: &'a mut Vec<(String, String)>,
    pending: &'a mut Vec<(Vec<String>, PathBuf)>,
}

impl Uses<'_> {
    /// Records the top-level module `segments` (a path as written) names.
    /// A path that stops at the crate root (`use crate as r;`, `use
    /// crate::*;`, `use super::*;` from a top-level module) is recorded as
    /// [`CRATE_ROOT`]: what it reaches cannot be told from here.
    fn record(&mut self, segments: &[String]) {
        let target = match segments.first().map(String::as_str) {
            Some("crate") => Some(
                segments
                    .get(1)
                    .map_or_else(|| CRATE_ROOT.to_string(), |s| self.root.module_of(s)),
            ),
            Some("super" | "self") => {
                let mut at = self.module.clone();
                let mut rest = segments.iter().peekable();
                while let Some(s) = rest.next_if(|s| *s == "super" || *s == "self") {
                    if s == "super" {
                        at.pop();
                    }
                }
                match at.first() {
                    Some(top) => Some(top.clone()),
                    None => Some(
                        rest.next()
                            .map_or_else(|| CRATE_ROOT.to_string(), |s| self.root.module_of(s)),
                    ),
                }
            }
            _ => None,
        };
        if let Some(t) = target {
            let at = format!("{}: {}", self.file.display(), segments.join("::"));
            self.found.push((t, at));
        }
    }

    fn use_tree(&mut self, prefix: &mut Vec<String>, tree: &syn::UseTree) {
        match tree {
            syn::UseTree::Path(p) => {
                prefix.push(p.ident.to_string());
                self.use_tree(prefix, &p.tree);
                prefix.pop();
            }
            syn::UseTree::Group(g) => g.items.iter().for_each(|t| self.use_tree(prefix, t)),
            syn::UseTree::Name(n) => {
                prefix.push(n.ident.to_string());
                self.record(&prefix.clone());
                prefix.pop();
            }
            syn::UseTree::Rename(r) => {
                prefix.push(r.ident.to_string());
                self.record(&prefix.clone());
                prefix.pop();
            }
            syn::UseTree::Glob(_) => self.record(&prefix.clone()),
        }
    }

    /// `crate :: a` and `super :: ... :: a` runs in a macro body, at any
    /// group depth. Literals are single tokens, so strings never match.
    fn tokens(&mut self, stream: TokenStream) {
        let tokens: Vec<TokenTree> = stream.into_iter().collect();
        let ident = |i: usize| match tokens.get(i) {
            Some(TokenTree::Ident(id)) => Some(id.to_string()),
            _ => None,
        };
        let path_sep = |i: usize| {
            matches!((tokens.get(i), tokens.get(i + 1)),
                (Some(TokenTree::Punct(a)), Some(TokenTree::Punct(b)))
                    if a.as_char() == ':' && b.as_char() == ':')
        };
        let mut i = 0;
        while i < tokens.len() {
            if let TokenTree::Group(g) = &tokens[i] {
                self.tokens(g.stream());
            }
            let starts = matches!(ident(i).as_deref(), Some("crate" | "super"))
                && (i < 2 || !path_sep(i - 2));
            if starts {
                let mut path = vec![ident(i).unwrap()];
                let mut j = i;
                while path_sep(j + 1) {
                    match ident(j + 3) {
                        Some(seg) => path.push(seg),
                        None => break,
                    }
                    j += 3;
                    if path.last().is_some_and(|s| s != "super") {
                        break;
                    }
                }
                self.record(&path);
            }
            i += 1;
        }
    }
}

/// The attributes of an item, impl item or trait item, if it has any.
fn item_attrs(item: &syn::Item) -> Option<&[syn::Attribute]> {
    use syn::Item::*;
    Some(match item {
        Const(i) => &i.attrs,
        Enum(i) => &i.attrs,
        ExternCrate(i) => &i.attrs,
        Fn(i) => &i.attrs,
        ForeignMod(i) => &i.attrs,
        Impl(i) => &i.attrs,
        Macro(i) => &i.attrs,
        Mod(i) => &i.attrs,
        Static(i) => &i.attrs,
        Struct(i) => &i.attrs,
        Trait(i) => &i.attrs,
        TraitAlias(i) => &i.attrs,
        Type(i) => &i.attrs,
        Union(i) => &i.attrs,
        Use(i) => &i.attrs,
        _ => return None,
    })
}

impl<'ast> Visit<'ast> for Uses<'_> {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        if !item_attrs(item).is_some_and(is_test_only) {
            syn::visit::visit_item(self, item);
        }
    }

    fn visit_impl_item(&mut self, item: &'ast syn::ImplItem) {
        let attrs = match item {
            syn::ImplItem::Const(i) => &i.attrs,
            syn::ImplItem::Fn(i) => &i.attrs,
            syn::ImplItem::Type(i) => &i.attrs,
            syn::ImplItem::Macro(i) => &i.attrs,
            _ => return syn::visit::visit_impl_item(self, item),
        };
        if !is_test_only(attrs) {
            syn::visit::visit_impl_item(self, item);
        }
    }

    fn visit_trait_item(&mut self, item: &'ast syn::TraitItem) {
        let attrs = match item {
            syn::TraitItem::Const(i) => &i.attrs,
            syn::TraitItem::Fn(i) => &i.attrs,
            syn::TraitItem::Type(i) => &i.attrs,
            syn::TraitItem::Macro(i) => &i.attrs,
            _ => return syn::visit::visit_trait_item(self, item),
        };
        if !is_test_only(attrs) {
            syn::visit::visit_trait_item(self, item);
        }
    }

    fn visit_item_mod(&mut self, m: &'ast syn::ItemMod) {
        let name = m.ident.to_string();
        let mut module = self.module.clone();
        module.push(name.clone());
        if m.content.is_none() {
            self.pending.push((module, child_file(&self.dir, &name)));
            return;
        }
        let saved = (
            std::mem::replace(&mut self.module, module),
            self.dir.clone(),
        );
        self.dir = self.dir.join(&name);
        syn::visit::visit_item_mod(self, m);
        (self.module, self.dir) = saved;
    }

    /// `pub(crate)` and `pub(in crate::x)` limit who sees an item; they do
    /// not make the module depend on anything.
    fn visit_visibility(&mut self, _: &'ast syn::Visibility) {}

    fn visit_item_use(&mut self, u: &'ast syn::ItemUse) {
        self.use_tree(&mut Vec::new(), &u.tree);
    }

    fn visit_path(&mut self, p: &'ast syn::Path) {
        let segments: Vec<String> = p.segments.iter().map(|s| s.ident.to_string()).collect();
        self.record(&segments);
        syn::visit::visit_path(self, p);
    }

    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        self.tokens(m.tokens.clone());
        syn::visit::visit_macro(self, m);
    }
}

/// Every production file of the module rooted at `file`, with the paths it
/// uses resolved to top-level modules: (target, "file: path"). Returns how
/// many files it read.
fn scan(root: &Root, file: &Path, module: Vec<String>, found: &mut Vec<(String, String)>) -> usize {
    let mut pending = vec![(module, file.to_path_buf())];
    let mut files = 0;
    while let Some((module, file)) = pending.pop() {
        let text =
            std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
        let ast = syn::parse_file(&text).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
        files += 1;
        Uses {
            root,
            module,
            dir: children_dir(&file),
            file: &file,
            found,
            pending: &mut pending,
        }
        .visit_file(&ast);
    }
    files
}

/// Whether module `name` using top-level module `target` breaks its rule.
/// A path to the crate root itself breaks every rule.
fn violates(forbidden: &Forbidden, name: &str, target: &str) -> bool {
    target == CRATE_ROOT
        || match forbidden {
            Forbidden::Modules(list) => list.contains(&target),
            Forbidden::AllOthers => target != name,
        }
}

/// No module reaches a top-level module its rule forbids. What fails this:
/// any path from `src/shell` to `cli`, from `paths` or `uv` to `core`, from
/// `core` to `cli`, or from `python_version` to another crate module,
/// written as `crate::x`, `crate::{x::..}`, `super::x`, a re-export such as
/// `crate::Result`, or inside a macro.
#[test]
fn modules_keep_their_dependency_direction() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let root = Root::parse(&std::fs::read_to_string(src.join("lib.rs")).unwrap());
    let mut broken = Vec::new();
    for (name, forbidden, why) in RULES {
        assert!(root.modules.iter().any(|m| m == name), "no module {name}");
        let mut found = Vec::new();
        scan(
            &root,
            &child_file(&src, name),
            vec![name.to_string()],
            &mut found,
        );
        for (target, at) in found {
            if violates(forbidden, name, &target) {
                broken.push(format!("{at} -> {target} ({why})"));
            }
        }
    }
    assert!(broken.is_empty(), "{}", broken.join("\n"));
}

/// The paths one source text uses, resolved as in a file of `module`.
fn targets(text: &str, module: &[&str]) -> Vec<String> {
    let root = Root::parse(
        "pub mod error; pub mod cli; pub use error::{Result, ScoopError as E}; \
         pub use crate::cli::Cli as RootCli; pub use self::cli::Other;",
    );
    let ast = syn::parse_file(text).unwrap();
    let (mut found, mut pending) = (Vec::new(), Vec::new());
    Uses {
        root: &root,
        module: module.iter().map(|s| s.to_string()).collect(),
        dir: PathBuf::from("d"),
        file: Path::new("t.rs"),
        found: &mut found,
        pending: &mut pending,
    }
    .visit_file(&ast);
    let mut t: Vec<String> = found.into_iter().map(|(t, _)| t).collect();
    t.sort();
    t.dedup();
    t
}

/// The resolver: strings and comments name nothing, test items (also on
/// one line, and test methods) are skipped without swallowing the next
/// item, grouped, relative, re-exported and macro paths resolve to their
/// module, and paths that stop at the crate root are flagged. What fails this: a text scan, a skip that drops the item
/// after a test one, or a macro scan that splits rendered text.
#[test]
fn paths_resolve_to_their_top_level_module() {
    let text = r#"
        const S: &str = "crate::strings { // not code";
        /* crate::comments */
        #[cfg(test)] mod tests {}
        use crate::{grouped::{A, B}};
        use super::relative::C;
        #[cfg(test)]
        fn t() { crate::testonly::f(); }
        impl X { #[cfg(test)] fn m(_: crate::testmethod::T) {} }
        fn p() -> crate::Result<()> {
            crate::expr::f();
            t!(crate::inmacro::K, "crate::macrostring::K");
            format!("{:?}", Some(crate::nested::K));
            Ok(())
        }
    "#;
    assert_eq!(
        targets(text, &["shell"]),
        ["error", "expr", "grouped", "inmacro", "nested", "relative"]
    );
    // `super` from a nested module stays inside its top-level module.
    assert_eq!(
        targets("use super::sibling::X;", &["shell", "common"]),
        ["shell"]
    );
    assert_eq!(
        targets("use super::super::cli::X;", &["shell", "common"]),
        ["cli"]
    );
    assert_eq!(
        targets("fn f() { m!(super::super::cli::X) }", &["shell", "common"]),
        ["cli"]
    );
    // Re-exports written with `crate::`/`self::` resolve to their module.
    assert_eq!(
        targets("fn f(_: crate::RootCli, _: crate::Other) {}", &["shell"]),
        ["cli"]
    );
    // Paths that stop at the crate root cannot be resolved and are flagged;
    // `pub(crate)` is visibility, not a dependency.
    for text in ["use super::*;", "use crate as root;", "use crate::*;"] {
        assert_eq!(targets(text, &["python_version"]), [CRATE_ROOT], "{text}");
    }
    assert!(targets("pub(crate) fn f() {}", &["uv"]).is_empty());
    // `super::*` below the top level stays inside the module.
    assert_eq!(targets("use super::*;", &["shell", "common"]), ["shell"]);
}

/// `mod x;` inside an inline module is a file under that module's
/// directory. What fails this: following only file-level declarations.
#[test]
fn nested_external_modules_are_followed() {
    let ast = syn::parse_file("mod a { mod b; } #[cfg(test)] mod c;").unwrap();
    let root = Root::parse("");
    let (mut found, mut pending) = (Vec::new(), Vec::new());
    Uses {
        root: &root,
        module: vec!["shell".into()],
        dir: PathBuf::from("src/shell"),
        file: Path::new("src/shell/mod.rs"),
        found: &mut found,
        pending: &mut pending,
    }
    .visit_file(&ast);
    assert_eq!(pending.len(), 1, "{pending:?}");
    assert_eq!(pending[0].0, ["shell", "a", "b"]);
    assert_eq!(pending[0].1, PathBuf::from("src/shell/a/b/mod.rs"));
}

/// The rule decision. What fails this: letting a crate-root path through a
/// module list, or `AllOthers` rejecting the module's own paths.
#[test]
fn violates_flags_the_crate_root_under_every_rule() {
    let cli_only = Forbidden::Modules(&["cli"]);
    assert!(violates(&cli_only, "shell", "cli"));
    assert!(!violates(&cli_only, "shell", "error"));
    assert!(violates(&cli_only, "shell", CRATE_ROOT));
    assert!(violates(&Forbidden::AllOthers, "python_version", "error"));
    assert!(!violates(
        &Forbidden::AllOthers,
        "python_version",
        "python_version"
    ));
    assert!(violates(
        &Forbidden::AllOthers,
        "python_version",
        CRATE_ROOT
    ));
}
