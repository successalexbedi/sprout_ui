use sprout_ui_tags as tag;
use std::hint::black_box;
use std::time::Instant;

const PAGES: usize = 1000;
const WARMUP: usize = 100;
const CARDS: usize = 140;
const DEPTH: usize = 4;
const TEXT_BLOCKS: usize = 8;

const DEPTH_CLASSES: &[&str] = &["depth-0", "depth-1", "depth-2", "depth-3", "depth-4"];
const DEPTH_STRS: &[&str] = &["0", "1", "2", "3", "4"];

// Shared helper
#[inline(always)]
fn attr_toggle(id: usize, i: usize) -> Option<&'static str> {
    if (id + i) % 3 == 0 { Some("active") } else { None }
}

mod sprout {
    use super::*;
    use sprout_ui_core::{sprout_fmt, SproutStr, Element};

    #[inline(always)]
    fn make_deep_card(id: usize, i: usize, title: SproutStr) -> Element {
        let mut current = tag::div().class(DEPTH_CLASSES[0]);
        for d in 1..DEPTH {
            current = tag::div()
                .class(DEPTH_CLASSES[d])
                .attr("data-d", DEPTH_STRS[d])
                .child(current)
                .child(tag::span().child(sprout_fmt!("d:{} c:{}-{}", d, id, i)));
        }

        tag::div()
            .class("card")
            .attr("data-id", sprout_fmt!("{}-{}", id, i))
            .attr("data-active", attr_toggle(id, i).unwrap_or(""))
            .child(tag::h3().child(title))
            .child(current)
    }

    pub fn build_full_page(id: usize, title: String) -> String {
        let nav_items = [("home", "/home"), ("docs", "/docs"), ("api", "/api"), ("bench", "/bench"), ("login", "/login")];

        let nav = tag::div().class("navbar").children(
            nav_items.iter().map(|&(label, href)| {
                tag::a()
                    .href(href)
                    .attr("aria-current", if label == "home" { "page" } else { "" })
                    .child(label)
            })
        );

        let hero = tag::section()
            .class("hero")
            .child(tag::h1().child(SproutStr::from(title)))
            .child(tag::p().child("stress benchmark mode"))
            .child(tag::p().child(sprout_fmt!("seed-{} nonce-{}", id % 97, black_box(id % 13))));

        let cards: Vec<_> = (0..CARDS)
            .map(|i| {
                let card_title = sprout_fmt!("Card {}-{}", id, i);
                let mut c = make_deep_card(id, i, card_title);
                if i % 5 == 0 { c = tag::div().class("featured").child(c); }
                if i % 7 == 0 { c = tag::div().child(c).child(tag::p().child("pinned content block")); }
                c
            })
            .collect();

        let content = tag::section()
            .class("content")
            .child(tag::h2().child("features"))
            .children(cards);

        let texts: Vec<_> = (0..TEXT_BLOCKS)
            .map(|i| {
                let mut p = tag::p();
                if i % 2 == 0 { p = p.class("alt"); }
                p.child(sprout_fmt!("lorem stress {}-{} jitter simulation data stream", id, black_box(i)))
            })
            .collect();

        let text_zone = tag::section().class("text").children(texts);

        let footer = tag::footer()
            .class("footer")
            .child(tag::p().child("sprout ui stress suite"))
            .child(tag::a().href("#").child("privacy"));

        tag::div()
            .class("layout")
            .id(sprout_fmt!("app-{}", id))
            .child(nav)
            .child(hero)
            .child(content)
            .child(text_zone)
            .child(footer)
            .build()
            .into_string()
    }
}

mod maud_bench {
    use super::*;
    use maud::{html, PreEscaped};

