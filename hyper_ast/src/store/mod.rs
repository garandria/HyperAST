use std::borrow::Borrow;

use crate::types::{SimpleHyperAST, TypeStore};

pub mod handle;
pub mod labels;
// pub mod mapped_world;
pub mod nodes;
// pub mod ecs; // TODO try a custom ecs ?
// pub mod radix_hash_store; // TODO yet another WIP store
// pub mod vec_map_store; // TODO yet another WIP store

pub struct SimpleStores<TS, NS = nodes::DefaultNodeStore, LS = labels::LabelStore> {
    pub label_store: LS,
    pub type_store: TS,
    pub node_store: NS,
}

impl<TS, NS, LS> SimpleStores<TS, NS, LS> {
    pub fn change_type_store<TS2>(self, new: TS2) -> SimpleStores<TS2, NS, LS> {
        SimpleStores {
            type_store: new,
            node_store: self.node_store,
            label_store: self.label_store,
        }
    }
}

impl<TS: Default, NS: Default, LS: Default> Default for SimpleStores<TS, NS, LS> {
    fn default() -> Self {
        Self {
            label_store: Default::default(),
            type_store: Default::default(),
            node_store: Default::default(),
        }
    }
}

impl<'store, T, TS, NS, LS> crate::types::RoleStore<T> for SimpleStores<TS, NS, LS>
where
    T: crate::types::TypedTree,
    T::TreeId: crate::types::NodeId<IdN = T::TreeId>,
    T::Type: 'static + std::hash::Hash,
    TS: TypeStore<T, Ty = T::Type>,
    NS: crate::types::NodeStore<T::TreeId>,
    TS: crate::types::RoleStore<T>,
{
    type IdF = TS::IdF;

    type Role = TS::Role;

    fn resolve_field(
        &self,
        lang: crate::types::LangWrapper<Self::Ty>,
        field_id: Self::IdF,
    ) -> Self::Role {
        self.type_store.resolve_field(lang, field_id)
    }
    fn intern_role(
        &self,
        lang: crate::types::LangWrapper<Self::Ty>,
        role: Self::Role,
    ) -> Self::IdF {
        self.type_store.intern_role(lang, role)
    }
}

impl<IdN, TS, NS, LS> crate::types::NodeStore<IdN> for SimpleStores<TS, NS, LS>
where
    for<'a> NS::R<'a>: crate::types::Tree<TreeId = IdN>,
    IdN: crate::types::NodeId<IdN = IdN>,
    NS: crate::types::NodeStore<IdN>,
{
    type R<'a> = NS::R<'a>
    where
        Self: 'a;

    fn resolve(&self, id: &IdN) -> Self::R<'_> {
        self.node_store.resolve(id)
    }
}

impl<IdN, TS, NS, LS> crate::types::NodeStoreLean<IdN> for SimpleStores<TS, NS, LS>
where
    NS::R: crate::types::Tree<TreeId = IdN>,
    IdN: crate::types::NodeId<IdN = IdN>,
    NS: crate::types::NodeStoreLean<IdN>,
{
    type R = NS::R;

    fn resolve(&self, id: &IdN) -> Self::R {
        self.node_store.resolve(id)
    }
}

impl<'store, TS, NS, LS> crate::types::LabelStore<str> for SimpleStores<TS, NS, LS>
where
    LS: crate::types::LabelStore<str>,
{
    type I = LS::I;

    fn get_or_insert<U: Borrow<str>>(&mut self, node: U) -> Self::I {
        self.label_store.get_or_insert(node)
    }

    fn get<U: Borrow<str>>(&self, node: U) -> Option<Self::I> {
        self.label_store.get(node)
    }

    fn resolve(&self, id: &Self::I) -> &str {
        self.label_store.resolve(id)
    }
}

impl<'store, T, TS, NS, LS> crate::types::TypeStore<T> for SimpleStores<TS, NS, LS>
where
    T: crate::types::TypedTree,
    T::TreeId: crate::types::NodeId<IdN = T::TreeId>,
    T::Type: 'static + std::hash::Hash,
    TS: TypeStore<T, Ty = T::Type>,
    NS: crate::types::NodeStore<T::TreeId>,
{
    type Ty = TS::Ty;

    fn resolve_type(&self, n: &T) -> Self::Ty {
        self.type_store.resolve_type(n)
    }
    fn resolve_lang(&self, n: &T) -> crate::types::LangWrapper<Self::Ty> {
        self.type_store.resolve_lang(n)
    }

    fn type_eq(&self, n: &T, m: &T) -> bool {
        self.type_store.type_eq(n, m)
    }
}

pub mod defaults {
    pub type LabelIdentifier = super::labels::DefaultLabelIdentifier;
    pub type LabelValue = super::labels::DefaultLabelValue;
    pub type NodeIdentifier = super::nodes::DefaultNodeIdentifier;
}

impl<'store, T, TS, NS, LS> From<&'store SimpleStores<TS, NS, LS>>
    for SimpleHyperAST<T, &'store TS, &'store NS, &'store LS>
{
    fn from(value: &'store SimpleStores<TS, NS, LS>) -> Self {
        Self {
            type_store: &value.type_store,
            node_store: &value.node_store,
            label_store: &value.label_store,
            _phantom: std::marker::PhantomData,
        }
    }
}
