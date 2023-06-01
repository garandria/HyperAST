use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

use git2::Repository;
use hyper_ast::store::nodes::DefaultNodeIdentifier as NodeIdentifier;

use crate::{
    git::{all_commits_between, Repo},
    maven::MavenModuleAcc,
    maven_processor::make,
    preprocessed::{CommitProcessor, RepositoryProcessor},
    processing::{file_sys, BuildSystem, ConfiguredRepo, ProcessingConfig, RepoConfig},
    Commit, DefaultMetrics, SimpleStores,
};

/// Preprocess git repositories
/// share most components with PreProcessedRepository

#[derive(Default)]
pub struct PreProcessedRepositories {
    pub commits: HashMap<RepoConfig, HashMap<git2::Oid, Commit>>,
    pub processor: RepositoryProcessor,
    // pub processing_ordered_commits: HashMap<String,Vec<git2::Oid>>,
}

pub struct RepositoryInfo {
    pub sys: BuildSystem,
    /// map repository names to some objects they contain (branches, references, commit).
    /// At least keeps roots
    pub commits: HashSet<git2::Oid>,
}

#[derive(Default)]
pub struct CommitsPerSys {
    pub maven: HashMap<git2::Oid, Commit>,
    pub make: HashMap<git2::Oid, Commit>,
    pub npm: HashMap<git2::Oid, Commit>,
    pub any: HashMap<git2::Oid, Commit>,
}

impl CommitsPerSys {
    // pub fn accessCommits<'a>(&'a self, sys: &BuildSystem) -> &'a HashMap<git2::Oid, Commit> {
    //     match sys {
    //         BuildSystem::Maven => &self.maven,
    //         BuildSystem::Make => &self.make,
    //         BuildSystem::Npm => &self.npm,
    //         BuildSystem::None => &self.any,
    //     }
    // }
    pub fn accessCommits<'a>(&'a self, sys: &RepoConfig) -> &'a HashMap<git2::Oid, Commit> {
        match sys {
            RepoConfig::JavaMaven => &self.maven,
            RepoConfig::CppMake => &self.make,
            RepoConfig::TsNpm => &self.npm,
            RepoConfig::Any => &self.any,
        }
    }
}

pub(crate) struct CommitBuilder<'prepro, 'repo, Sys, CP: CommitProcessor<Sys>> {
    pub commits: &'prepro mut HashMap<git2::Oid, Commit>,
    pub processor: &'prepro mut CP,
    repository: &'repo mut ConfiguredRepo,
    phantom: PhantomData<Sys>,
}
impl<'prepro, 'repo, Sys, CP: CommitProcessor<Sys>> CommitBuilder<'prepro, 'repo, Sys, CP> {
    pub fn with_limit(
        self,
        before: &str,
        after: &str,
        dir_path: &str,
        limit: usize,
    ) -> Result<Vec<git2::Oid>, git2::Error> {
        log::info!(
            "commits to process: {:?}",
            all_commits_between(&self.repository.repo, before, after).map(|x| x.count())
        );
        Ok(all_commits_between(&self.repository.repo, before, after)?
            .take(limit)
            .map(|oid| {
                let oid = oid.unwrap();
                let c = self
                    .processor
                    .handle_commit::<true>(&self.repository.repo, dir_path, oid);
                self.commits.insert(oid.clone(), c);
                oid.clone()
            })
            .collect())
    }

    pub fn single(
        &mut self,
        repository: &mut Repository,
        ref_or_commit: &str,
        dir_path: &str,
    ) -> git2::Oid {
        let oid = crate::git::retrieve_commit(repository, ref_or_commit)
            .unwrap()
            .id();
        let c = self
            .processor
            .handle_commit::<true>(&repository, dir_path, oid);
        self.commits.insert(oid.clone(), c);
        oid
    }
}

impl PreProcessedRepositories {
    // pub fn commit_builder<'prepro, 'repo, Sys, CP:CommitProcessor<Sys>>(
    //     &'prepro mut self,
    //     repository: &'repo mut ConfiguredRepo,
    // ) -> CommitBuilder<'prepro, 'repo, Sys, CP> {
    //     todo!()
    //     // CommitBuilder {
    //     //     commits: self.commits.get_mut(&repository.config).unwrap(),
    //     //     processor: &mut self.processor,
    //     //     repository: repository,
    //     //     phantom: PhantomData,
    //     // }
    // }

    pub fn purge_caches(&mut self) {
        self.processor.purge_caches()
    }

