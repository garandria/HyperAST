use std::path::Path;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

mod shared;
use shared::*;

pub const QUERIES: &[(&[&str], &str, &str, &str, usize)] = &[
    (
        &[QUERY_OVERRIDES_SUBS[1]],
        QUERY_OVERRIDES.0,
        QUERY_OVERRIDES.1,
        "overrides",
        1,
    ),
    (
        &[QUERY_OVERRIDES_SUBS[1]],
        QUERY_OVERRIDES.0,
        QUERY_OVERRIDES.1,
        "overrides",
        2,
    ),
    (
        &[QUERY_MAIN_METH_SUBS[0]],
        QUERY_MAIN_METH.0,
        QUERY_MAIN_METH.1,
        "main_meth",
        1,
    ),
    (
        &[QUERY_MAIN_METH_SUBS[0]],
        QUERY_MAIN_METH.0,
        QUERY_MAIN_METH.1,
        "main_meth",
        2,
    ),
    (
        &[QUERY_MAIN_METH_SUBS[0]],
        QUERY_MAIN_METH.0,
        QUERY_MAIN_METH.1,
        "main_meth",
        3,
    ),
    // (
    //     &[QUERY_MAIN_METH_SUBS[1]],
    //     QUERY_MAIN_METH.0,
    //     QUERY_MAIN_METH.1,
    //     "main_meth",
    //     4,
    // ),
];

fn compare_querying_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("QueryingRepeatSpoon");
    group.sample_size(10);

    let codes = "../../../../spoon/src/main/java";
    let codes = Path::new(&codes).to_owned();
    let codes = It::new(codes).map(|x| {
        let text = std::fs::read_to_string(&x).expect(&format!(
            "{:?} is not a java file or a dir containing java files: ",
            x
        ));
        (x, text)
    });
    let codes: Box<[_]> = codes.collect();
    // let queries: Vec<_> = QUERIES.iter().enumerate().collect();

    for p in QUERIES.into_iter().map(|x| (x, codes.as_ref())) {
        group.throughput(Throughput::Elements(p.0 .4 as u64));

        group.bench_with_input(
            BenchmarkId::new(format!("baseline-{}", p.0 .3), p.0 .4),
            &p,
            |b, (q, f)| {
                b.iter(|| {
                    for _ in 0..p.0 .4 {
                        for p in f.into_iter() {
                            let (q, t, text) = prep_baseline(q.2)(p);
                            let mut cursor = tree_sitter::QueryCursor::default();
                            black_box(cursor.matches(&q, t.root_node(), text.as_bytes()).count());
                        }
                    }
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new(format!("sharing_default-{}", p.0 .3), p.0 .4),
            &p,
            |b, (q, f)| {
                b.iter(|| {
                    let query =
                        hyper_ast_tsquery::Query::new(q.1, tree_sitter_java::language()).unwrap();
                    let mut stores = hyper_ast::store::SimpleStores::<hyper_ast_gen_ts_java::types::TStore>::default();
                    let mut md_cache = Default::default();
                    let mut java_tree_gen =
                        hyper_ast_gen_ts_java::legion_with_refs::JavaTreeGen::new(
                            &mut stores,
                            &mut md_cache,
                        );
                    let roots: Vec<_> = f
                        .into_iter()
                        .map(|(name, text)| {
                            let tree =
                                match hyper_ast_gen_ts_java::legion_with_refs::tree_sitter_parse(
                                    text.as_bytes(),
                                ) {
                                    Ok(t) => t,
                                    Err(t) => t,
                                };
                            let full_node = java_tree_gen.generate_file(
                                name.to_str().unwrap().as_bytes(),
                                text.as_bytes(),
                                tree.walk(),
                            );
                            full_node.local.compressed_node
                        })
                        .collect();
                    for _ in 0..p.0 .4 {
                        for &n in &roots {
                            let pos = hyper_ast::position::StructuralPosition::new(n);
                            let cursor = hyper_ast_tsquery::hyperast::TreeCursor::new(&stores, pos);
                            let matches = query.matches(cursor);
                            black_box(matches.count());
                        }
                    }
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new(format!("sharing_precomputed-{}", p.0 .3), p.0 .4),
            &p,
            |b, (q, f)| {
                b.iter(|| {
                    let (precomp, query) = hyper_ast_tsquery::Query::with_precomputed(
                        q.1,
                        tree_sitter_java::language(),
                        q.0,
                    )
                    .unwrap();
                    let mut stores = hyper_ast::store::SimpleStores::<hyper_ast_gen_ts_java::types::TStore>::default();
                    let mut md_cache = Default::default();
                    let mut java_tree_gen = hyper_ast_gen_ts_java::legion_with_refs::JavaTreeGen {
                        line_break: "\n".as_bytes().to_vec(),
                        stores: &mut stores,
                        md_cache: &mut md_cache,
                        more: precomp,
                    };
                    let roots: Vec<_> = f
                        .into_iter()
                        .map(|(name, text)| {
                            let tree =
                                match hyper_ast_gen_ts_java::legion_with_refs::tree_sitter_parse(
                                    text.as_bytes(),
                                ) {
                                    Ok(t) => t,
                                    Err(t) => t,
                                };
                            let full_node = java_tree_gen._generate_file(
                                name.to_str().unwrap().as_bytes(),
                                text.as_bytes(),
                                tree.walk(),
                            );
                            full_node.local.compressed_node
                        })
                        .collect();
                    for _ in 0..p.0 .4 {
                        for &n in &roots {
                            let pos = hyper_ast::position::StructuralPosition::new(n);
                            let cursor = hyper_ast_tsquery::hyperast::TreeCursor::new(&stores, pos);
                            let matches = query.matches(cursor);
                            black_box(matches.count());
                        }
                    }
                })
            },
        );
    }
    group.finish()
}

criterion_group!(querying, compare_querying_group);
criterion_main!(querying);
