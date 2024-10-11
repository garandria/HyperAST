use std::{iter::Peekable, path::Components};

use git2::{Oid, Repository};
use hyper_ast::hashed::{IndexingHashBuilder, MetaDataHashsBuilder};
use hyper_ast_gen_ts_java::legion_with_refs::{self, add_md_ref_ana};
use hyper_ast_gen_ts_java::{legion_with_refs::add_md_precomp_queries, types::Type};

use crate::{
    git::BasicGitObject,
    java::JavaAcc,
    preprocessed::{IsSkippedAna, RepositoryProcessor},
    processing::{erased::CommitProcExt, CacheHolding, InFiles, ObjectName},
    Processor, SimpleStores,
};

pub(crate) fn prepare_dir_exploration(tree: git2::Tree) -> Vec<BasicGitObject> {
    tree.iter()
        .rev()
        .map(TryInto::try_into)
        .filter_map(|x| x.ok())
        .collect()
}

pub struct JavaProcessor<'repo, 'prepro, 'd, 'c, Acc> {
    repository: &'repo Repository,
    prepro: &'prepro mut RepositoryProcessor,
    stack: Vec<(Oid, Vec<BasicGitObject>, Acc)>,
    pub dir_path: &'d mut Peekable<Components<'c>>,
    handle: &'d crate::processing::erased::ParametrizedCommitProcessor2Handle<JavaProc>,
}

impl<'repo, 'b, 'd, 'c, Acc: From<String>> JavaProcessor<'repo, 'b, 'd, 'c, Acc> {
    pub(crate) fn new(
        repository: &'repo Repository,
        prepro: &'b mut RepositoryProcessor,
        dir_path: &'d mut Peekable<Components<'c>>,
        name: &ObjectName,
        oid: git2::Oid,
        handle: &'d crate::processing::erased::ParametrizedCommitProcessor2Handle<JavaProc>,
    ) -> Self {
        let tree = repository.find_tree(oid).unwrap();
        let prepared = prepare_dir_exploration(tree);
        let name = name.try_into().unwrap();
        let stack = vec![(oid, prepared, Acc::from(name))];
        Self {
            stack,
            repository,
            prepro,
            dir_path,
            handle,
        }
    }
}

impl<'repo, 'b, 'd, 'c> Processor<JavaAcc> for JavaProcessor<'repo, 'b, 'd, 'c, JavaAcc> {
    fn pre(&mut self, current_object: BasicGitObject) {
        match current_object {
            BasicGitObject::Tree(oid, name) => {
                if let Some(
                    // (already, skiped_ana)
                    already,
                ) = self
                    .prepro
                    .processing_systems
                    .mut_or_default::<JavaProcessorHolder>()
                    .get_caches_mut() //.with_parameters(self.parameters.0)
                    .object_map
                    .get(&(oid, name.clone()))
                {
                    // reinit already computed node for post order
                    let full_node = already.clone();
                    // let skiped_ana = *skiped_ana;
                    let w = &mut self.stack.last_mut().unwrap().2;
                    let name = self.prepro.intern_object_name(&name);
                    assert!(!w.primary.children_names.contains(&name));
                    hyper_ast::tree_gen::Accumulator::push(w, (name, full_node));
                    // w.push(name, full_node, skiped_ana);
                    return;
                }
                log::info!("tree {:?}", name.try_str());
                let tree = self.repository.find_tree(oid).unwrap();
                let prepared: Vec<BasicGitObject> = prepare_dir_exploration(tree);
                self.stack
                    .push((oid, prepared, JavaAcc::new(name.try_into().unwrap())));
            }
            BasicGitObject::Blob(oid, name) => {
                if crate::processing::file_sys::Java::matches(&name) {
                    self.prepro
                        .help_handle_java_file(
                            oid,
                            &mut self.stack.last_mut().unwrap().2,
                            &name,
                            self.repository,
                            *self.handle,
                        )
                        .unwrap();
                } else {
                    log::debug!("not java source file {:?}", name.try_str());
                }
            }
        }
    }

    fn post(&mut self, oid: Oid, acc: JavaAcc) -> Option<(legion_with_refs::Local, IsSkippedAna)> {
        let skiped_ana = acc.skiped_ana;
        let name = &acc.primary.name;
        let key = (oid, name.as_bytes().into());
        let name = self.prepro.get_or_insert_label(name);
        let full_node = make(acc, self.prepro.main_stores_mut());
        self.prepro
            .processing_systems
            .mut_or_default::<JavaProcessorHolder>()
            .get_caches_mut()
            .object_map
            .insert(key, (full_node.clone(), skiped_ana));
        if self.stack.is_empty() {
            Some((full_node, skiped_ana))
        } else {
            let w = &mut self.stack.last_mut().unwrap().2;
            assert!(
                !w.primary.children_names.contains(&name),
                "{:?} {:?}",
                w.primary.children_names,
                name
            );
            w.push(name, full_node.clone(), skiped_ana);
            None
        }
    }