    pub fn pre_process_with_limit(
        &mut self,
        repository: &mut Repository,
        before: &str,
        after: &str,
        dir_path: &str,
        limit: usize,
    ) -> Result<Vec<git2::Oid>, git2::Error> {
        log::info!(
            "commits to process: {:?}",
            all_commits_between(&repository, before, after).map(|x| x.count())
        );
        let mut processing_ordered_commits = vec![];
        let rw = all_commits_between(&repository, before, after)?;
        let commits = self.commits.entry(RepoConfig::JavaMaven).or_default();
        rw
            // .skip(1500)release-1.0.0 refs/tags/release-3.3.2-RC4
            .take(limit) // TODO make a variable
            .for_each(|oid| {
                let oid = oid.unwrap();
                let c = CommitProcessor::<file_sys::Maven>::handle_commit::<true>(
                    &mut self.processor,
                    &repository,
                    dir_path,
                    oid,
                );
                processing_ordered_commits.push(oid.clone());
                commits.insert(oid.clone(), c);
            });
        Ok(processing_ordered_commits)
    }

    pub fn pre_process_with_config(
        &mut self,
        repository: &mut ConfiguredRepo,
        before: &str,
        after: &str,
    ) -> Result<Vec<git2::Oid>, git2::Error> {
        let config = &repository.config;
        let config = config.into();
        let repository = &mut repository.repo;
        log::info!(
            "commits to process: {:?}",
            all_commits_between(&repository, before, after).map(|x| x.count())
        );
        let mut processing_ordered_commits = vec![];
        let rw = all_commits_between(&repository, before, after)?;
        match config {
            ProcessingConfig::JavaMaven { limit, dir_path } => {
                let commits = self.commits.entry(RepoConfig::JavaMaven).or_default();
                rw
                    // .skip(1500)release-1.0.0 refs/tags/release-3.3.2-RC4
                    .take(limit) // TODO make a variable
                    .for_each(|oid| {
                        let oid = oid.unwrap();
                        let c = CommitProcessor::<file_sys::Maven>::handle_commit::<true>(
                            &mut self.processor,
                            &repository,
                            dir_path,
                            oid,
                        );
                        processing_ordered_commits.push(oid.clone());
                        commits.insert(oid.clone(), c);
                    });
            }
            ProcessingConfig::CppMake { limit, dir_path } => {
                let commits = self.commits.entry(RepoConfig::CppMake).or_default();
                rw
                    // .skip(1500)release-1.0.0 refs/tags/release-3.3.2-RC4
                    .take(limit) // TODO make a variable
                    .for_each(|oid| {
                        let oid = oid.unwrap();
                        let c = CommitProcessor::<file_sys::Make>::handle_commit::<true>(
                            &mut self.processor,
                            &repository,
                            dir_path,
                            oid,
                        );
                        processing_ordered_commits.push(oid.clone());
                        commits.insert(oid.clone(), c);
                    });
            }
            ProcessingConfig::TsNpm { limit, dir_path } => {
                let commits = self.commits.entry(RepoConfig::TsNpm).or_default();
                rw
                    // .skip(1500)release-1.0.0 refs/tags/release-3.3.2-RC4
                    .take(limit) // TODO make a variable
                    .for_each(|oid| {
                        let oid = oid.unwrap();
                        let c = CommitProcessor::<file_sys::Npm>::handle_commit::<true>(
                            &mut self.processor,
                            &repository,
                            dir_path,
                            oid,
                        );
                        processing_ordered_commits.push(oid.clone());
                        commits.insert(oid.clone(), c);
                    });
            }
            ProcessingConfig::Any { limit, dir_path } => {
                let commits = self.commits.entry(RepoConfig::Any).or_default();
                rw
                    // .skip(1500)release-1.0.0 refs/tags/release-3.3.2-RC4
                    .take(limit) // TODO make a variable
                    .for_each(|oid| {
                        let oid = oid.unwrap();
                        let c = CommitProcessor::<file_sys::Any>::handle_commit::<true>(
                            &mut self.processor,
                            &repository,
                            dir_path,
                            oid,
                        );
                        processing_ordered_commits.push(oid.clone());
                        commits.insert(oid.clone(), c);
                    });
            }
        }
        Ok(processing_ordered_commits)
    }

    pub fn make(
        acc: MavenModuleAcc,
        stores: &mut SimpleStores,
    ) -> (NodeIdentifier, crate::maven::MD) {
        make(acc, stores)
    }
}
