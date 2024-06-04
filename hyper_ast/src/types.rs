use std::borrow::Borrow;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;

use num::ToPrimitive;
use strum_macros::AsRefStr;
use strum_macros::Display;
use strum_macros::EnumCount;
use strum_macros::EnumIter;
use strum_macros::EnumString;

use crate::PrimInt;

pub trait HashKind {
    fn structural() -> Self;
    fn label() -> Self;
}

/// TODO handle roles in a polyglote way
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Role {
    Name,
    Scope,
    Body,
    SuperType,
    Interfaces,
    Constructor,
    Object,
    Arguments,
    TypeArguments,
    Type,
    Declarator,
    Value,
    TypeParameters,
    Parameters,
    Condition,
    Init,
    Update,
    Alternative,
    Resources,
    Field,
    Left,
    Right,
    Superclass,
    Element,
    Consequence,
    Key,
}
impl<'a> TryFrom<&'a str> for Role {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "name" => Ok(Role::Name),
            "scope" => Ok(Role::Scope),
            "body" => Ok(Role::Body),
            "super_type" => Ok(Role::SuperType),
            "interfaces" => Ok(Role::Interfaces),
            "constructor" => Ok(Role::Constructor),
            "object" => Ok(Role::Object),
            "arguments" => Ok(Role::Arguments),
            "type_arguments" => Ok(Role::TypeArguments),
            "type" => Ok(Role::Type),
            "declarator" => Ok(Role::Declarator),
            "value" => Ok(Role::Value),
            "type_parameters" => Ok(Role::TypeParameters),
            "parameters" => Ok(Role::Parameters),
            "condition" => Ok(Role::Condition),
            "init" => Ok(Role::Init),
            "update" => Ok(Role::Update),
            "alternative" => Ok(Role::Alternative),
            "resources" => Ok(Role::Resources),
            "field" => Ok(Role::Field),
            "left" => Ok(Role::Left),
            "right" => Ok(Role::Right),
            "superclass" => Ok(Role::Superclass),
            "element" => Ok(Role::Element),
            "consequence" => Ok(Role::Consequence),
            "key" => Ok(Role::Key),
            _ => Err(()),
        }
    }
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Role::Name => "name",
            Role::Scope => "scope",
            Role::Body => "body",
            Role::SuperType => "super_type",
            Role::Interfaces => "interfaces",
            Role::Constructor => "constructor",
            Role::Object => "object",
            Role::Arguments => "arguments",
            Role::TypeArguments => "type_arguments",
            Role::Type => "type",
            Role::Declarator => "declarator",
            Role::Value => "value",
            Role::TypeParameters => "type_parameters",
            Role::Parameters => "parameters",
            Role::Condition => "condition",
            Role::Init => "init",
            Role::Update => "update",
            Role::Alternative => "alternative",
            Role::Resources => "resources",
            Role::Field => "field",
            Role::Left => "left",
            Role::Right => "right",
            Role::Superclass => "superclass",
            Role::Element => "element",
            Role::Consequence => "consequence",
            Role::Key => "key",
        })
    }
}

#[allow(unused)]
mod exp {
    use super::*;

    // keywords (leafs with a specific unique serialized form)
    // and concrete types (concrete rules) should definitely be stored.
    // But hidden nodes are can either be supertypes or nodes that are just deemed uninteresting (but still useful to for example the treesitter internal repr.)
    // The real important difference is the (max) number of children (btw an it cannot be a leaf (at least one child)),
    // indeed, with a single child it is possible to easily implement optimization that effectively reduce the number of nodes.
    // - a supertype should only have a single child
    // - in tree-sitter repeats (star and plus patterns) are binary nodes (sure balanced?)
    // - in tree-sitter other nodes can be hidden (even when they have fields), it can be espetially useful to add more structure without breaking existing queries !
    // Anyway lets wait for better type generation, this way it should be possible to explicitely/completely handle optimizable cases (supertypes,...)

    #[repr(transparent)]
    pub struct T(u16);

    #[repr(u16)]
    pub enum T2 {
        Java(u16),
        Cpp(u16),
    }

    // pub trait Lang {
    //     type Factory;
    //     type Type;
    // }

    trait TypeFactory {
        fn new() -> Self
        where
            Self: Sized;
    }

    mod polyglote {
        /// has statements
        struct Block;
        /// has a name
        struct Member;
    }

    // WARN order of fields matter in java for instantiation
    // stuff where order does not matter should be sorted before erasing anything

    pub enum TypeMapElement<Concrete, Abstract> {
        Keyword(Keyword),
        Concrete(Concrete),
        Abstract(Abstract),
    }

    pub enum ConvertResult<Concrete, Abstract> {
        Keyword(Keyword),
        Concrete(Concrete),
        Abstract(Abstract),
        Missing,
    }

    trait KeywordProvider: Sized {
        fn parse(&self, s: &str) -> Option<Self>;
        fn as_str(&'static self) -> &'static str;
        fn len(&self) -> usize;
    }

    /// only contains keywords such as
    #[derive(Debug, EnumString, AsRefStr, EnumIter, EnumCount, Display)]
    #[strum(serialize_all = "snake_case")]
    #[derive(Hash, Clone, Copy, PartialEq, Eq)]
    pub enum Keyword {
        // While,
        // For,
        // #[strum(serialize = ";")]
        // SemiColon,
        // #[strum(serialize = ".")]
        // Dot,
        // #[strum(serialize = "{")]
        // LeftCurly,
        // #[strum(serialize = "}")]
        // RightCurly,
    }

    impl KeywordProvider for Keyword {
        fn parse(&self, s: &str) -> Option<Self> {
            Keyword::from_str(s).ok()
        }

        fn as_str(&'static self) -> &'static str {
            Keyword::as_ref(&self)
        }

        fn len(&self) -> usize {
            <Keyword as strum::EnumCount>::COUNT
        }
    }

