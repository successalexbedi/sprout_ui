
## Folder structure
```
src/
├── main.rs
├── core/
│   └── ... (not used here, layout lives inside the feature's templates/ per your structure)
└── features/
    └── home/
        ├── mod.rs
        ├── model.rs
        ├── handler.rs
        ├── routes.rs
        └── templates/
            ├── mod.rs
            ├── layout.rs
            └── nav.rs
```

## src/features/home/mod.rs
```rust
pub mod model;
pub mod handler;
pub mod routes;
pub mod templates;
```

## src/features/home/templates/mod.rs
```rust
pub mod layout;
pub mod nav;
```

## src/features/home/templates/nav.rs
**Job: build the nav bar. Nothing else.**
```rust
use sprout_ui_core::Element;
use sprout_ui_tags as tag;

pub fn nav() -> Element {
    tag::nav()
        .class("site-nav")
        .child(tag::h1().child("Fictreon"))
        .child(
            tag::div().class("nav-links").children(vec![
                tag::a().href("/").child("Home"),
                tag::a().href("/blog").child("Blog"),
            ])
        )
}
```

## src/features/home/templates/layout.rs
**Job: wrap any page's content in the full `<html>` shell, including the nav and HTMX script. Knows nothing about what "home" or "blog" actually contain.**
```rust
use sprout_ui_core::Element;
use sprout_ui_tags as tag;
use super::nav::nav;

pub fn layout(title: &str, content: Element) -> Element {
    tag::html()
        .child(
            tag::head()
                .child(tag::title().child(title.to_string()))
                .child(tag::link().rel_stylesheet("/static/style.css"))
                .child(tag::script().src("https://unpkg.com/htmx.org@1.9.10"))
        )
        .child(
            tag::body()
                .child(nav())
                .child(content)
        )
}
```

Quick note: `.rel_stylesheet()` doesn't exist yet in core — use `.attr("rel", "stylesheet").src(...)` instead, since `<link>` doesn't use `src`, it uses `href`:

```rust
.child(
    tag::link().attr("rel", "stylesheet").href("/static/style.css")
)
```

So the corrected `layout.rs`:
```rust
use sprout_ui_core::Element;
use sprout_ui_tags as tag;
use super::nav::nav;

pub fn layout(title: &str, content: Element) -> Element {
    tag::html()
        .child(
            tag::head()
                .child(tag::title().child(title.to_string()))
                .child(tag::link().attr("rel", "stylesheet").href("/static/style.css"))
                .child(tag::script().src("https://unpkg.com/htmx.org@1.9.10"))
        )
        .child(
            tag::body()
                .child(nav())
                .child(content)
        )
}
```

## src/features/home/model.rs
**Job: the only file allowed to know where homepage data comes from.**
```rust
// For now, this is just a static welcome message — but this is where
// you'd later pull real data (featured Houses, recent posts, etc.)
// from the database, the same way blog's model.rs talks to SQLite.

pub struct HomeData {
    pub welcome_message: String,
}

pub fn get_home_data() -> HomeData {
    HomeData {
        welcome_message: "Fiction's first label system.".to_string(),
    }
}
```

## src/features/home/handler.rs
**Job: connect model → templates, return HTML. No HTML-building logic lives here directly — it calls out to templates for that.**
```rust
use axum::response::Html;
use sprout_ui_core::Element;
use sprout_ui_tags as tag;

use crate::features::home::{model, templates::layout::layout};

pub async fn index() -> Html<String> {
    let data = model::get_home_data();
    let content = home_content(&data.welcome_message);
    let page = layout("Fictreon — Home", content);
    Html(page.build().into_string())
}

fn home_content(welcome_message: &str) -> Element {
    tag::main()
        .class("home-content")
        .child(tag::h1().child("Welcome to Fictreon"))
        .child(tag::p().child(welcome_message.to_string()))
}
```

## src/features/home/routes.rs
**Job: map a URL path to a handler function. Nothing else.**
```rust
use axum::{routing::get, Router};
use crate::features::home::handler;

pub fn home_routes() -> Router<()> {
    Router::new().route("/", get(handler::index))
}
```

## src/main.rs
```rust
use axum::Router;

mod features;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .merge(features::home::routes::home_routes());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Fictreon running at http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
```

## src/features/mod.rs
```rust
pub mod home;
```

---

## The mental model, in one line per file

- **`model.rs`** — knows where data comes from. Nothing else touches the database.
- **`handler.rs`** — connects model → templates, hands back HTML. No raw HTML-building.
- **`routes.rs`** — maps URLs to handler functions. No logic.
- **`templates/nav.rs`** — builds the nav bar, reused by `layout.rs`.
- **`templates/layout.rs`** — wraps any page's content in the full shell. Doesn't know what's inside `content`.
- **`templates/mod.rs`** — declares both template files exist.

When you add `blog` next, you reuse `templates/layout.rs` and `templates/nav.rs` as-is (they don't live "inside" home conceptually — worth moving them to a shared `core/templates/` once you have two features, so you're not duplicating the nav across folders). Want to do that move now, before blog, or build blog first and refactor once the duplication is actually in front of you?