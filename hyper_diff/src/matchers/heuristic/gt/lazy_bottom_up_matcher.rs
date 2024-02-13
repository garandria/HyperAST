use std::{fmt::Debug, marker::PhantomData};

use num_traits::{PrimInt, ToPrimitive};

use crate::{
    decompressed_tree_store::{
        DecompressedTreeStore, DecompressedWithParent, LazyDecompressedTreeStore, Shallow,
    },
    matchers::mapping_store::MonoMappingStore,
};
use hyper_ast::types::{NodeStore, Tree, TypeStore, Typed, WithStats};

pub struct BottomUpMatcher<'a, Dsrc, Ddst, T, HAST, M> {
    pub(super) stores: &'a HAST,
    pub src_arena: Dsrc,
    pub dst_arena: Ddst,
    pub mappings: M,
    pub(super) _phantom: PhantomData<*const T>,
}

impl<
        'a,
        Dsrc: 'a
            + DecompressedTreeStore<'a, T, Dsrc::IdD, M::Src>
            + DecompressedWithParent<'a, T, Dsrc::IdD>
            + LazyDecompressedTreeStore<'a, T, M::Src>,
        Ddst: 'a
            + DecompressedTreeStore<'a, T, Ddst::IdD, M::Dst>
            + DecompressedWithParent<'a, T, Ddst::IdD>
            + LazyDecompressedTreeStore<'a, T, M::Dst>,
        T: 'a + Tree + WithStats,
        HAST: HyperAST<'a, IdN = T::TreeId, T = T>,
        M: MonoMappingStore,
    > BottomUpMatcher<'a, Dsrc, Ddst, T, HAST, M>
where
    // T::Type: Copy + Eq + Send + Sync,
    M::Src: PrimInt + std::ops::SubAssign + Debug,
    M::Dst: PrimInt + std::ops::SubAssign + Debug,
    Dsrc::IdD: PrimInt + std::ops::SubAssign + Debug,
    Ddst::IdD: PrimInt + std::ops::SubAssign + Debug,
{
    pub(super) fn get_dst_candidates(&mut self, src: &Dsrc::IdD) -> Vec<Ddst::IdD> {
        let mut seeds = vec![];
        let s = &self.src_arena.original(src);
        for c in self.src_arena.descendants(self.stores.node_store(), src) {
            if self.mappings.is_src(&c) {
                let m = self.mappings.get_dst_unchecked(&c);
                let m = self.dst_arena.decompress_to(self.stores.node_store(), &m);
                seeds.push(m);
            }
        }
        let mut candidates = vec![];
        let mut visited = bitvec::bitbox![0;self.dst_arena.len()];
        let t = self.stores.resolve_type(s);
        for mut seed in seeds {
            loop {
                let Some(parent) = self.dst_arena.parent(&seed) else {
                    break;
                };
                if visited[parent.to_usize().unwrap()] {
                    break;
                }
                visited.set(parent.to_usize().unwrap(), true);
                let p = &self.dst_arena.original(&parent);
                if self.stores.resolve_type(p) == t
                    && !(self.mappings.is_dst(parent.shallow())
                        || parent.shallow() == &self.dst_arena.root())
                {
                    candidates.push(parent);
                }
                seed = parent;
            }
        }
        candidates
    }
}

use hyper_ast::types::HyperAST;

impl<
        'a,
        HAST: HyperAST<'a>,
        Dsrc: DecompressedTreeStore<'a, HAST::T, Dsrc::IdD, M::Src>
            + DecompressedWithParent<'a, HAST::T, Dsrc::IdD>
            + LazyDecompressedTreeStore<'a, HAST::T, M::Src>,
        Ddst: DecompressedTreeStore<'a, HAST::T, Ddst::IdD, M::Dst>
            + DecompressedWithParent<'a, HAST::T, Ddst::IdD>
            + LazyDecompressedTreeStore<'a, HAST::T, M::Dst>,
        M: MonoMappingStore,
    > crate::matchers::Mapper<'a, HAST, Dsrc, Ddst, M>
where
    HAST::T: 'a + Tree + WithStats,
    // <HAST::T as Typed>::Type: Eq + Copy + Send + Sync,
    M::Src: PrimInt + std::ops::SubAssign + Debug,
    M::Dst: PrimInt + std::ops::SubAssign + Debug,
    Dsrc::IdD: PrimInt + std::ops::SubAssign + Debug,
    Ddst::IdD: PrimInt + std::ops::SubAssign + Debug,
{
    pub(super) fn get_dst_candidates_lazily(&mut self, src: &Dsrc::IdD) -> Vec<Ddst::IdD> {
        let node_store = self.hyperast.node_store();
        let src_arena = &self.mapping.src_arena;
        let dst_arena = &mut self.mapping.dst_arena;
        let mappings = &self.mapping.mappings;
        let mut seeds = vec![];
        let s = &src_arena.original(src);
        for c in src_arena.descendants(node_store, src) {
            if mappings.is_src(&c) {
                let m = mappings.get_dst_unchecked(&c);
                let m = dst_arena.decompress_to(node_store, &m);
                seeds.push(m);
            }
        }
        let mut candidates = vec![];
        let mut visited = bitvec::bitbox![0;dst_arena.len()];
        let t = self.hyperast.resolve_type(s);
        for mut seed in seeds {
            loop {
                let Some(parent) = dst_arena.parent(&seed) else {
                    break;
                };
                if visited[parent.to_usize().unwrap()] {
                    break;
                }
                visited.set(parent.to_usize().unwrap(), true);
                let p = &dst_arena.original(&parent);
                if self.hyperast.resolve_type(p) == t
                    && !(mappings.is_dst(parent.shallow()) || parent.shallow() == &dst_arena.root())
                {
                    candidates.push(parent);
                }
                seed = parent;
            }
        }
        candidates
    }
}
