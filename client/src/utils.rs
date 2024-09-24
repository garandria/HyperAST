use std::collections::HashMap;

use dashmap::{RwLock, SharedValue};
use hyper_ast::{
    store::defaults::NodeIdentifier,
    types::{HyperAST, WithStats},
};
use hyper_diff::decompressed_tree_store::{lazy_post_order, PersistedNode};

pub type LPO<T> = SharedValue<lazy_post_order::LazyPostOrder<T, u32>>;
type IdN = NodeIdentifier;

/// CAUTION a cache should be used on a single HyperAST
/// btw a given HyperAST can be used by multiple caches
pub(crate) fn get_pair_simp<'a, 'store, HAST: HyperAST<'store, IdN = IdN>>(
    partial_comp_cache: &'a crate::PartialDecompCache,
    hyperast: &'store HAST,
    src: &IdN,
    dst: &IdN,
) -> (&'a mut LPO<HAST::T>, &'a mut LPO<HAST::T>)
where
    <HAST as HyperAST<'store>>::T: WithStats,
{
    use hyper_ast::types::DecompressedSubtree;
    use lazy_post_order::LazyPostOrder;

    let (shard1, shard2) = bi_sharding(partial_comp_cache, src, dst);

    let (v1, v2) = if shard2.is_none() {
        let shard1 = shard1.get_mut();
        if !shard1.contains_key(src) {
            shard1.insert(
                src.clone(),
                SharedValue::new({
                    let src = LazyPostOrder::<_, u32>::decompress(hyperast.node_store(), src);
                    let src: LazyPostOrder<PersistedNode<IdN>, u32> =
                        unsafe { std::mem::transmute(src) };
                    src
                }),
            );
        }
        if !shard1.contains_key(dst) {
            shard1.insert(
                dst.clone(),
                SharedValue::new({
                    let dst = LazyPostOrder::<_, u32>::decompress(hyperast.node_store(), dst);
                    let dst: LazyPostOrder<PersistedNode<IdN>, u32> =
                        unsafe { std::mem::transmute(dst) };
                    dst
                }),
            );
        }
        let [v1, v2] = shard1.get_many_mut([src, dst]).unwrap();
        (v1, v2)
    } else {
        let v1 = shard1.get_mut().entry(*src).or_insert_with(|| {
            let src = LazyPostOrder::<_, u32>::decompress(hyperast.node_store(), src);
            let src: LazyPostOrder<PersistedNode<IdN>, u32> = unsafe { std::mem::transmute(src) };
            SharedValue::new(src)
        });
        let v2 = shard2.unwrap().get_mut().entry(*dst).or_insert_with(|| {
            let dst = LazyPostOrder::<_, u32>::decompress(hyperast.node_store(), dst);
            let dst: LazyPostOrder<PersistedNode<IdN>, u32> = unsafe { std::mem::transmute(dst) };
            SharedValue::new(dst)
        });
        (v1, v2)
    };

    // SAFETY: should be the same hyperast TODO check if it is the case, store identifier along map ?
    let res: (
        &mut SharedValue<LazyPostOrder<<HAST as HyperAST<'store>>::T, u32>>,
        &mut SharedValue<LazyPostOrder<<HAST as HyperAST<'store>>::T, u32>>,
    ) = unsafe { std::mem::transmute((v1, v2)) };
    res
}