    mod macro_test {
        macro_rules! parse_unitary_variants {
        (@as_expr $e:expr) => {$e};
        (@as_item $($i:item)+) => {$($i)+};

        // Exit rules.
        (
            @collect_unitary_variants ($callback:ident ( $($args:tt)* )),
            ($(,)*) -> ($($var_names:ident,)*)
        ) => {
            parse_unitary_variants! {
                @as_expr
                $callback!{ $($args)* ($($var_names),*) }
            }
        };

        (
            @collect_unitary_variants ($callback:ident { $($args:tt)* }),
            ($(,)*) -> ($($var_names:ident,)*)
        ) => {
            parse_unitary_variants! {
                @as_item
                $callback!{ $($args)* ($($var_names),*) }
            }
        };

        // Consume an attribute.
        (
            @collect_unitary_variants $fixed:tt,
            (#[$_attr:meta] $($tail:tt)*) -> ($($var_names:tt)*)
        ) => {
            parse_unitary_variants! {
                @collect_unitary_variants $fixed,
                ($($tail)*) -> ($($var_names)*)
            }
        };

        // Handle a variant, optionally with an with initialiser.
        (
            @collect_unitary_variants $fixed:tt,
            ($var:ident $(= $_val:expr)*, $($tail:tt)*) -> ($($var_names:tt)*)
        ) => {
            parse_unitary_variants! {
                @collect_unitary_variants $fixed,
                ($($tail)*) -> ($($var_names)* $var,)
            }
        };

        // Abort on variant with a payload.
        (
            @collect_unitary_variants $fixed:tt,
            ($var:ident $_struct:tt, $($tail:tt)*) -> ($($var_names:tt)*)
        ) => {
            const _error: () = "cannot parse unitary variants from enum with non-unitary variants";
        };

        // Entry rule.
        (enum $name:ident {$($body:tt)*} => $callback:ident $arg:tt) => {
            parse_unitary_variants! {
                @collect_unitary_variants
                ($callback $arg), ($($body)*,) -> ()
            }
        };
    }

        macro_rules! coucou {
            ( f(C, D)) => {
                struct B {}
            };
        }
        parse_unitary_variants! {
            enum A {
                C,D,
            } => coucou{ f}
        }
    }

