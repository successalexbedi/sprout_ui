use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sprout_ui_tags as tag;

pub const CARDS_PER_SECTION: usize = 1000;

pub fn build_page(id: usize) -> String {
    let navbar = tag::div()
        .class("navbar")
        .child(tag::a().href("/").child("Home"))
        .child(tag::a().href("/docs").child("Docs"))
        .child(tag::a().href("/github").child("GitHub"));

    let hero = tag::section()
        .class("hero")
        .child(tag::h1().child(format!("SproutUI Page {}", id)))
        .child(tag::p().child("Build fast UIs with zero noise"));

    let make_section = |title: &str| {
        let mut section = tag::section()
            .class("content")
            .child(tag::h2().child(title));

        for i in 0..CARDS_PER_SECTION {
            section = section.child(
                tag::div()
                    .class("card featured")
                    .child(
                        tag::h3().child(format!("Card {}-{}", id, i))
                    )
                    .child(
                        tag::p().child("This is a realistic card body")
                    ),
            );
        }

        section
    };

    tag::div()
        .class("layout")
        .child(navbar)
        .child(hero)
        .child(make_section("Features"))
        .child(make_section("Pricing"))
        .child(make_section("Testimonials"))
        .build()
        .into_string()
}

fn bench(c: &mut Criterion) {
    c.bench_function("SproutUI Full Page", |b| {
        b.iter(|| black_box(build_page(42)))
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);