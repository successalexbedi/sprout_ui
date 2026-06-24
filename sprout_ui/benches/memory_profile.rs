use sprout_ui_tags as tag;
use std::hint::black_box;

#[global_allocator]
static ALLOCATOR: dhat::Alloc = dhat::Alloc;

fn build_full_page(id: usize) -> String {
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
        let mut section = tag::section().class("content");
        // Fix: .to_string() so it owns the data
        section = section.child(tag::h2().child(title.to_string()));

        for i in 0..500 {
            let card = tag::div()
                .class("card featured")
                .child(tag::h3().child(black_box(format!("Card {}-{}", id, i))))
                .child(tag::p().child(black_box(
                    "This is a real card with text content to simulate actual usage",
                )));
            section = section.child(card);
        }
        section
    };

    let footer = tag::footer()
        .class("footer")
        .child(tag::p().child("© 2026 SproutUI"))
        .child(tag::a().href("#").child("Privacy"));

    let page = tag::div()
        .class("layout")
        .id(black_box(format!("app-{}", id)))
        .child(navbar)
        .child(hero)
        .child(make_section("Features"))
        .child(make_section("Pricing"))
        .child(make_section("Testimonials"))
        .child(footer);

    page.build().into_string()
}

fn main() {
    let _profiler = dhat::Profiler::new_heap();

    // 1000 full pages ≈ 460 elements each = 460k nodes total
    for i in 0..100000 {
        let html = build_full_page(black_box(i));
        black_box(html);
    }
}