    macro_rules! make_type {
        (
            Keyword {$(
                $(#[$km:meta])*
                $ka:ident
            ),* $(,)?}
            Concrete {$(
                $(#[$cm:meta])*
                $ca:ident$({$($cl:expr),+ $(,)*})?$(($($co:ident),+ $(,)*))?$([$($cx:ident),+ $(,)*])?
            ),* $(,)?}
            WithFields {$(
                $(#[$wm:meta])*
                $wa:ident{$($wb:tt)*}
            ),* $(,)?}
            Abstract {$(
                $(#[$am:meta])*
                $aa:ident($($ab:ident),* $(,)?)
            ),* $(,)?}
        ) => {
            #[derive(Debug, EnumString, AsRefStr, EnumIter, EnumCount, Display)]
            #[strum(serialize_all = "snake_case")]
            #[derive(Hash, Clone, Copy, PartialEq, Eq)]
            pub enum Type {
                // Keywords
            $(
                $( #[$km] )*
                $ka,
            )*
                // Concrete
            $(
                $ca,
            )*
                // WithFields
            $(
                $( #[$wm] )*
                $wa,
            )*
            }
            enum Abstract {
                $(
                    $aa,
                )*
            }

            pub struct Factory {
                map: Box<[u16]>,
            }

            pub struct Language;
        };
    }

    macro_rules! make_type_store {
    ($kw:ty, $sh:ty, $($a:ident($l:ty)),* $(,)?) => {

        #[repr(u16)]
        pub enum CustomTypeStore {$(
            $a(u16),
        )*}

        impl CustomTypeStore {
            // fn lang<L: Lang>(&self) -> Option<L> {
            //     todo!()
            // }
            fn eq_keyword(kw: &$kw) -> bool {
                todo!()
            }
            fn eq_shared(kw: &$sh) -> bool {
                todo!()
            }
        }
    };
}

    make_type_store!(Keyword, Shared, Java(java::Language), Cpp(cpp::Language),);

    pub mod java {
        use super::*;

        pub enum Field {
            Name,
            Body,
            Expression,
            Condition,
            Then,
            Else,
            Block,
            Type,
        }

        make_type! {
            Keyword{
                While,
                For,
                Public,
                Private,
                Protected,
                #[strum(serialize = ";")]
                SemiColon,
                #[strum(serialize = ".")]
                Dot,
                #[strum(serialize = "{")]
                LeftCurly,
                #[strum(serialize = "}")]
                RightCurly,
                #[strum(serialize = "(")]
                LeftParen,
                #[strum(serialize = ")")]
                RightParen,
                #[strum(serialize = "[")]
                LeftBracket,
                #[strum(serialize = "]")]
                RightBracket,
            }
            Concrete {
                Comment{r"//.\*$",r"/\*.*\*/"},
                Identifier{r"[a-zA-Z].*"},
                ExpressionStatement(Statement, Semicolon),
                ReturnStatement(Return, Expression, Semicolon),
                TryStatement(Try, Paren, Block),
            }
            WithFields {
                Class {
                    name(Identifier),
                    body(ClassBody),
                },
                Interface {
                    name(Identifier),
                    body(InterfaceBody),
                },
            }
            Abstract {
                Statement(
                    StatementExpression,
                    TryStatement,
                ),
                Expression(
                    BinaryExpression,
                    UnaryExpression,
                ),
            }
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq, EnumString, AsRefStr, EnumIter, EnumCount, Display)]
#[strum(serialize_all = "snake_case")]
enum Abstract {
    Expression,
    Statement,
    Executable,
    Declaration,
    Literal,
}

#[derive(Debug, EnumString, AsRefStr, EnumIter, EnumCount, Display)]
#[strum(serialize_all = "snake_case")]
#[derive(Hash, Clone, Copy, PartialEq, Eq)]
pub enum Shared {
    Comment,
    // ExpressionStatement,
    // ReturnStatement,
    // TryStatement,
    Identifier,
    TypeDeclaration,
    Other,
    // WARN do not include Abtract type/rules (should go in Abstract) ie.
    // Expression,
    // Statement,
}

pub trait Lang<T>: LangRef<T> {
    fn make(t: u16) -> &'static T;
    fn to_u16(t: T) -> u16;
}

pub trait LangRef<T> {
    fn name(&self) -> &'static str;
    fn make(&self, t: u16) -> &'static T;
    fn to_u16(&self, t: T) -> u16;
    fn ts_symbol(&self, t: T) -> u16;
}

pub struct LangWrapper<T: 'static + ?Sized>(&'static dyn LangRef<T>);

impl<T> From<&'static (dyn LangRef<T> + 'static)> for LangWrapper<T> {
    fn from(value: &'static (dyn LangRef<T> + 'static)) -> Self {
        LangWrapper(value)
    }
}

impl<T> LangRef<T> for LangWrapper<T> {
    fn make(&self, t: u16) -> &'static T {
        self.0.make(t)
    }

    fn to_u16(&self, t: T) -> u16 {
        self.0.to_u16(t)
    }

    fn name(&self) -> &'static str {
        self.0.name()
    }

    fn ts_symbol(&self, t: T) -> u16 {
        self.0.ts_symbol(t)
    }
}

// trait object used to facilitate erasing node types
pub trait HyperType: Display + Debug {
    fn as_shared(&self) -> Shared;
    fn as_any(&self) -> &dyn std::any::Any;
    // returns the same address for the same type
    fn as_static(&self) -> &'static dyn HyperType;
    fn as_static_str(&self) -> &'static str;
    fn generic_eq(&self, other: &dyn HyperType) -> bool
    where
        Self: 'static + Sized;
    fn is_file(&self) -> bool;
    fn is_directory(&self) -> bool;
    fn is_spaces(&self) -> bool;
    fn is_syntax(&self) -> bool;
    fn is_hidden(&self) -> bool;
    fn is_named(&self) -> bool;
    fn is_supertype(&self) -> bool;
    fn get_lang(&self) -> LangWrapper<Self> where Self: Sized;
    fn lang_ref(&self) -> LangWrapper<AnyType>;
}

// experiment
// NOTE: it might actually be a good way to share types between languages.
// EX on a u16: lang on 4 bits, supertypes on 4 bits, concrete and hidden on the 8 remaining bits.
// lets also say the super types are precomputed on shared types.
// TODO still need to think about it

impl HyperType for u8 {
    fn generic_eq(&self, other: &dyn HyperType) -> bool
    where
        Self: 'static + PartialEq + Sized,
    {
        // Do a type-safe casting. If the types are different,
        // return false, otherwise test the values for equality.
        other
            .as_any()
            .downcast_ref::<Self>()
            .map_or(false, |a| self == a)
    }

    fn as_shared(&self) -> Shared {
        todo!()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        todo!()
    }

    fn as_static(&self) -> &'static dyn HyperType {
        todo!()
    }

    fn as_static_str(&self) -> &'static str {
        todo!()
    }

    fn is_file(&self) -> bool {
        todo!()
    }

    fn is_directory(&self) -> bool {
        todo!()
    }

    fn is_spaces(&self) -> bool {
        todo!()
    }

    fn is_syntax(&self) -> bool {
        todo!()
    }

    fn is_hidden(&self) -> bool {
        todo!()
    }

    fn is_supertype(&self) -> bool {
        todo!()
    }

    fn is_named(&self) -> bool {
        todo!()
    }

    fn get_lang(&self) -> LangWrapper<Self>
    where
        Self: Sized,
    {
        todo!()
    }
    fn lang_ref(&self) -> LangWrapper<AnyType> {
        todo!()
    }
}

pub trait TypeTrait: HyperType + Hash + Copy + Eq + Send + Sync {
    type Lang: Lang<Self>;
    fn is_fork(&self) -> bool;

    fn is_literal(&self) -> bool;
    fn is_primitive(&self) -> bool;
    fn is_type_declaration(&self) -> bool;
    fn is_identifier(&self) -> bool;
    fn is_instance_ref(&self) -> bool;

    fn is_type_body(&self) -> bool;

    fn is_value_member(&self) -> bool;

    fn is_executable_member(&self) -> bool;

    fn is_statement(&self) -> bool;

    fn is_declarative_statement(&self) -> bool;

    fn is_structural_statement(&self) -> bool;

    fn is_block_related(&self) -> bool;

    fn is_simple_statement(&self) -> bool;

    fn is_local_declare(&self) -> bool;

    fn is_parameter(&self) -> bool;

    fn is_parameter_list(&self) -> bool;

    fn is_argument_list(&self) -> bool;

    fn is_expression(&self) -> bool;
    fn is_comment(&self) -> bool;
}

pub trait Node {}

pub trait AsTreeRef<T> {
    fn as_tree_ref(&self) -> T;
}

pub trait Stored: Node {
    type TreeId: NodeId;
}

pub trait Typed {
    type Type: HyperType + Eq + Copy + Send + Sync; // todo try remove Copy
    fn get_type(&self) -> Self::Type; // TODO add TypeTrait bound on Self::Type to forbid AnyType from being given
    fn try_get_type(&self) -> Option<Self::Type> {
        Some(self.get_type())
    }
}

pub trait WithChildren: Node + Stored {
    type ChildIdx: PrimInt;
    type Children<'a>: Children<Self::ChildIdx, <Self::TreeId as NodeId>::IdN> + ?Sized
    where
        Self: 'a;

