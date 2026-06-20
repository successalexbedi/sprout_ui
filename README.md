```markdown
# Sprout

A small Rust builder pattern for writing HTML, built because reading Maud's macro syntax was genuinely hard to follow. Sprout trades some of Maud's compile-time guarantees and raw performance for code that reads top-to-bottom like a sentence.

```rust
tag::div()
    .class("card")
    .child(tag::h1().child("Hello"))
    .child(tag::p().child("Body text"))
```

vs Maud:

```rust
html! {
    div class="card" {
        h1 { "Hello" }
        p { "Body text" }
    }
}
```

If the second one is hard for you to scan, sprout exists for the same reason it would be hard for someone else.

## What's in this repo

- **sprout_ui_core** — the actual `Element` / `VoidElement` / `Node` types and the rendering logic. The only crate that knows how to turn a tree into an HTML string.
- **sprout_ui_tags** — one function per HTML tag (`div()`, `p()`, `img()`, etc.), each returning either an `Element` or a `VoidElement` depending on whether that tag is allowed to have children.
- **sprout_ui_components** — example reusable pieces (`navbar()`, `card()`) built from the above two.

## Install

```toml
[dependencies]
sprout_ui_core = { git = "https://github.com/YOUR_USERNAME/sprout", package = "sprout_ui_core" }
sprout_ui_tags = { git = "https://github.com/YOUR_USERNAME/sprout", package = "sprout_ui_tags" }
sprout_ui_components = { git = "https://github.com/YOUR_USERNAME/sprout", package = "sprout_ui_components" }
```

## Quick example

```rust
use sprout_ui_tags as tag;

fn page() -> String {
    let html = tag::main()
        .child(tag::h1().child("Fictreon"))
        .child(
            tag::div().class("post-list").children(vec![
                tag::div().class("post-card").child(tag::h2().child("First post")),
            ])
        );

    html.build().into_string()
}
```

## What sprout guarantees

- **Output is escaped.** Text content and attribute values are HTML-escaped before they reach the page. A post title containing `<script>` renders as literal text, not executable markup.
- **Void elements can't take children, at compile time.** `tag::br().child("x")` fails to compile — `VoidElement` has no `.child()` method. This was a real bug in an earlier version; it's now structurally impossible.
- **Tag names are checked at compile time.** `tag::dvi()` fails to compile because no such function exists — only tags listed in `sprout_ui_tags` exist as functions at all.
- **Duplicate classes are deduplicated.** Calling `.class("card")` twice doesn't produce `class="card card"`.
- **Misclassifying a tag as both void and container is a compile error.** If a tag is ever accidentally listed in both groups in `sprout_ui_tags`, you get a duplicate-function compile error, not a silent runtime bug.

All of the above is covered by an automated test suite (30 tests as of writing) in `sprout_ui_core`.

## What sprout does NOT guarantee

Being direct about this matters more than sounding impressive:

- **It does not check whether an attribute belongs on a given tag.** `tag::img().attr("href", "...")` compiles fine, even though `<img>` has no `href`. Catching this would require a distinct type per HTML tag with its own hand-written attribute methods — a much larger rewrite that would remove the generic `.attr()` this project is built around.
- **It does not check HTML structural rules.** Nothing stops you from putting a `<tr>` outside a `<table>`. Sprout checks "is this a valid tag" and "can this tag have children," nothing deeper.
- **It allocates more than Maud per render.** Every `Element` allocates a `Vec` for children and a `Vec` for attrs, plus a `Box` for every nested element. For a typical page this is invisible. It would start to matter under heavy traffic with deeply nested pages rendered thousands of times per second — not a problem this project has been benchmarked against yet.
- **It has one maintainer.** No community bug reports, no second pair of eyes on security-relevant code, no guarantee every HTML edge case has been considered.
- **It has no editor tooling.** No syntax highlighting, no autocomplete beyond standard Rust method-chaining.

## Running the tests

```bash
cargo test
```

## Status

Used in production for [Fictreon](https://fictreon.com)'s blog feature. Built to solve one specific problem — Maud's syntax being hard to read — not to be a general-purpose HTML framework competing with Maud on safety or speed.

## License

[Add a license — MIT or Apache-2.0 are the common defaults for Rust crates.]
```

That's the README — what it is, what it's for, what it does and doesn't protect against, stated plainly. Ready for the doc and cheatsheet whenever you want to start those.