use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};
use syn::visit::Visit;
use syn::visit_mut::VisitMut;
use syn::{
    AngleBracketedGenericArguments, ConstParam, GenericArgument, GenericParam, Ident, Item, ItemFn,
    ItemUse, PatType, PathArguments, PathSegment, Type, TypePath,
};

const ATTESTATION_BOUND_IDENT: &str = "PENDING_ATTESTATIONS_BOUND";
const SYNC_COMMITTEE_SIZE_IDENT: &str = "SYNC_COMMITTEE_SIZE";
const SPEC_IMPORT: &str = "use crate::phase0 as spec;";
const GENERATION_WARNING: &str =
    "// WARNING: This file was derived by the `gen-spec` utility. DO NOT EDIT MANUALLY.\n\n";

enum Pass {
    RemoveOverrides,
    FixGenerics,
    ImportOverrides,
    Finalize,
}

struct Editor {
    overrides: HashSet<String>,
    pattern_to_expire: String,
    pass: Pass,
    target_fork_module: String,
    source_module: String,
    extend_for_block: bool,
}

impl<'ast> Visit<'ast> for Editor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let fn_name = &node.sig.ident;
        self.overrides.insert(fn_name.to_string());
    }
}

impl VisitMut for Editor {
    fn visit_angle_bracketed_generic_arguments_mut(
        &mut self,
        node: &mut AngleBracketedGenericArguments,
    ) {
        match self.pass {
            Pass::FixGenerics => {
                if self.extend_for_block {
                    let arg: Type = syn::parse_str("SYNC_COMMITTEE_SIZE").unwrap();
                    node.args.push(GenericArgument::Type(arg));
                }
            }
            _ => {}
        }
        syn::visit_mut::visit_angle_bracketed_generic_arguments_mut(self, node);
    }

    // `sed` in `syn`
    fn visit_ident_mut(&mut self, node: &mut Ident) {
        match self.pass {
            Pass::FixGenerics => {
                let target_ident: Ident = syn::parse_str(&ATTESTATION_BOUND_IDENT).unwrap();
                if node == &target_ident {
                    let replacement: Ident = syn::parse_str(&SYNC_COMMITTEE_SIZE_IDENT).unwrap();
                    *node = replacement;
                }
            }
            _ => {}
        }
        syn::visit_mut::visit_ident_mut(self, node);
    }

    fn visit_path_segment_mut(&mut self, node: &mut PathSegment) {
        match self.pass {
            Pass::FixGenerics => {
                let ident = node.ident.to_string();
                if ident.contains("BeaconBlock") {
                    match &mut node.arguments {
                        PathArguments::AngleBracketed(args) => {
                            self.extend_for_block = true;
                            self.visit_angle_bracketed_generic_arguments_mut(args);
                            self.extend_for_block = false;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        syn::visit_mut::visit_path_segment_mut(self, node);
    }

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
                            let is_overriden = self.overrides.contains(&fn_name);
                            let is_expired = fn_name.starts_with(&self.pattern_to_expire);
                            let should_drop = is_overriden || is_expired;
                            if should_drop {
                                Some(i)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                let mut removed = 0;
                for index in indices_to_remove {
                    node.items.remove(index - removed);
                    removed += 1;
                }
            }
            Pass::FixGenerics => {}
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
                        "pub use crate::{target_fork_mod}::{src_mod}_{target_fork_mod}::{ident} as {ident};"
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
        syn::visit_mut::visit_file_mut(self, node);
    }
}

fn assemble(
    target_fork_module: &str,
    source_module: &str,
    base_src: &str,
    overrides_src: Option<String>,
) -> syn::File {
    let mut editor = Editor {
        overrides: Default::default(),
        pattern_to_expire: "get_matching_".to_string(),
        pass: Pass::RemoveOverrides,
        target_fork_module: target_fork_module.to_string(),
        source_module: source_module.to_string(),
        extend_for_block: false,
    };

    if let Some(overrides_src) = overrides_src {
        let overrides = syn::parse_str::<syn::File>(&overrides_src).unwrap();
        editor.visit_file(&overrides);
    }

    // Remove overrides from base
    let mut base = syn::parse_str::<syn::File>(base_src).unwrap();
    editor.visit_file_mut(&mut base);

    // Fix generics
    editor.pass = Pass::FixGenerics;
    editor.visit_file_mut(&mut base);

    // Import overrides from supplemental module
    editor.pass = Pass::ImportOverrides;
    editor.visit_file_mut(&mut base);

    // Finalize any remaining edits...
    editor.pass = Pass::Finalize;
    editor.visit_file_mut(&mut base);

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
    let target = assemble(target_fork_module, source_module, &src_src, patch_src);
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