    fn child_count(&self) -> Self::ChildIdx;
    fn child(&self, idx: &Self::ChildIdx) -> Option<<Self::TreeId as NodeId>::IdN>;
    fn child_rev(&self, idx: &Self::ChildIdx) -> Option<<Self::TreeId as NodeId>::IdN>;
    fn children(&self) -> Option<&Self::Children<'_>>;
}

pub trait WithRoles: WithChildren {
    fn role_at<Role: 'static + Copy + std::marker::Sync + std::marker::Send>(
        &self,
        at: Self::ChildIdx,
    ) -> Option<Role>;
}

pub trait WithChildrenSameLang: WithChildren {
    type TChildren<'a>: Children<Self::ChildIdx, Self::TreeId> + ?Sized
    where
        Self: 'a;

    fn child_count(&self) -> Self::ChildIdx;
    fn child(&self, idx: &Self::ChildIdx) -> Option<Self::TreeId>;
    fn child_rev(&self, idx: &Self::ChildIdx) -> Option<Self::TreeId>;
    fn children(&self) -> Option<&Self::Children<'_>>;
}

pub trait IterableChildren<T> {
    type ChildrenIter<'a>: Iterator<Item = &'a T> + Clone
    where
        T: 'a,
        Self: 'a;
    fn iter_children(&self) -> Self::ChildrenIter<'_>;
    fn is_empty(&self) -> bool;
}

pub trait Children<IdX, T>: std::ops::Index<IdX, Output = T> + IterableChildren<T> {
    fn child_count(&self) -> IdX;
    fn get(&self, i: IdX) -> Option<&T>;
    fn rev(&self, i: IdX) -> Option<&T>;
    fn after(&self, i: IdX) -> &Self;
    fn before(&self, i: IdX) -> &Self;
    fn between(&self, start: IdX, end: IdX) -> &Self;
    fn inclusive(&self, start: IdX, end: IdX) -> &Self;
}

// pub trait AsSlice<'a, IdX, T: 'a> {
//     type Slice: std::ops::Index<IdX, Output = [T]> + ?Sized;

//     fn as_slice(&self) -> &Self::Slice;
// }

impl<T> IterableChildren<T> for [T] {
    type ChildrenIter<'a> = core::slice::Iter<'a, T> where T: 'a;

    fn iter_children(&self) -> Self::ChildrenIter<'_> {
        <[T]>::iter(&self)
    }

    fn is_empty(&self) -> bool {
        <[T]>::is_empty(&self)
    }
}

impl<IdX: num::NumCast, T> Children<IdX, T> for [T]
where
    IdX: std::slice::SliceIndex<[T], Output = T>,
{
    fn child_count(&self) -> IdX {
        IdX::from(<[T]>::len(&self)).unwrap()
        // num::cast::<_, IdX>(<[T]>::len(&self)).unwrap()
    }

    fn get(&self, i: IdX) -> Option<&T> {
        self.get(i.to_usize()?)
    }

    fn rev(&self, idx: IdX) -> Option<&T> {
        let c = <[T]>::len(&self);
        let c = c.checked_sub(idx.to_usize()?.checked_add(1)?)?;
        self.get(c.to_usize()?)
    }

    fn after(&self, i: IdX) -> &Self {
        (&self[i.to_usize().unwrap()..]).into()
    }

    fn before(&self, i: IdX) -> &Self {
        (&self[..i.to_usize().unwrap()]).into()
    }

    fn between(&self, start: IdX, end: IdX) -> &Self {
        (&self[start.to_usize().unwrap()..end.to_usize().unwrap()]).into()
    }

    fn inclusive(&self, start: IdX, end: IdX) -> &Self {
        (&self[start.to_usize().unwrap()..=end.to_usize().unwrap()]).into()
    }
}

#[repr(transparent)]
pub struct MySlice<T>(pub [T]);

impl<'a, T> From<&'a [T]> for &'a MySlice<T> {
    fn from(value: &'a [T]) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl<T> std::ops::Index<u16> for MySlice<T> {
    type Output = T;

    fn index(&self, index: u16) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl<T> std::ops::Index<u8> for MySlice<T> {
    type Output = T;

    fn index(&self, index: u8) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl<T> std::ops::Index<usize> for MySlice<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T: Clone> From<&MySlice<T>> for Vec<T> {
    fn from(value: &MySlice<T>) -> Self {
        value.0.to_vec()
    }
}

impl<T: Debug> Debug for MySlice<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<T: Debug> Default for &MySlice<T> {
    fn default() -> Self {
        let r: &[T] = &[];
        r.into()
    }
}

impl<T> IterableChildren<T> for MySlice<T> {
    type ChildrenIter<'a> = core::slice::Iter<'a, T> where T: 'a;

    fn iter_children(&self) -> Self::ChildrenIter<'_> {
        <[T]>::iter(&self.0)
    }

    fn is_empty(&self) -> bool {
        <[T]>::is_empty(&self.0)
    }
}

impl<T> Children<u16, T> for MySlice<T> {
    fn child_count(&self) -> u16 {
        <[T]>::len(&self.0).to_u16().unwrap()
    }

    fn get(&self, i: u16) -> Option<&T> {
        self.0.get(usize::from(i))
    }

    fn rev(&self, idx: u16) -> Option<&T> {
        let c: u16 = self.child_count();
        let c = c.checked_sub(idx.checked_add(1)?)?;
        self.get(c)
    }

    fn after(&self, i: u16) -> &Self {
        (&self.0[i.into()..]).into()
    }

    fn before(&self, i: u16) -> &Self {
        (&self.0[..i.into()]).into()
    }

    fn between(&self, start: u16, end: u16) -> &Self {
        (&self.0[start.into()..end.into()]).into()
    }

    fn inclusive(&self, start: u16, end: u16) -> &Self {
        (&self.0[start.into()..=end.into()]).into()
    }
}

impl<T> Children<u8, T> for MySlice<T> {
    fn child_count(&self) -> u8 {
        <[T]>::len(&self.0).to_u8().unwrap()
    }

    fn get(&self, i: u8) -> Option<&T> {
        self.0.get(usize::from(i))
    }

    fn rev(&self, idx: u8) -> Option<&T> {
        let c: u8 = self.child_count();
        let c = c.checked_sub(idx.checked_add(1)?)?;
        self.get(c)
    }

    fn after(&self, i: u8) -> &Self {
        (&self.0[i.into()..]).into()
    }

    fn before(&self, i: u8) -> &Self {
        (&self.0[..i.into()]).into()
    }

    fn between(&self, start: u8, end: u8) -> &Self {
        (&self.0[start.into()..end.into()]).into()
    }

    fn inclusive(&self, start: u8, end: u8) -> &Self {
        (&self.0[start.into()..=end.into()]).into()
    }
}

/// just to show that it is not efficient
/// NOTE: it might prove necessary for ecs like hecs
mod owned {
    use std::cell::{Ref, RefMut};

    use super::*;

    pub trait WithChildren: Node {
        type ChildIdx: PrimInt;

        fn child_count(&self) -> Self::ChildIdx;
        fn get_child(&self, idx: &Self::ChildIdx) -> RefMut<Self>;
        fn get_child_mut(&mut self, idx: &Self::ChildIdx) -> Ref<Self>;
    }
    pub trait WithParent: Node {
        fn get_parent(&self) -> Ref<Self>;
        fn get_parent_mut(&mut self) -> RefMut<Self>;
    }
}

pub trait WithStats {
    fn size(&self) -> usize;
    fn height(&self) -> usize;
    fn line_count(&self) -> usize;
}
pub trait WithMetaData<C> {
    fn get_metadata(&self) -> Option<&C>;
}

pub trait WithSerialization {
    fn try_bytes_len(&self) -> Option<usize>;
}

pub trait WithHashs {
    type HK: HashKind;
    type HP: PrimInt + PartialEq + Eq;
    fn hash(&self, kind: &Self::HK) -> Self::HP;
}

pub trait Labeled {
    type Label: Eq;
    fn get_label_unchecked<'a>(&'a self) -> &'a Self::Label;
    fn try_get_label<'a>(&'a self) -> Option<&'a Self::Label>;
}
pub trait Tree: Labeled + WithChildren {
    fn has_children(&self) -> bool;
    fn has_label(&self) -> bool;
}

