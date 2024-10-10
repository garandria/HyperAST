use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

use num_traits::{cast, zero, PrimInt, ToPrimitive, Zero};

use crate::decompressed_tree_store::{DecompressedTreeStore, PostOrder};
use hyper_ast::types::TypeStore;
use hyper_ast::types::{HyperAST, LabelStore, Labeled, NodeStore, WithChildren, WithSerialization};

use super::FullyDecompressedTreeStore;

pub struct SimplePreOrderMapper<'a, T: WithChildren, IdD, D: DecompressedTreeStore<'a, T, IdD>> {
    pub map: Vec<IdD>,
    // fc: Vec<IdD>,
    rev: Vec<IdD>,
    pub(crate) depth: Vec<u16>,
    back: &'a D,
    phantom: PhantomData<*const T>,
}

impl<'a, T: WithChildren, IdD: Debug, D: Debug + DecompressedTreeStore<'a, T, IdD>> Debug
    for SimplePreOrderMapper<'a, T, IdD, D>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SD")
            .field("map", &self.map)
            .field("rev", &self.rev)
            .field("back", &self.back)
            .field("phantom", &self.phantom)
            .finish()
    }
}

impl<'a, T: 'a + WithChildren, IdD: PrimInt, D: PostOrder<'a, T, IdD>> From<&'a D>
    for SimplePreOrderMapper<'a, T, IdD, D>
where
    D: FullyDecompressedTreeStore<'a, T, IdD>,
{
    fn from(x: &'a D) -> Self {
        let mut map: Vec<IdD> = vec![zero(); x.len()];
        let mut rev: Vec<IdD> = vec![zero(); x.len()];
        let mut depth = vec![0; x.len()];
        let mut o_id = x.root();
        map[0] = o_id;
        let mut fd = x.first_descendant(&o_id);
        let mut d_len = (o_id - fd).to_usize().unwrap();
        (0..d_len).for_each(|x| {
            depth[1 + x] = 1;
        });

        let mut n_id = 0;

        loop {
            if o_id == num_traits::zero() {
                break;
            }
            o_id = o_id - num_traits::one();
            if d_len == 0 {
                while map[n_id] != zero() {
                    n_id = n_id - 1;
                }
            }
            n_id = n_id + d_len;
            fd = x.first_descendant(&o_id);
            d_len = (o_id - fd).to_usize().unwrap();

            n_id = n_id - d_len;

            let dep = depth[n_id] + 1;

            (n_id..n_id + d_len).for_each(|x| {
                depth[1 + x] = dep;
            });

            map[n_id] = o_id;
            rev[o_id.to_usize().unwrap()] = cast(n_id).unwrap();

            if d_len == 0 {
                n_id = n_id - 1;
            }
        }

        Self {
            map,
            // fc,
            rev,
            depth,
            back: x,
            phantom: PhantomData,
        }
    }
}

pub struct DisplaySimplePreOrderMapper<
    'store: 'a,
    'a: 'b,
    'b,
    IdD: PrimInt,
    HAST: HyperAST<'store>,
    D: PostOrder<'a, HAST::T, IdD>,
> {
    pub inner: &'b SimplePreOrderMapper<'a, HAST::T, IdD, D>,
    pub stores: &'store HAST,
}

impl<'store: 'a, 'a: 'b, 'b, IdD: PrimInt, HAST, D> Display
    for DisplaySimplePreOrderMapper<'store, 'a, 'b, IdD, HAST, D>
where
    HAST: HyperAST<'store>,
    HAST::T: WithSerialization,
    // T::TreeId: Clone + Debug + Eq,
    // T::Type: Copy + Send + Sync,
    // T::Type: Debug,
    D: PostOrder<'a, HAST::T, IdD>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut pos = 0;
        for i in 0..self.inner.map.len() {
            let o = self.inner.map[i];
            let ori = self.inner.back.original(&o);
            let node = self.stores.node_store().resolve(&ori);
            let len = node.try_bytes_len().unwrap_or(0);
            writeln!(
                f,
                "{:>3}:{} {:?}    [{},{}]",
                o.to_usize().unwrap(),
                "  ".repeat(self.inner.depth[i].to_usize().unwrap()),
                self.stores.resolve_type(&ori),
                pos,
                pos + len,
            )?;
            if node.child_count().is_zero() {
                pos += len;
            }
        }
        Ok(())
    }
}
impl<'store: 'a, 'a: 'b, 'b, IdD: PrimInt, HAST, D> Debug
    for DisplaySimplePreOrderMapper<'store, 'a, 'b, IdD, HAST, D>
where
    // HAST::IdN: Clone + Debug + Eq,
    // T::Type: Copy + Send + Sync,
    HAST: HyperAST<'store>,
    // T::Type: Debug,
    D: PostOrder<'a, HAST::T, IdD>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            for i in 0..self.inner.map.len() {
                let o = self.inner.map[i];
                let ori = self.inner.back.original(&o);
                let node = self.stores.node_store().resolve(&ori);
                let mut s = self
                    .stores
                    .label_store()
                    .resolve(&node.get_label_unchecked())
                    .to_owned();
                s.truncate(5);
                writeln!(
                    f,
                    "{:>3}:{} {:?}; {}",
                    o.to_usize().unwrap(),
                    "  ".repeat(self.inner.depth[i].to_usize().unwrap()),
                    self.stores.resolve_type(&ori),
                    s.escape_debug()
                )?;
            }
            return Ok(());
        }
        for i in 0..self.inner.map.len() {
            let o = self.inner.map[i];
            let ori = self.inner.back.original(&o);
            let node = self.stores.node_store().resolve(&ori);
            writeln!(
                f,
                "{:>3}:{} {:?}",
                o.to_usize().unwrap(),
                "  ".repeat(self.inner.depth[i].to_usize().unwrap()),
                self.stores.resolve_type(&ori),
            )?;
        }
        Ok(())
    }
}
