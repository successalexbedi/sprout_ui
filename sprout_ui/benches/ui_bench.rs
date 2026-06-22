use criterion::{black_box, criterion_group, criterion_main, Criterion};
use maud::html;
use sprout_ui_tags as tag;
use sprout_ui_components as ui;

fn bench_sprout_ui(c: &mut Criterion) {
    c.bench_function("build and serialize card tree", |b| {
        b.iter(|| {
            let tree = tag::div()
                .class("layout")
                .child(ui::card(black_box("Title"), black_box("Content")));
            
            let html = html! { (tree) }.into_string();
            black_box(html);
        })
    });
}

// Explicitly define the group and main
criterion_group!(benches, bench_sprout_ui);
criterion_main!(benches);