pub trait TypedTree: Typed + Tree {}

impl<T> TypedTree for T where Self: Typed + Tree {}

pub trait DeCompressedTree<T: PrimInt>: Tree {
    fn get_parent(&self) -> T;
}

pub trait TreePath {}

pub trait GenericItem<'a> {
    type Item;
}

pub trait NodeStore<IdN> {
    type R<'a>
    where
        Self: 'a;
    fn resolve(&self, id: &IdN) -> Self::R<'_>;
}

pub trait NodeStoreLean<IdN> {
    type R;
    fn resolve(&self, id: &IdN) -> Self::R;
}

pub trait NodeStoreLife<'store, IdN> {
    type R<'s>
    where
        Self: 's,
        Self: 'store;
    fn resolve(&'store self, id: &IdN) -> Self::R<'store>;
}

pub trait NodeId: Eq + Clone {
    type IdN: Eq + NodeId;
    fn as_id(&self) -> &Self::IdN;
    // fn as_ty(&self) -> &Self::Ty;
    unsafe fn from_id(id: Self::IdN) -> Self;
    unsafe fn from_ref_id(id: &Self::IdN) -> &Self;
}

impl NodeId for u16 {
    type IdN = u16;
    fn as_id(&self) -> &Self::IdN {
        self
    }
    unsafe fn from_id(id: Self::IdN) -> Self {
        id
    }

    unsafe fn from_ref_id(id: &Self::IdN) -> &Self {
        id
    }
}

pub trait TypedNodeId: NodeId {
    type Ty: HyperType + Hash + Copy + Eq + Send + Sync;
}

pub trait TypedNodeStore<IdN: TypedNodeId> {
    type R<'a>: Typed<Type = IdN::Ty>
    where
        Self: 'a;
    fn try_typed(&self, id: &IdN::IdN) -> Option<IdN>;
    fn try_resolve(&self, id: &IdN::IdN) -> Option<(Self::R<'_>, IdN)> {
        self.try_typed(id).map(|x| (self.resolve(&x), x))
    }
    fn resolve(&self, id: &IdN) -> Self::R<'_>;
}

pub trait TypedNodeStoreLean<IdN: TypedNodeId> {
    type R: Typed<Type = IdN::Ty>;
    fn try_typed(&self, id: &IdN::IdN) -> Option<IdN>;
    fn try_resolve(&self, id: &IdN::IdN) -> Option<(Self::R, IdN)> {
        self.try_typed(id).map(|x| (self.resolve(&x), x))
    }
    fn resolve(&self, id: &IdN) -> Self::R;
}

pub trait DecompressedSubtree<'a, T: Stored> {
    type Out: DecompressedSubtree<'a, T>;
    fn decompress<S>(store: &'a S, id: &T::TreeId) -> Self::Out
    where
        S: NodeStore<T::TreeId, R<'a> = T>;
}