    #[inline(always)]
    fn make_deep_card(id: usize, i: usize, title: String) -> PreEscaped<String> {
        let mut current = html! { div class=(DEPTH_CLASSES[0]) {} };
        for d in 1..DEPTH {
            current = html! {
                div class=(DEPTH_CLASSES[d]) data-d=(DEPTH_STRS[d]) {
                    (current)
                    span { "d:" (d) " c:" (id) "-" (i) }
                }
            };
        }
        html! {
            div class="card" data-id=(format!("{}-{}", id, i)) data-active=[attr_toggle(id, i)] {
                h3 { (title) }
                (current)
            }
        }
    }

    pub fn build_full_page(id: usize, title: String) -> String {
        let nav_items = [("home", "/home"), ("docs", "/docs"), ("api", "/api"), ("bench", "/bench"), ("login", "/login")];

        let nav = html! {
            div class="navbar" {
                @for &(label, href) in &nav_items {
                    a href=(href) aria-current=[if label == "home" { Some("page") } else { None }] { (label) }
                }
            }
        };

        let hero = html! {
            section class="hero" {
                h1 { (title) }
                p { "stress benchmark mode" }
                p { "seed-" (id % 97) " nonce-" (black_box(id % 13)) }
            }
        };

        let cards = (0..CARDS).map(|i| {
            let card_title = format!("Card {}-{}", id, i);
            let mut c = make_deep_card(id, i, card_title);
            if i % 5 == 0 { c = html! { div class="featured" { (c) } }; }
            if i % 7 == 0 { c = html! { div { (c) p { "pinned content block" } } }; }
            c
        });

        let content = html! {
            section class="content" {
                h2 { "features" }
                @for card in cards { (card) }
            }
        };

        let texts = (0..TEXT_BLOCKS).map(|i| {
            html! {
                p class=[if i % 2 == 0 { Some("alt") } else { None }] {
                    "lorem stress " (id) "-" (black_box(i)) " jitter simulation data stream"
                }
            }
        });

        let text_zone = html! { section class="text" { @for text in texts { (text) } } };
        let footer = html! { footer class="footer" { p { "maud stress suite" } a href="#" { "privacy" } } };

        html! {
            div class="layout" id=(format!("app-{}", id)) { (nav) (hero) (content) (text_zone) (footer) }
        }.into_string()
    }
}

// =====================================================================
// BENCHMARKING HARNESS
// =====================================================================
struct TestData { titles: Vec<String> }
impl TestData { fn new(count: usize) -> Self { Self { titles: (0..count).map(|i| format!("Page {}", i)).collect() } } }

struct BenchResult {
    name: &'static str,
    total_time: f64,
    bytes: usize,
    times: Vec<f64>,
}

fn run_bench<F>(name: &'static str, data: &TestData, build: F) -> BenchResult
where F: Fn(usize, String) -> String
{
    for i in 0..WARMUP { let _ = build(i, data.titles[i].clone()); }
    let mut times = Vec::with_capacity(PAGES);
    let mut total_bytes = 0usize;
    let start = Instant::now();
    for i in 0..PAGES {
        let title = data.titles[i].clone();
        let iter_start = Instant::now();
        let html = build(i, title);
        times.push(iter_start.elapsed().as_secs_f64());
        total_bytes += html.len();
        black_box(html);
    }
    BenchResult { name, total_time: start.elapsed().as_secs_f64(), bytes: total_bytes, times }
}

fn print_metrics(r: &BenchResult) {
    let mut t = r.times.clone();
    t.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mean = r.times.iter().sum::<f64>() / PAGES as f64;
    let p99 = t[(PAGES as f64 * 0.99) as usize];
    println!("\n>>> {} RESULTS\nThroughput: {:.0} pages/sec\nMean: {:.2} µs\nP99: {:.2} µs", r.name, PAGES as f64 / r.total_time, mean * 1e6, p99 * 1e6);
}

fn main() {
    let data = TestData::new(PAGES);
    let sprout = run_bench("Sprout UI", &data, sprout::build_full_page);
    let maud = run_bench("Maud", &data, maud_bench::build_full_page);
    print_metrics(&sprout);
    print_metrics(&maud);
}
