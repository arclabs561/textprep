use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn sample_text() -> &'static str {
    // Mixed script + whitespace + diacritics + emoji: exercises unicode paths.
    "  François  Müller\tmet  習近平  in  北京.\r\nThen   they  said:  🎉🎊  "
}

fn bench_collapse_whitespace(c: &mut Criterion) {
    let mut group = c.benchmark_group("unicode::collapse_whitespace");
    let input = sample_text();
    group.bench_function("baseline", |b| {
        b.iter(|| textprep::unicode::collapse_whitespace(black_box(input)))
    });
    group.finish();
}

fn bench_tokenize_with_offsets(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenize::tokenize_with_offsets");
    let input = sample_text();
    group.bench_function("baseline", |b| {
        b.iter(|| textprep::tokenize::tokenize_with_offsets(black_box(input)))
    });
    group.finish();
}

fn bench_trigram_similarity(c: &mut Criterion) {
    let mut group = c.benchmark_group("similarity");
    let a = "François Müller";
    let b = "Francois Muller";

    group.bench_function("trigram_jaccard", |bencher| {
        bencher.iter(|| textprep::similarity::trigram_jaccard(black_box(a), black_box(b)))
    });

    for n in [2usize, 3, 4, 5] {
        group.bench_with_input(BenchmarkId::new("char_ngram_jaccard", n), &n, |bencher, &n| {
            bencher.iter(|| textprep::similarity::char_ngram_jaccard(black_box(a), black_box(b), n))
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_collapse_whitespace,
    bench_tokenize_with_offsets,
    bench_trigram_similarity
);
criterion_main!(benches);