pub trait DecompressibleNodeStore<IdN>: NodeStore<IdN> {
    fn decompress<'a, D: DecompressedSubtree<'a, Self::R<'a>>>(
        &'a self,
        id: &IdN,
    ) -> (&'a Self, D::Out)
    where
        Self: Sized,
        Self::R<'a>: Stored<TreeId = IdN>,
    {
        (self, D::decompress(self, id))
    }

    fn decompress_pair<'a, D1, D2>(&'a self, id1: &IdN, id2: &IdN) -> (&'a Self, (D1::Out, D2::Out))
    where
        Self: Sized,
        Self::R<'a>: Stored<TreeId = IdN>,
        D1: DecompressedSubtree<'a, Self::R<'a>>,
        D2: DecompressedSubtree<'a, Self::R<'a>>,
    {
        (self, (D1::decompress(self, id1), D2::decompress(self, id2)))
    }
}

impl<IdN, S> DecompressibleNodeStore<IdN> for S where S: NodeStore<IdN> {}

pub trait NodeStoreMut<T: Stored> {
    fn get_or_insert(&mut self, node: T) -> T::TreeId;
}
pub trait NodeStoreExt<T: TypedTree> {
    fn build_then_insert(
        &mut self,
        i: T::TreeId,
        t: T::Type,
        l: Option<T::Label>,
        cs: Vec<T::TreeId>,
    ) -> T::TreeId;
}

pub trait VersionedNodeStore<'a, IdN>: NodeStore<IdN> {
    fn resolve_root(&self, version: (u8, u8, u8), node: IdN);
}

pub trait VersionedNodeStoreMut<'a, T: Stored>: NodeStoreMut<T>
where
    T::TreeId: Clone,
{
    // fn insert_as_root(&mut self, version: (u8, u8, u8), node: T) -> T::TreeId;
    //  {
    //     let r = self.get_or_insert(node);
    //     self.as_root(version, r.clone());
    //     r
    // }

    fn as_root(&mut self, version: (u8, u8, u8), node: T::TreeId);
}

pub type OwnedLabel = String;
pub type SlicedLabel = str;

pub trait LabelStore<L: ?Sized> {
    type I: Copy + Eq;

    fn get_or_insert<T: Borrow<L>>(&mut self, node: T) -> Self::I;

    fn get<T: Borrow<L>>(&self, node: T) -> Option<Self::I>;

    fn resolve(&self, id: &Self::I) -> &L;
}

type TypeInternalSize = u16;
pub trait TypeStore<T> {
    type Ty: 'static
        + HyperType
        + Eq
        + std::hash::Hash
        + Copy
        + std::marker::Send
        + std::marker::Sync;
    const MASK: TypeInternalSize;
    fn resolve_type(&self, n: &T) -> Self::Ty;
    fn resolve_lang(&self, n: &T) -> LangWrapper<Self::Ty>;
    type Marshaled;
    fn marshal_type(&self, n: &T) -> Self::Marshaled;
    fn type_eq(&self, n: &T, m: &T) -> bool;
    fn type_to_u16(&self, t: Self::Ty) -> u16 {
        t.get_lang().ts_symbol(t)
    }
}

pub trait SpecializedTypeStore<T: Typed>: TypeStore<T> {}

pub trait RoleStore {
    type IdF: 'static + Copy;
    type Role: 'static + Copy + PartialEq + std::marker::Sync + std::marker::Send;
    fn resolve_field(&self, field_id: Self::IdF) -> Self::Role;
    fn intern_role(&self, role: Self::Role) -> Self::IdF;
}

pub trait HyperAST<'store> {
    type IdN: NodeId<IdN = Self::IdN>;
    type Idx: PrimInt;
    type Label;
    type T: Tree<Label = Self::Label, TreeId = Self::IdN, ChildIdx = Self::Idx>;
    type NS: 'store + NodeStore<Self::IdN, R<'store> = Self::T>;
    fn node_store(&self) -> &Self::NS;

    type LS: LabelStore<str, I = Self::Label>;
    fn label_store(&self) -> &Self::LS;

    type TS: TypeStore<Self::T>;
    fn type_store(&self) -> &Self::TS;

    fn decompress<D: DecompressedSubtree<'store, Self::T, Out = D>>(
        &'store self,
        id: &Self::IdN,
    ) -> (&'store Self, D)
    where
        Self: Sized,
    {
        {
            (self, D::decompress(self.node_store(), id))
        }
    }

    fn decompress_pair<D1, D2>(
        &'store self,
        id1: &Self::IdN,
        id2: &Self::IdN,
    ) -> (&'store Self, (D1, D2))
    where
        Self: Sized,
        D1: DecompressedSubtree<'store, Self::T, Out = D1>,
        D2: DecompressedSubtree<'store, Self::T, Out = D2>,
    {
        {
            (
                self,
                (
                    D1::decompress(self.node_store(), id1),
                    D2::decompress(self.node_store(), id2),
                ),
            )
        }
    }
    fn resolve_type(&'store self, id: &Self::IdN) -> <Self::TS as TypeStore<Self::T>>::Ty {
        let ns = self.node_store();
        let n = ns.resolve(id);
        self.type_store().resolve_type(&n).clone()
    }
}

pub trait HyperASTShared {
    type IdN: NodeId;
    type Idx: PrimInt;
    type Label;
}

impl<T> HyperASTShared for &T
where
    T: HyperASTShared,
{
    type IdN = T::IdN;
    type Idx = T::Idx;
    type Label = T::Label;
}

pub trait HyperASTLean: HyperASTShared {
    type T: Tree<Label = Self::Label, TreeId = Self::IdN, ChildIdx = Self::Idx>;

    type NS;
    fn node_store(&self) -> &Self::NS
    where
        for<'a> &'a Self::NS: NodeStoreLean<Self::IdN, R = Self::T>;

    type LS: LabelStore<str, I = Self::Label>;
    fn label_store(&self) -> &Self::LS;

    type TS: TypeStore<Self::T>;
    fn type_store(&self) -> &Self::TS;

    fn resolve_type(&self, id: &Self::IdN) -> <Self::TS as TypeStore<Self::T>>::Ty
    where
        for<'a> &'a Self::NS: NodeStoreLean<Self::IdN, R = Self::T>,
    {
        let ns = self.node_store();
        let n = ns.resolve(id);
        self.type_store().resolve_type(&n).clone()
    }
}

pub trait HyperASTAsso: HyperASTShared {
    type T<'store>: Tree<Label = Self::Label, TreeId = Self::IdN, ChildIdx = Self::Idx>
    where
        Self: 'store;

    type NS<'store>: NodeStore<Self::IdN, R<'store> = Self::T<'store>>
    where
        Self: 'store,
        Self::T<'store>: 'store;
    fn node_store<'a>(&'a self) -> &'a Self::NS<'a>;
    fn node_store2<'a, 'b>(&'a self) -> Self::NS<'b> {
        panic!()
    }
    fn node_store3(&self) -> Self::NS<'_> {
        panic!()
    }

    type LS: LabelStore<str, I = Self::Label>;
    fn label_store(&self) -> &Self::LS;

    type TS<'store>: TypeStore<Self::T<'store>>
    where
        Self: 'store;
    fn type_store(&self) -> &Self::TS<'_>;

    fn resolve_type(&self, id: &Self::IdN) -> <Self::TS<'_> as TypeStore<Self::T<'_>>>::Ty {
        let ns = self.node_store();
        let n = ns.resolve(id);
        self.type_store().resolve_type(&n).clone()
    }
}