    fn stack(&mut self) -> &mut Vec<(Oid, Vec<BasicGitObject>, JavaAcc)> {
        &mut self.stack
    }
}

fn make(acc: JavaAcc, stores: &mut SimpleStores) -> hyper_ast_gen_ts_java::legion_with_refs::Local {
    use hyper_ast::{
        cyclomatic::Mcc,
        store::nodes::legion::{eq_node, NodeStore},
        types::LabelStore,
    };
    let node_store = &mut stores.node_store;
    let label_store = &mut stores.label_store;
    let interned_kind = Type::Directory;
    let label_id = label_store.get_or_insert(acc.primary.name.clone());

    let primary = acc
        .primary
        .map_metrics(|m| m.finalize(&interned_kind, &label_id, 0));

    let hashable = primary.metrics.hashs.most_discriminating();

    let eq = eq_node(&interned_kind, Some(&label_id), &primary.children);

    let insertion = node_store.prepare_insertion(&hashable, eq);

    let compute_ana = || {
        let ana = acc.ana;
        let ana = if acc.skiped_ana {
            log::info!(
                "show ana with at least {} refs",
                ana.lower_estimate_refs_count()
            );
            None
        } else {
            log::info!(
                "ref count lower estimate in dir {}",
                ana.lower_estimate_refs_count()
            );
            log::debug!("refs in directory");
            for x in ana.display_refs(label_store) {
                log::debug!("    {}", x);
            }
            log::debug!("decls in directory");
            for x in ana.display_decls(label_store) {
                log::debug!("    {}", x);
            }
            let c = ana.estimated_refs_count();
            if c < crate::MAX_REFS {
                Some(ana.resolve())
            } else {
                Some(ana)
            }
        };
        // log::info!(
        //     "ref count in dir after resolver {}",
        //     ana.lower_estimate_refs_count()
        // );
        // log::debug!("refs in directory after resolve: ");
        // for x in ana.display_refs(label_store) {
        //     log::debug!("    {}", x);
        // }
        ana
    };

    // Guard to avoid computing metadata for an already present subtree
    if let Some(id) = insertion.occupied_id() {
        // TODO add (debug) assertions to detect non-local metadata
        // TODO use the cache ?
        // this branch should be really cold
        let ana = compute_ana();
        let metrics = primary
            .metrics
            .map_hashs(|h| MetaDataHashsBuilder::build(h));
        return legion_with_refs::Local {
            compressed_node: id,
            metrics,
            ana,
            mcc: Mcc::new(&Type::Directory),
            role: None,
            precomp_queries: Default::default(),
        };
    }

    let ana = compute_ana();

    let mut dyn_builder = hyper_ast::store::nodes::legion::dyn_builder::EntityBuilder::new();

    add_md_precomp_queries(&mut dyn_builder, acc.precomp_queries);
    let children_is_empty = primary.children.is_empty();
    if acc.skiped_ana {
        use hyper_ast::store::nodes::EntityBuilder;
        dyn_builder.add(hyper_ast::filter::BloomSize::None);
    } else {
        add_md_ref_ana(&mut dyn_builder, children_is_empty, ana.as_ref());
    }
    let metrics = primary.persist(&mut dyn_builder, interned_kind, label_id);
    let metrics = metrics.map_hashs(|h| h.build());
    let hashs = metrics.add_md_metrics(&mut dyn_builder, children_is_empty);
    hashs.persist(&mut dyn_builder);

    let vacant = insertion.vacant();
    let node_id = NodeStore::insert_built_after_prepare(vacant, dyn_builder.build());

    let full_node = legion_with_refs::Local {
        compressed_node: node_id.clone(),
        metrics,
        ana,
        mcc: Mcc::new(&interned_kind),
        role: None,
        precomp_queries: acc.precomp_queries,
    };
    full_node
}

use hyper_ast_gen_ts_java::legion_with_refs as java_tree_gen;

#[derive(Clone, PartialEq, Eq)]
pub struct Parameter {
    pub(crate) query: Option<std::sync::Arc<[String]>>,
}