fn bi_sharding<'a>(
    partial_comp_cache: &'a crate::PartialDecompCache,
    src: &IdN,
    dst: &IdN,
) -> (
    &'a mut RwLock<
        HashMap<
            IdN,
            SharedValue<
                hyper_diff::decompressed_tree_store::lazy_post_order::LazyPostOrder<
                    PersistedNode<IdN>,
                    u32,
                >,
            >,
        >,
    >,
    Option<
        &'a mut RwLock<
            HashMap<
                IdN,
                SharedValue<
                    hyper_diff::decompressed_tree_store::lazy_post_order::LazyPostOrder<
                        PersistedNode<IdN>,
                        u32,
                    >,
                >,
            >,
        >,
    >,
) {
    use hyper_diff::decompressed_tree_store::lazy_post_order::LazyPostOrder;
    let hash1 = partial_comp_cache.hash_usize(src);
    let hash2 = partial_comp_cache.hash_usize(dst);
    let index1 = partial_comp_cache.determine_shard(hash1);
    let index2 = partial_comp_cache.determine_shard(hash2);
    let shards: &[_] = partial_comp_cache.shards();
    let shards = shards as *const [_];
    let shards = shards
        as *mut [RwLock<HashMap<IdN, SharedValue<LazyPostOrder<PersistedNode<IdN>, u32>>>>];
    let shards = unsafe { shards.as_mut().unwrap() };
    // dbg!(index1, index2, shards.len());
    // let mut shards:&mut [_] = unsafe { std::mem::transmute(shards) };

    // let mut shard1: &mut RwLock<HashMap<IdN, SharedValue<LazyPostOrder<PersistedNode<IdN>, u32>>>> = &mut shards[index1];
    let (shard1, shard2) = if index1 == index2 {
        (&mut shards[index1], None)
    } else if index1 < index2 {
        let (shards1, shards2) = shards.split_at_mut(index2);
        (&mut shards1[index1], Some(&mut shards2[0]))
    } else {
        let (shards2, shards1) = shards.split_at_mut(index1);
        (&mut shards1[0], Some(&mut shards2[index2]))
    };
    (shard1, shard2)
}

/// Ensures the range is preprocessed --doing it if needed-- while avoiding to lock global state
pub(crate) fn handle_pre_processing(
    state: &std::sync::Arc<crate::AppState>,
    repo: &mut hyper_ast_cvs_git::processing::ConfiguredRepo2,
    before: &str,
    after: &str,
    limit: usize,
) -> Result<Vec<hyper_ast_cvs_git::git::Oid>, Box<dyn std::error::Error>> {
    let rw = hyper_ast_cvs_git::git::Builder::new(&repo.repo)?
        .before(before)?
        .after(after)?
        .walk()?
        .take(limit)
        .map(|x| x.unwrap());
    // all_commits_between(&repository.repo, before, after)?;
    // NOTE the read with a fallback on a write ensures that we are not waiting to, in the end, not writing anything
    // TODO later start processing the commit subset and schedule the remaining range for processing
    // NOTE a sceduling approach would be much cleaner than the current lock approach
    Ok(handle_pre_processing_aux(state, repo, rw))
}

pub(crate) fn walk_commits_multi<'a, R: AsRef<str>>(
    repo: &'a hyper_ast_cvs_git::processing::ConfiguredRepo2,
    after: impl Iterator<Item = R>,
) -> Result<impl Iterator<Item = hyper_ast_cvs_git::git::Oid> + 'a, Box<dyn std::error::Error>> {
    let mut rw = hyper_ast_cvs_git::git::Builder::new(&repo.repo)?;
    for after in after {
        rw = rw.after(after.as_ref())?;
    }
    let rw = rw.walk()?.map(|x| x.unwrap());
    Ok(rw)
}

/// Ensures the range is preprocessed --doing it if needed-- while avoiding to lock global state
pub(crate) fn handle_pre_processing_aux(
    state: &std::sync::Arc<crate::AppState>,
    repo: &hyper_ast_cvs_git::processing::ConfiguredRepo2,
    rw: impl Iterator<Item = hyper_ast_cvs_git::git::Oid>,
) -> Vec<hyper_ast_cvs_git::git::Oid> {
    let mut rw = rw.peekable();
    let commits = {
        state
            .repositories
            .read()
            .unwrap()
            .processor
            .ensure_prepro(&mut rw, repo)
    };
    match commits {
        Ok(commits) => commits,
        Err(mut commits) => {
            let repository_processor = &mut state.repositories.write().unwrap().processor;
            commits.extend(repository_processor.pre_pro(&mut rw, repo));
            commits
        }
    }
}

// rw: impl Iterator<Item = git2::Oid>,
