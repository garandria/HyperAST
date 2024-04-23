use crate::tests::cpp_tree;
use hyper_ast::{position::TreePath, store::defaults::NodeIdentifier, types::Typed};

const Q0: &str =
    r#"(binary_expression (_expression (number_literal)) "+" (_expression (number_literal)))"#; // TODO make _expression optional
const C0: &str = r#"int f() {
    return 21 + 21;
}"#;

const C1: &str = r#"int f() {
    return 21 - 21;
}"#;

const C2: &str = r#"int f() {
    int a = 21;
    return a + a;
}"#;
const Q1: &str = r#"(binary_expression (_expression (identifier) @first) "+" (_expression (identifier) @second)) (#eq? @first @second)"#; // TODO make _expression optional

// Possible useful stuff:
// - test if subtree is conforming to ts query
//   - initially for each node in subtree, do the test
//     - terminate on wrong root type as fast as possible
//   - after that find different oracles
//     - type oracle
//     - structure hash oracle
//     - filtered structure hash oracle
//     - other convolutions (including prev hashes)
//     - labels through bags of words and defered bloom filters computing
// - edit distance between query and subtree
// - acceleration related to extracting entropy from basic constructs

#[test]
fn simple() {
    let (code_store, code) = cpp_tree(C0.as_bytes());
    let (query_store, query) = crate::search::ts_query(Q0.as_bytes());
    let path = hyper_ast::position::StructuralPosition::new(code);
    let prepared_matcher =
        crate::search::PreparedMatcher::<_, hyper_ast_gen_ts_cpp::types::Type>::new(
            &query_store,
            query,
        );
    let mut matched = false;
    for e in hyper_ast_gen_ts_cpp::iter::IterAll::new(&code_store, path, code) {
        if prepared_matcher.is_matching::<_, hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>(
            &code_store,
            *e.node().unwrap(),
        ) {
            type T = hyper_ast_gen_ts_cpp::types::TIdN<hyper_ast::store::defaults::NodeIdentifier>;
            let n = code_store
                .node_store
                .try_resolve_typed::<T>(e.node().unwrap())
                .unwrap()
                .0;
            let t = n.get_type();
            dbg!(t);
            matched = true;
        }
    }
    assert!(matched);
    let (code_store1, code1) = cpp_tree(C1.as_bytes());
    let path = hyper_ast::position::StructuralPosition::new(code1);
    let prepared_matcher =
        crate::search::PreparedMatcher::<_, hyper_ast_gen_ts_cpp::types::Type>::new(
            &query_store,
            query,
        );
    for e in hyper_ast_gen_ts_cpp::iter::IterAll::new(&code_store1, path, code1) {
        if prepared_matcher.is_matching::<_, hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>(
            &code_store1,
            *e.node().unwrap(),
        ) {
            type T = hyper_ast_gen_ts_cpp::types::TIdN<hyper_ast::store::defaults::NodeIdentifier>;
            let n = code_store1
                .node_store
                .try_resolve_typed::<T>(e.node().unwrap())
                .unwrap()
                .0;
            let t = n.get_type();
            dbg!(t);
            panic!("should not match")
        }
    }
}

#[test]
fn named() {
    let (code_store, code) = cpp_tree(C2.as_bytes());
    let (query_store, query) = crate::search::ts_query(Q1.as_bytes());
    let path = hyper_ast::position::StructuralPosition::new(code);
    let prepared_matcher =
        crate::search::PreparedMatcher::<_, hyper_ast_gen_ts_cpp::types::Type>::new(
            &query_store,
            query,
        );
    let mut matched = false;
    for e in hyper_ast_gen_ts_cpp::iter::IterAll::new(&code_store, path, code) {
        if let Some(_) = prepared_matcher
            .is_matching_and_capture::<_, hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>(
                &code_store,
                *e.node().unwrap(),
            )
        {
            type T = hyper_ast_gen_ts_cpp::types::TIdN<hyper_ast::store::defaults::NodeIdentifier>;
            let n = code_store
                .node_store
                .try_resolve_typed::<T>(e.node().unwrap())
                .unwrap()
                .0;
            let t = n.get_type();
            dbg!(t);
            matched = true;
        }
    }
    assert!(matched);
}
