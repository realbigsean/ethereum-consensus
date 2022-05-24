use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};
use syn::visit::Visit;
use syn::visit_mut::VisitMut;
use syn::{Item, ItemFn, ItemUse};

const SPEC_IMPORT: &str = "use crate::phase0 as spec;";
const GENERATION_WARNING: &str =
    "// WARNING: This file was derived by the `gen-spec` utility. DO NOT EDIT MANUALLY.\n\n";

enum Pass {
    RemoveOverrides,
    ImportOverrides,
    Finalize,
}

struct Editor {
    overrides: HashSet<String>,
    pass: Pass,
    target_fork_module: String,
    source_module: String,
}

impl<'ast> Visit<'ast> for Editor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let fn_name = &node.sig.ident;
        self.overrides.insert(fn_name.to_string());
    }
}

impl VisitMut for Editor {
    fn visit_file_mut(&mut self, node: &mut syn::File) {
        match self.pass {
            Pass::RemoveOverrides => {
                let indices_to_remove = node
                    .items
                    .iter()
                    .enumerate()
                    .filter_map(|(i, item)| match item {
                        Item::Fn(node) => {
                            let fn_name = node.sig.ident.to_string();
                            if self.overrides.contains(&fn_name) {
                                Some(i)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                for index in indices_to_remove {
                    node.items.remove(index);
                }
            }
            Pass::ImportOverrides => {
                let mut iter = node
                    .items
                    .iter()
                    .enumerate()
                    .skip_while(|(_, item)| !matches!(item, Item::Use(..)))
                    .skip_while(|(_, item)| matches!(item, Item::Use(..)));
                let (insertion_index, _) = iter.next().unwrap();
                for (offset, ident) in self.overrides.iter().enumerate() {
                    let src_mod = &self.source_module;
                    let target_fork_mod = &self.target_fork_module;
                    let use_item: ItemUse = syn::parse_str(&format!(
                        "use {src_mod}_{target_fork_mod}::{ident} as {ident};"
                    ))
                    .unwrap();
                    node.items
                        .insert(insertion_index + offset, Item::Use(use_item.into()));
                }
            }
            Pass::Finalize => {
                let target: ItemUse = syn::parse_str(&SPEC_IMPORT).unwrap();
                let target_index = node
                    .items
                    .iter()
                    .enumerate()
                    .filter_map(|(i, item)| {
                        if item == &Item::Use(target.clone()) {
                            Some(i)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                if target_index.len() > 1 {
                    panic!("more than one 'spec import' found, please fix source");
                }
                let target_index = target_index[0];
                let target_fork_mod = &self.target_fork_module;
                let replacement: ItemUse =
                    syn::parse_str(&format!("use crate::{target_fork_mod} as spec;")).unwrap();
                node.items[target_index] = replacement.into();
            }
        }
    }
}

fn merge(
    target_fork_module: &str,
    source_module: &str,
    base_src: &str,
    overrides_src: Option<String>,
) -> syn::File {
    let mut merger = Editor {
        overrides: Default::default(),
        pass: Pass::RemoveOverrides,
        target_fork_module: target_fork_module.to_string(),
        source_module: source_module.to_string(),
    };

    if let Some(overrides_src) = overrides_src {
        let overrides = syn::parse_str::<syn::File>(&overrides_src).unwrap();
        merger.visit_file(&overrides);
    }

    // Remove overrides from base
    let mut base = syn::parse_str::<syn::File>(base_src).unwrap();
    merger.visit_file_mut(&mut base);

    // Import overrides from supplemental module
    merger.pass = Pass::ImportOverrides;
    merger.visit_file_mut(&mut base);

    // Finalize any remaining edits...
    merger.pass = Pass::Finalize;
    merger.visit_file_mut(&mut base);

    base
}

fn render(
    target_fork_module: &str,
    source_module: &str,
    src_path: &PathBuf,
    dest_path: PathBuf,
    modification_path: PathBuf,
) {
    let patch_src = match File::open(modification_path) {
        Ok(mut file) => {
            let mut patch_src = String::new();
            file.read_to_string(&mut patch_src).unwrap();
            Some(patch_src)
        }
        Err(err) => match err.kind() {
            ErrorKind::NotFound => None,
            _ => panic!("{}", err),
        },
    };

    let src_src = fs::read_to_string(src_path).unwrap();
    let target = merge(target_fork_module, source_module, &src_src, patch_src);
    let dest_src = prettyplease::unparse(&target);
    let mut output = String::from(GENERATION_WARNING);
    output.push_str(&dest_src);
    fs::write(dest_path, output).unwrap();
}

fn main() {
    let root = Path::new("./src");
    let source_dir = root.join("phase0");
    let source_modules_to_gen = vec![
        "helpers",
        "block_processing",
        "epoch_processing",
        "slot_processing",
        "state_transition",
        "genesis",
    ];

    let target_fork_modules = vec!["altair", "bellatrix"];

    for source_module in source_modules_to_gen {
        let source_path = source_dir.join(source_module).with_extension("rs");
        for target_fork_module in &target_fork_modules {
            let dest_path = root
                .join(target_fork_module)
                .join(source_module)
                .with_extension("rs");
            let modification_path = root
                .join(target_fork_module)
                .join(format!("{source_module}_{target_fork_module}"))
                .with_extension("rs");
            render(
                target_fork_module,
                source_module,
                &source_path,
                dest_path,
                modification_path,
            );
        }
    }
}