impl<T> HyperASTLean for &T
where
    T: HyperASTLean,
{
    type T = T::T;

    type NS = T::NS;
    fn node_store(&self) -> &T::NS
    where
        for<'a> &'a Self::NS: NodeStoreLean<Self::IdN, R = Self::T>,
    {
        (*self).node_store()
    }

    type LS = T::LS;
    fn label_store(&self) -> &Self::LS {
        (*self).label_store()
    }

    type TS = T::TS;
    fn type_store(&self) -> &Self::TS {
        (*self).type_store()
    }

    fn resolve_type(&self, id: &Self::IdN) -> <Self::TS as TypeStore<Self::T>>::Ty
    where
        for<'a> &'a Self::NS: NodeStoreLean<Self::IdN, R = Self::T>,
    {
        (*self).resolve_type(id)
    }
}

pub trait TypedHyperASTLean<TIdN: TypedNodeId<IdN = Self::IdN>>: HyperASTLean {
    type TT: TypedTree<
        Type = TIdN::Ty,
        TreeId = TIdN::IdN,
        Label = Self::Label,
        ChildIdx = <<Self as HyperASTLean>::T as WithChildren>::ChildIdx,
    >;
    // type TNS<'a> where &'a Self::TNS<'a>: TypedNodeStoreLean<Self::IdN, R = Self::T>, Self: 'a;
    // fn typed_node_store(&self) -> Self::TNS<'_>;
}

pub trait TypedHyperAST<'store, TIdN: TypedNodeId<IdN = Self::IdN>>: HyperAST<'store> {
    type TT: TypedTree<
        Type = TIdN::Ty,
        TreeId = TIdN::IdN,
        Label = Self::Label,
        ChildIdx = <<Self as HyperAST<'store>>::T as WithChildren>::ChildIdx,
    >;
    type TNS: 'store + TypedNodeStore<TIdN, R<'store> = Self::TT>;
    fn typed_node_store(&self) -> &Self::TNS;
}

pub struct SimpleHyperAST<T, TS, NS, LS> {
    pub type_store: TS,
    pub node_store: NS,
    pub label_store: LS,
    pub _phantom: std::marker::PhantomData<T>,
}