#[derive(Default)]
pub(crate) struct JavaProcessorHolder(Option<JavaProc>);
pub(crate) struct JavaProc {
    parameter: Parameter,
    query: Query,
    cache: crate::processing::caches::Java,
    commits: std::collections::HashMap<git2::Oid, crate::Commit>,
}
impl crate::processing::erased::Parametrized for JavaProcessorHolder {
    type T = Parameter;
    fn register_param(
        &mut self,
        t: Self::T,
    ) -> crate::processing::erased::ParametrizedCommitProcessorHandle {
        let l = self
            .0
            .iter()
            .position(|x| &x.parameter == &t)
            .unwrap_or_else(|| {
                let l = 0; //self.0.len();
                           // self.0.push(JavaProc(t));
                let query = if let Some(q) = &t.query {
                    Query::new(q.iter().map(|x| x.as_str()))
                } else {
                    Query::default()
                };
                self.0 = Some(JavaProc {
                    parameter: t,
                    query,
                    cache: Default::default(),
                    commits: Default::default(),
                });
                l
            });
        use crate::processing::erased::{
            ConfigParametersHandle, ParametrizedCommitProc, ParametrizedCommitProcessorHandle,
        };
        ParametrizedCommitProcessorHandle(self.erased_handle(), ConfigParametersHandle(l))
    }
}

#[derive(Clone)]
pub(crate) struct Query(
    pub(crate) std::sync::Arc<hyper_ast_tsquery::Query>,
    std::sync::Arc<String>,
);

unsafe impl Send for Query {}
unsafe impl Sync for Query {}
impl PartialEq for Query {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}
impl Eq for Query {}

impl Default for Query {
    fn default() -> Self {
        let precomputeds = unsafe { crate::java_processor::SUB_QUERIES };
        Query::new(precomputeds.into_iter().map(|x| x.as_ref()))
    }
}

impl Query {
    fn new<'a>(precomputeds: impl Iterator<Item = &'a str>) -> Self {
        let precomputeds = precomputeds.collect::<Vec<_>>();
        let (precomp, _) = hyper_ast_tsquery::Query::with_precomputed(
            "(_)",
            hyper_ast_gen_ts_java::language(),
            precomputeds.as_slice(),
        )
        .unwrap();
        Self(precomp.into(), precomputeds.join("\n").into())
    }
}

impl crate::processing::erased::CommitProc for JavaProc {
    fn get_commit(&self, commit_oid: git2::Oid) -> Option<&crate::Commit> {
        self.commits.get(&commit_oid)
    }

    fn prepare_processing<'repo>(
        &self,
        _repository: &'repo git2::Repository,
        _commit_builder: crate::preprocessed::CommitBuilder,
    ) -> Box<dyn crate::processing::erased::PreparedCommitProc + 'repo> {
        unimplemented!("required for processing java at the root of project")
    }
}

impl crate::processing::erased::CommitProcExt for JavaProc {
    type Holder = JavaProcessorHolder;
}
impl crate::processing::erased::ParametrizedCommitProc2 for JavaProcessorHolder {
    type Proc = JavaProc;

    fn with_parameters_mut(
        &mut self,
        parameters: crate::processing::erased::ConfigParametersHandle,
    ) -> &mut Self::Proc {
        assert_eq!(0, parameters.0);
        self.0.as_mut().unwrap()
    }

    fn with_parameters(
        &self,
        parameters: crate::processing::erased::ConfigParametersHandle,
    ) -> &Self::Proc {
        assert_eq!(0, parameters.0);
        self.0.as_ref().unwrap()
    }
}
impl CacheHolding<crate::processing::caches::Java> for JavaProc {
    fn get_caches_mut(&mut self) -> &mut crate::processing::caches::Java {
        &mut self.cache
    }
    fn get_caches(&self) -> &crate::processing::caches::Java {
        &self.cache
    }
}

impl CacheHolding<crate::processing::caches::Java> for JavaProcessorHolder {
    fn get_caches_mut(&mut self) -> &mut crate::processing::caches::Java {
        &mut self.0.as_mut().unwrap().cache
    }
    fn get_caches(&self) -> &crate::processing::caches::Java {
        &self.0.as_ref().unwrap().cache
    }
}

