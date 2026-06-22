## Part 1 — Tutorial: Build Your First Page From Zero

This builds one tiny page from nothing, step by step, so you see the whole process once before doing it on Fictreon.

### Step 1 — Make a new project
```bash
cargo new sprout_tutorial
cd sprout_tutorial
```

### Step 2 — Add dependencies
Open `Cargo.toml`, add:
```toml
[dependencies]
axum = "0.8"
maud = "0.27"
tokio = { version = "1", features = ["full"] }
sprout_ui_core = { git = "https://github.com/YOUR_USERNAME/sprout", package = "sprout_ui_core" }
sprout_ui_tags = { git = "https://github.com/YOUR_USERNAME/sprout", package = "sprout_ui_tags" }
```

### Step 3 — Write the smallest possible page
Open `src/main.rs`, replace everything with:
```rust
use sprout_ui_tags as tag;

fn main() {
    let page = tag::h1().child("Hello, Sprout");
    println!("{}", page.build().into_string());
}
```

Run it:
```bash
cargo run
```
You should see printed:
```
<h1>Hello, Sprout</h1>
```
That's the whole loop in miniature: build a tree (`tag::h1()...`), turn it into HTML (`.build().into_string()`).

### Step 4 — Add structure
Replace the `let page = ...` line with:
```rust
let page = tag::div()
    .class("card")
    .child(tag::h1().child("Hello, Sprout"))
    .child(tag::p().child("This is your first real page."));
```
Run again. You'll see one nested block of HTML printed — this is exactly how every bigger page gets built, just with more pieces.

### Step 5 — Serve it over a real browser instead of printing
Replace the whole file:
```rust
use axum::{response::Html, routing::get, Router};
use sprout_ui_tags as tag;

async fn homepage() -> Html<String> {
    let page = tag::div()
        .class("card")
        .child(tag::h1().child("Hello, Sprout"))
        .child(tag::p().child("This is your first real page."));

    Html(page.build().into_string())
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(homepage));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Running at http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
```
Run `cargo run`, open `http://127.0.0.1:3000` in a browser. You'll see your card rendered as a real webpage.

### Step 6 — Add a list of dynamic items
Replace the `homepage` function:
```rust
async fn homepage() -> Html<String> {
    let fruits = vec!["Apple", "Banana", "Cherry"];

    let page = tag::div()
        .class("card")
        .child(tag::h1().child("My Fruits"))
        .child(
            tag::ul().children(
                fruits.iter().map(|f| tag::li().child(f.to_string())).collect::<Vec<_>>()
            )
        );

    Html(page.build().into_string())
}
```
Refresh the browser — you'll see a real list built from a `Vec`, which is exactly the pattern you'll use for blog posts later.

That's the entire workflow, end to end. Everything bigger you build is the same five ingredients: `tag::`, `.class()`/`.attr()`, `.child()`/`.children()`, `.build()`, send it through Axum.

---

## Part 2 — Full File-by-File Explanation: "Where Do I Touch This?"

This is your map for when you want to change something later and don't remember which file controls it.

### `sprout_ui_core/src/lib.rs`

**What it is:** the engine room. Every other file depends on this one. You'll touch this file when you want to change *how rendering itself works* — not what a specific page looks like.

| If you want to... | Touch this part |
|---|---|
| Add a new shortcut method (like `.src()` or `.hx_post()`) | The `impl_attr_methods!` macro block — add a new line there, it applies to both `Element` and `VoidElement` automatically. |
| Change how escaping works | `escape_text()` and `escape_attr()` functions near the bottom. |
| Change what counts as a "container" vs "void" element at the type level | `Element` struct and `VoidElement` struct definitions. |
| Fix a rendering bug (wrong output HTML) | `impl Render for Element` and `impl Render for VoidElement` — these write the actual `<tag attr="...">` strings. |
| Add a test | Inside `#[cfg(test)] mod tests { ... }` at the very bottom. |

**You will rarely need to touch this file day-to-day.** It's the foundation — once it works, you build pages by using it, not by editing it.

### `sprout_ui_tags/src/lib.rs`

**What it is:** the list of every HTML tag name sprout knows about, split into container tags and void tags.

| If you want to... | Touch this part |
|---|---|
| Add support for an HTML tag that's missing | Add its name to either the `container:` or `void:` list inside `declare_tags!`. Pick container if the tag can have children, void if it can't. |
| Check if a tag exists | Just search this file for the tag name — if it's not here, it's not available as `tag::whatever()`. |

**This is a small, low-risk file.** Adding a tag is one word in one list.

### `sprout_ui_components/src/lib.rs`

**What it is:** your actual reusable building blocks — `navbar()`, `card()`, and anything else you build for Fictreon specifically (House cards, Castle banners, etc., later).

| If you want to... | Touch this part |
|---|---|
| Change how the navbar looks everywhere | Edit `navbar()` here — every page using it updates automatically. |
| Add a new reusable piece (a "Sovereign rank badge," say) | Add a new `pub fn` here, same pattern as `card()`. |
| Fix a bug in one specific component | Find that component's function by name, fix the `tag::` chain inside it. |

**This is the file you'll touch most often** as Fictreon grows — every new visual piece you reuse across pages goes here.

### `Cargo.toml` (each crate has one)

| If you want to... | Touch this part |
|---|---|
| Point at a new version of sprout on GitHub | The `git = "..."` lines. |
| Add a new dependency (a new crate) | Add a new line under `[dependencies]`. |

---

## Quick Mental Map

```
Want to change a specific page's content?      → wherever you call ui::blog_page() etc. (your feature files)
Want to change a reusable piece used everywhere? → sprout_ui_components
Want a new HTML tag available?                  → sprout_ui_tags
Want to change how rendering/escaping works?     → sprout_ui_core
```

---