impl<T, TS, NS: Copy, LS> SimpleHyperAST<T, TS, NS, &LS> {
    pub fn change_type_store_ref<TS2>(&self, new: TS2) -> SimpleHyperAST<T, TS2, NS, &LS> {
        SimpleHyperAST {
            type_store: new,
            node_store: self.node_store,
            label_store: self.label_store,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, TS, NS, LS> SimpleHyperAST<T, TS, NS, LS> {
    pub fn change_type_store<TS2>(self, new: TS2) -> SimpleHyperAST<T, TS2, NS, LS> {
        SimpleHyperAST {
            type_store: new,
            node_store: self.node_store,
            label_store: self.label_store,
            _phantom: std::marker::PhantomData,
        }
    }
    pub fn change_tree_type<T2>(self) -> SimpleHyperAST<T2, TS, NS, LS> {
        SimpleHyperAST {
            type_store: self.type_store,
            node_store: self.node_store,
            label_store: self.label_store,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, TS: Default, NS: Default, LS: Default> Default for SimpleHyperAST<T, TS, NS, LS> {
    fn default() -> Self {
        Self {
            type_store: Default::default(),
            node_store: Default::default(),
            label_store: Default::default(),
            _phantom: Default::default(),
        }
    }
}

impl<T, TS, NS, LS> NodeStore<T::TreeId> for SimpleHyperAST<T, TS, NS, LS>
where
    T: Tree,
    T::TreeId: NodeId<IdN = T::TreeId>,
    NS: NodeStore<T::TreeId>,
{
    type R<'a> = NS::R<'a>
    where
        Self: 'a;

    fn resolve(&self, id: &T::TreeId) -> Self::R<'_> {
        self.node_store.resolve(id)
    }
}

impl<T, TS, NS, LS> NodeStoreLean<T::TreeId> for SimpleHyperAST<T, TS, NS, LS>
where
    T: Tree,
    T::TreeId: NodeId<IdN = T::TreeId>,
    NS: NodeStoreLean<T::TreeId>,
{
    type R = NS::R;

    fn resolve(&self, id: &T::TreeId) -> Self::R {
        self.node_store.resolve(id)
    }
}

impl<'store, T, TS, NS, LS> LabelStore<str> for SimpleHyperAST<T, TS, NS, LS>
where
    T: Tree,
    T::TreeId: NodeId<IdN = T::TreeId>,
    LS: LabelStore<str, I = T::Label>,
    <T as Labeled>::Label: Copy,
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

impl<'store, T, TS, NS, LS> TypeStore<T> for SimpleHyperAST<T, TS, NS, LS>
where
    T: TypedTree,
    T::TreeId: NodeId<IdN = T::TreeId>,
    T::Type: 'static + std::hash::Hash,
    TS: TypeStore<T, Ty = T::Type>,
{
    type Ty = TS::Ty;

    const MASK: u16 = TS::MASK;

    fn resolve_type(&self, n: &T) -> Self::Ty {
        self.type_store.resolve_type(n)
    }

    fn resolve_lang(&self, n: &T) -> LangWrapper<Self::Ty> {
        self.type_store.resolve_lang(n)
    }

    type Marshaled = TS::Marshaled;

    fn marshal_type(&self, n: &T) -> Self::Marshaled {
        self.type_store.marshal_type(n)
    }
    fn type_eq(&self, n: &T, m: &T) -> bool {
        self.type_store.type_eq(n, m)
    }

    fn type_to_u16(&self, t: Self::Ty) -> u16 {
        self.type_store.type_to_u16(t)
    }
}

pub struct TypeIndex {
    pub lang: &'static str,
    pub ty: u16,
}

impl<'store, T, TS, NS, LS> HyperAST<'store> for SimpleHyperAST<T, TS, NS, LS>
where
    T: Tree,
    T::TreeId: NodeId<IdN = T::TreeId>,
    TS: TypeStore<T>,
    NS: 'store + NodeStore<T::TreeId, R<'store> = T>,
    LS: LabelStore<str, I = T::Label>,
{
    type IdN = T::TreeId;

    type Idx = T::ChildIdx;

    type Label = T::Label;

    type T = T;

    type NS = NS;

    fn node_store(&self) -> &Self::NS {
        &self.node_store
    }

    type LS = LS;

    fn label_store(&self) -> &Self::LS {
        &self.label_store
    }

    type TS = TS;

    fn type_store(&self) -> &Self::TS {
        &self.type_store
    }
}

impl<'store, T, TS, NS, LS> HyperASTShared for SimpleHyperAST<T, TS, NS, LS>
where
    T: Tree,
{
    type IdN = T::TreeId;

    type Idx = T::ChildIdx;

    type Label = T::Label;
}
impl<T, TS, NS, LS> HyperASTAsso for SimpleHyperAST<T, TS, NS, LS>
where
    T: Tree,
    T::TreeId: NodeId,
    TS: TypeStore<T>,
    for<'s> NS: 's + NodeStore<T::TreeId, R<'s> = T>,
    LS: LabelStore<str, I = T::Label>,
{
    type T<'s> = T where Self:'s;

    type NS<'s> = NS where Self:'s;

    fn node_store(&self) -> &Self::NS<'_> {
        &self.node_store
    }

    type LS = LS;

    fn label_store(&self) -> &Self::LS {
        &self.label_store
    }

    type TS<'s> = TS where Self:'s;

    fn type_store(&self) -> &Self::TS<'_> {
        &self.type_store
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AnyType(&'static dyn HyperType);

unsafe impl Send for AnyType {}
unsafe impl Sync for AnyType {}
impl PartialEq for AnyType {
    fn eq(&self, other: &Self) -> bool {
        self.generic_eq(other.0)
    }
}
// impl Default for AnyType {}
impl Eq for AnyType {}
impl Hash for AnyType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_shared().hash(state);
    }
}
impl Display for AnyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}
impl From<&'static dyn HyperType> for AnyType {
    fn from(value: &'static dyn HyperType) -> Self {
        Self(value)
    }
}

impl HyperType for AnyType {
    fn generic_eq(&self, other: &dyn HyperType) -> bool
    where
        Self: 'static + PartialEq + Sized,
    {
        // elegant solution leveraging the static nature of node types
        std::ptr::eq(self.as_static(), other.as_static())
    }

    fn is_file(&self) -> bool {
        self.0.is_file()
    }

    fn is_directory(&self) -> bool {
        self.0.is_directory()
    }

    fn is_spaces(&self) -> bool {
        self.0.is_spaces()
    }

    fn is_syntax(&self) -> bool {
        self.0.is_syntax()
    }

    fn as_shared(&self) -> Shared {
        self.0.as_shared()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self.0.as_any()
    }

    fn as_static(&self) -> &'static dyn HyperType {
        self.0.as_static()
    }

    fn as_static_str(&self) -> &'static str {
        self.0.as_static_str()
    }

    fn is_hidden(&self) -> bool {
        self.0.is_hidden()
    }

    fn is_supertype(&self) -> bool {
        self.0.is_supertype()
    }

    fn is_named(&self) -> bool {
        self.0.is_named()
    }

    fn get_lang(&self) -> LangWrapper<Self>
    where
        Self: Sized,
    {
        // self.0.get_lang()
        // NOTE quite surprising Oo
        // the type inference is working in our favour
        // TODO post on https://users.rust-lang.org/t/understanding-trait-object-safety-return-types/73425 or https://stackoverflow.com/questions/54465400/why-does-returning-self-in-trait-work-but-returning-optionself-requires or https://www.reddit.com/r/rust/comments/lbbobv/3_things_to_try_when_you_cant_make_a_trait_object/
        self.0.lang_ref()
    }

    fn lang_ref(&self) -> LangWrapper<AnyType> {
        self.0.lang_ref()
    }
}