/// WARN be cautious about mutating that
/// TODO make something safer
#[doc(hidden)]
pub static mut SUB_QUERIES: &[&str] = &[
    r#"(method_invocation
    (identifier) (#EQ? "fail")
)"#,
    r#"(try_statement
    (block)
    (catch_clause)
)"#,
    r#"(marker_annotation 
    name: (identifier) (#EQ? "Test")
)"#,
    "(constructor_declaration)",
    "(class_declaration)",
    "(interface_declaration)",
    r#"(method_invocation
    name: (identifier) (#EQ? "sleep")
)"#,
    r#"(marker_annotation
    name: (identifier) (#EQ? "Ignored")
)"#,
    r#"(block
    "{"
    .
    "}"
)"#,
    r#"(method_invocation
    (identifier) (#EQ? "assertEquals")
)"#,
    r#"(method_invocation
    (identifier) (#EQ? "assertSame")
)"#,
    r#"(method_invocation
    (identifier) (#EQ? "assertThat")
)"#,
];

pub fn sub_queries() -> &'static [&'static str] {
    unsafe { SUB_QUERIES }
}

impl RepositoryProcessor {
    pub(crate) fn handle_java_file(
        &mut self,
        name: &ObjectName,
        text: &[u8],
    ) -> Result<java_tree_gen::FNode, ()> {
        todo!() // not used much anyway apart from  check_random_files_reserialization
                // crate::java::handle_java_file(&mut self.java_generator(text), name, text)
    }

    fn java_generator(
        &mut self,
        text: &[u8],
    ) -> java_tree_gen::JavaTreeGen<crate::TStore, hyper_ast_tsquery::Query> {
        let line_break = if text.contains(&b'\r') {
            "\r\n".as_bytes().to_vec()
        } else {
            "\n".as_bytes().to_vec()
        };

        let (precomp, _) = hyper_ast_tsquery::Query::with_precomputed(
            "(_)",
            hyper_ast_gen_ts_java::language(),
            sub_queries(),
        )
        .unwrap();
        java_tree_gen::JavaTreeGen {
            line_break,
            stores: &mut self.main_stores,
            md_cache: &mut self
                .processing_systems
                .mut_or_default::<JavaProcessorHolder>()
                .get_caches_mut()
                .md_cache, //java_md_cache,
            more: precomp,
        }
    }

    pub(crate) fn help_handle_java_folder<'a, 'b, 'c, 'd: 'c>(
        &'a mut self,
        repository: &'b Repository,
        dir_path: &'c mut Peekable<Components<'d>>,
        oid: Oid,
        name: &ObjectName,
    ) -> <JavaAcc as hyper_ast::tree_gen::Accumulator>::Node {
        let full_node = self.handle_java_directory(repository, dir_path, name, oid);
        let name = self.intern_object_name(name);
        (name, full_node)
    }

    fn handle_java_blob(
        &mut self,
        oid: Oid,
        name: &ObjectName,
        repository: &Repository,
        parameters: crate::processing::erased::ParametrizedCommitProcessor2Handle<JavaProc>,
    ) -> Result<(java_tree_gen::Local, IsSkippedAna), crate::ParseErr> {
        self.processing_systems
            .caching_blob_handler::<crate::processing::file_sys::Java>()
            .handle2(oid, repository, name, parameters, |c, n, t| {
                let line_break = if t.contains(&b'\r') {
                    "\r\n".as_bytes().to_vec()
                } else {
                    "\n".as_bytes().to_vec()
                };

                let holder = c.mut_or_default::<JavaProcessorHolder>();
                let precomp = holder.0.as_ref().unwrap().query.0.clone();
                let caches = holder.get_caches_mut();
                let mut java_tree_gen = java_tree_gen::JavaTreeGen {
                    line_break,
                    stores: self
                        .main_stores
                        .mut_with_ts::<hyper_ast_gen_ts_java::types::TStore>(),
                    md_cache: &mut caches.md_cache,
                    more: precomp,
                };

                crate::java::handle_java_file(&mut java_tree_gen, n, t)
                    .map_err(|_| crate::ParseErr::IllFormed)
                    .map(|x| (x.local.clone(), false))
            })
    }

    fn help_handle_java_file(
        &mut self,
        oid: Oid,
        w: &mut JavaAcc,
        name: &ObjectName,
        repository: &Repository,
        parameters: crate::processing::erased::ParametrizedCommitProcessor2Handle<JavaProc>,
    ) -> Result<(), crate::ParseErr> {
        let (full_node, skiped_ana) = self.handle_java_blob(oid, name, repository, parameters)?;
        let name = self.intern_object_name(name);
        assert!(!w.primary.children_names.contains(&name));
        w.push(name, full_node, skiped_ana);
        Ok(())
    }

    /// oid : Oid of a dir such that */src/main/java/ or */src/test/java/
    fn handle_java_directory<'b, 'd: 'b>(
        &mut self,
        repository: &Repository,
        dir_path: &'b mut Peekable<Components<'d>>,
        name: &ObjectName,
        oid: git2::Oid,
    ) -> (java_tree_gen::Local, IsSkippedAna) {
        let h = self
            .processing_systems
            .mut_or_default::<JavaProcessorHolder>();
        let handle = JavaProc::register_param(h, Parameter { query: None });
        JavaProcessor::<JavaAcc>::new(repository, self, dir_path, name, oid, &handle).process()
    }
}

