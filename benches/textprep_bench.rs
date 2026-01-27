use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;

fn sample_text() -> &'static str {
    // Mixed script + whitespace + diacritics + emoji: exercises unicode paths.
    "  François  Müller\tmet  習近平  in  北京.\r\nThen   they  said:  🎉🎊  "
}

fn bench_collapse_whitespace(c: &mut Criterion) {
    let mut group = c.benchmark_group("unicode::collapse_whitespace");
    let input_small = sample_text();
    let input_med = input_small.repeat(10);
    let input_large = input_small.repeat(100);

    for (label, input) in [
        ("small", input_small),
        ("med", input_med.as_str()),
        ("large", input_large.as_str()),
    ] {
        group.bench_with_input(BenchmarkId::new("alloc", label), &input, |b, &input| {
            b.iter(|| textprep::unicode::collapse_whitespace(black_box(input)))
        });

        let mut out = String::new();
        group.bench_with_input(
            BenchmarkId::new("into_reuse", label),
            &input,
            |b, &input| {
                b.iter(|| {
                    textprep::unicode::collapse_whitespace_into(black_box(input), &mut out);
                    black_box(out.len())
                })
            },
        );
    }
    group.finish();
}

fn bench_unicode_transforms(c: &mut Criterion) {
    let mut group = c.benchmark_group("unicode::transforms");

    let input_small = sample_text();
    let input_med = input_small.repeat(10);
    let input_large = input_small.repeat(100);

    for (label, input) in [
        ("small", input_small),
        ("med", input_med.as_str()),
        ("large", input_large.as_str()),
    ] {
        group.bench_with_input(
            BenchmarkId::new("normalize_newlines_alloc", label),
            &input,
            |b, &input| b.iter(|| textprep::unicode::normalize_newlines(black_box(input))),
        );

        let mut out = String::new();
        group.bench_with_input(
            BenchmarkId::new("normalize_newlines_into_reuse", label),
            &input,
            |b, &input| {
                b.iter(|| {
                    textprep::unicode::normalize_newlines_into(black_box(input), &mut out);
                    black_box(out.len())
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("remove_zero_width_alloc", label),
            &input,
            |b, &input| b.iter(|| textprep::unicode::remove_zero_width(black_box(input))),
        );

        let mut out = String::new();
        group.bench_with_input(
            BenchmarkId::new("remove_zero_width_into_reuse", label),
            &input,
            |b, &input| {
                b.iter(|| {
                    textprep::unicode::remove_zero_width_into(black_box(input), &mut out);
                    black_box(out.len())
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("remove_bidi_controls_alloc", label),
            &input,
            |b, &input| b.iter(|| textprep::unicode::remove_bidi_controls(black_box(input))),
        );

        let mut out = String::new();
        group.bench_with_input(
            BenchmarkId::new("remove_bidi_controls_into_reuse", label),
            &input,
            |b, &input| {
                b.iter(|| {
                    textprep::unicode::remove_bidi_controls_into(black_box(input), &mut out);
                    black_box(out.len())
                })
            },
        );
    }

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
        group.bench_with_input(
            BenchmarkId::new("char_ngram_jaccard", n),
            &n,
            |bencher, &n| {
                bencher.iter(|| {
                    textprep::similarity::char_ngram_jaccard(black_box(a), black_box(b), n)
                })
            },
        );
    }

    group.finish();
}

fn bench_flashtext(c: &mut Criterion) {
    let mut group = c.benchmark_group("flash::FlashText::find");
    let input_small = sample_text();
    let input_med = input_small.repeat(10);
    let input_large = input_small.repeat(100);

    // Use the same keyword set across sizes.
    fn build_ft() -> textprep::FlashText {
        let mut ft = textprep::FlashText::new();
        ft.add_keyword("François", "francois");
        ft.add_keyword("Müller", "muller");
        ft.add_keyword("北京", "beijing");
        ft.add_keyword("🎉", "party");
        ft
    }

    for (label, input) in [
        ("small", input_small),
        ("med", input_med.as_str()),
        ("large", input_large.as_str()),
    ] {
        let mut ft = build_ft();
        group.bench_with_input(BenchmarkId::new("alloc_vec", label), &input, |b, &input| {
            b.iter(|| ft.find(black_box(input)))
        });

        let mut ft = build_ft();
        let mut out = Vec::new();
        group.bench_with_input(
            BenchmarkId::new("into_reuse_vec", label),
            &input,
            |b, &input| {
                b.iter(|| {
                    ft.find_into(black_box(input), &mut out);
                    black_box(out.len())
                })
            },
        );
    }

    group.finish();
}

fn bench_scrub(c: &mut Criterion) {
    let mut group = c.benchmark_group("scrub::scrub_with");
    let input_small = sample_text();
    let input_med = input_small.repeat(10);
    let input_large = input_small.repeat(100);

    let cfg_search = textprep::ScrubConfig::search_key();
    let cfg_strict = textprep::ScrubConfig::search_key_strict_invisibles();

    for (label, input) in [
        ("small", input_small),
        ("med", input_med.as_str()),
        ("large", input_large.as_str()),
    ] {
        group.bench_with_input(
            BenchmarkId::new("search_key", label),
            &input,
            |b, &input| b.iter(|| textprep::scrub_with(black_box(input), black_box(&cfg_search))),
        );

        group.bench_with_input(
            BenchmarkId::new("search_key_strict", label),
            &input,
            |b, &input| b.iter(|| textprep::scrub_with(black_box(input), black_box(&cfg_strict))),
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_collapse_whitespace,
    bench_unicode_transforms,
    bench_tokenize_with_offsets,
    bench_trigram_similarity,
    bench_flashtext,
    bench_scrub
);
criterion_main!(benches);