// TODO try to separate processing from caching from git
#[cfg(test)]
#[allow(unused)]
mod experiments {
    use super::*;
    use crate::{
        git::{NamedObject, ObjectType, TypedObject, UniqueObject},
        processing::InFiles,
        Accumulator,
    };

    pub(crate) struct GitProcessorMiddleWare<'repo, 'prepro, 'd, 'c> {
        repository: &'repo Repository,
        prepro: &'prepro mut RepositoryProcessor,
        dir_path: &'d mut Peekable<Components<'c>>,
    }

    impl<'repo, 'b, 'd, 'c> GitProcessorMiddleWare<'repo, 'b, 'd, 'c> {
        pub(crate) fn prepare_dir_exploration<It>(&self, current_object: It::Item) -> Vec<It::Item>
        where
            It: Iterator,
            It::Item: NamedObject + UniqueObject<Id = Oid>,
        {
            let tree = self.repository.find_tree(*current_object.id()).unwrap();
            tree.iter()
                .rev()
                .map(|_| todo!())
                // .filter_map(|x| x.ok())
                .collect()
        }
    }

    impl<'repo, 'b, 'd, 'c> JavaProcessor<'repo, 'b, 'd, 'c, JavaAcc> {
        pub(crate) fn prepare_dir_exploration<T>(&self, current_object: &T) -> Vec<T>
        where
            T: NamedObject + UniqueObject<Id = git2::Oid>,
        {
            let tree = self.repository.find_tree(*current_object.id()).unwrap();
            todo!()
        }
        pub(crate) fn stack(
            &mut self,
            current_object: BasicGitObject,
            prepared: Vec<BasicGitObject>,
            acc: JavaAcc,
        ) {
            let tree = self.repository.find_tree(*current_object.id()).unwrap();
            self.stack.push((*current_object.id(), prepared, acc));
        }
        fn pre(
            &mut self,
            current_object: BasicGitObject,
            already: Option<<JavaAcc as Accumulator>::Unlabeled>,
        ) -> Option<<JavaAcc as Accumulator>::Unlabeled> {
            match current_object.r#type() {
                ObjectType::Dir => {
                    if let Some(already) = already {
                        let full_node = already.clone();
                        return Some(full_node);
                    }
                    log::info!("tree {:?}", current_object.name().try_str());
                    let prepared: Vec<BasicGitObject> =
                        self.prepare_dir_exploration(&current_object);
                    let acc = JavaAcc::new(current_object.name().try_into().unwrap());
                    self.stack(current_object, prepared, acc);
                    None
                }
                ObjectType::File => {
                    if crate::processing::file_sys::Java::matches(current_object.name()) {
                        self.prepro
                            .help_handle_java_file(
                                *current_object.id(),
                                &mut self.stack.last_mut().unwrap().2,
                                current_object.name(),
                                self.repository,
                                *self.handle,
                            )
                            .unwrap();
                    } else {
                        log::debug!("not java source file {:?}", current_object.name().try_str());
                    }
                    None
                }
            }
        }
        fn post(
            &mut self,
            oid: Oid,
            acc: JavaAcc,
        ) -> Option<(legion_with_refs::Local, IsSkippedAna)> {
            let skiped_ana = acc.skiped_ana;
            let name = &acc.primary.name;
            let key = (oid, name.as_bytes().into());
            let name = self.prepro.intern_label(name);
            let full_node = make(acc, self.prepro.main_stores_mut());
            let full_node = (full_node, skiped_ana);
            self.prepro
                .processing_systems
                .mut_or_default::<JavaProcessorHolder>()
                .get_caches_mut()
                .object_map
                .insert(key, full_node.clone());
            if self.stack.is_empty() {
                Some(full_node)
            } else {
                let w = &mut self.stack.last_mut().unwrap().2;
                assert!(
                    !w.primary.children_names.contains(&name),
                    "{:?} {:?}",
                    w.primary.children_names,
                    name
                );
                hyper_ast::tree_gen::Accumulator::push(w, (name, full_node));
                None
            }
        }
    }
}
