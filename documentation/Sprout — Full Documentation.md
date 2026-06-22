
```markdown
# Sprout — Full Documentation

A builder-pattern HTML library for Rust. Built because Maud's macro syntax was hard to read, and method-chaining reads like plain English instead.

---

## Table of Contents

1. [Philosophy](#1-philosophy)
2. [Installation](#2-installation)
3. [Core Concepts](#3-core-concepts)
   - 3.1 [`Element`](#31-element)
   - 3.2 [`VoidElement`](#32-voidelement)
   - 3.3 [`Node`](#33-node)
   - 3.4 [`Attrs`](#34-attrs)
4. [Two Ways to Build a Tag — and Which One to Use](#4-two-ways-to-build-a-tag--and-which-one-to-use)
5. [Tags Reference](#5-tags-reference)
   - 5.1 [Container tags](#51-container-tags)
   - 5.2 [Void tags](#52-void-tags)
6. [Building Pages](#6-building-pages)
7. [Components — What They Are and How to Build Your Own](#7-components--what-they-are-and-how-to-build-your-own)
8. [Frontend Behavior: HTMX and Alpine.js](#8-frontend-behavior-htmx-and-alpinejs)
9. [Inline Styles](#9-inline-styles)
10. [Escaping and Safety](#10-escaping-and-safety)
11. [Dynamic Attribute Keys and Why `Cow` Matters](#11-dynamic-attribute-keys-and-why-cow-matters)
12. [Wiring Into Axum](#12-wiring-into-axum)
13. [Testing](#13-testing)
14. [Known Limitations](#14-known-limitations)
15. [FAQ](#15-faq)

---

## 1. Philosophy

Sprout exists for one reason: Maud's `html! { }` macro was hard to read. Sprout trades some of Maud's compile-time HTML-structure checking and a bit of raw performance for code that reads as a chain of plain Rust method calls.

```rust
// Sprout
tag::div().class("card").child(tag::p().child("Hello"))

// Maud
html! { div class="card" { p { "Hello" } } }
```

Sprout isn't trying to replace Maud for everyone — it's solving a specific readability problem for a specific person, and it does that honestly, without pretending to be something it's not.

---

## 2. Installation

```toml
[dependencies]
sprout_ui_core = { git = "https://github.com/YOUR_USERNAME/sprout", package = "sprout_ui_core" }
sprout_ui_tags = { git = "https://github.com/YOUR_USERNAME/sprout", package = "sprout_ui_tags" }
sprout_ui_components = { git = "https://github.com/YOUR_USERNAME/sprout", package = "sprout_ui_components" }
maud = "0.27"
```

---

## 3. Core Concepts

Everything lives in `sprout_ui_core`.

### 3.1 `Element`

Any tag allowed to have children — `<div>`, `<p>`, `<button>`, `<form>`, etc.

```rust
pub struct Element {
    pub tag: &'static str,
    pub attrs: Attrs,
    pub children: Vec<Node>,
}
```

| Method | What it does |
|---|---|
| `Element::new(tag)` | Low-level constructor. See [Section 4](#4-two-ways-to-build-a-tag--and-which-one-to-use). |
| `.class(name)` | Adds a CSS class. Calling it twice with the same name does not duplicate it. |
| `.id(name)` | Sets the `id` attribute. Calling it again overwrites the previous value — last call wins. |
| `.attr(key, value)` | Sets any attribute by name. Accepts both `&'static str` and owned `String` keys. |
| `.src(url)` `.href(url)` `.alt(text)` `.name(n)` `.value(v)` `.placeholder(p)` `.type_(t)` | Shortcuts for common HTML attributes. |
| `.style(css)` | Sets the `style` attribute directly. |
| `.hx_get(url)` `.hx_post(url)` `.hx_target(sel)` `.hx_swap(mode)` `.hx_trigger(t)` | HTMX attribute shortcuts. |
| `.x_data(expr)` `.x_show(expr)` `.x_model(expr)` `.x_text(expr)` `.x_on(event, expr)` `.x_bind(attr, expr)` `.x_transition()` | Alpine.js attribute shortcuts. |
| `.child(node)` | Adds one child. Accepts a string, an `Element`, or a `VoidElement`. |
| `.children(iter)` | Adds many children from any iterator. |
| `.build()` | Converts the tree into `maud::Markup`. |

### 3.2 `VoidElement`

Any tag that **cannot** have children — `<br>`, `<img>`, `<input>`, `<hr>`, etc. Same attribute methods as `Element`, minus `.child()` and `.children()` — those methods simply don't exist on this type.

```rust
tag::br().child("oops"); // does not compile — VoidElement has no .child()
```

This is verified by a `compile_fail` doc-test in `sprout_ui_core` — it's not just a design intention, it's something the test suite actively proves stays true.

### 3.3 `Node`

What lives inside an `Element`'s `children` list:

```rust
pub enum Node {
    Element(Box<Element>),
    Void(Box<VoidElement>),
    Text(String),
}
```

You rarely build a `Node` directly — `.child()`/`.children()` convert automatically:

| You pass in | Becomes |
|---|---|
| `&str` / `String` | `Node::Text` |
| an `Element` | `Node::Element` |
| a `VoidElement` | `Node::Void` |

### 3.4 `Attrs`

Shared storage used by both `Element` and `VoidElement`, so attribute logic is written once:

```rust
pub struct Attrs {
    pub classes: Vec<String>,
    pub id: Option<String>,
    pub pairs: Vec<(Cow<'static, str>, String)>,
}
```

The `Cow<'static, str>` key type is covered in [Section 11](#11-dynamic-attribute-keys-and-why-cow-matters).

---

## 4. Two Ways to Build a Tag — and Which One to Use

You'll see two patterns in this codebase:

```rust
Element::new("div").class("home")   // low-level
tag::div().class("home")            // everyday
```

These aren't two equal styles to pick between — `tag::div()` is sugar built directly on top of `Element::new`:

```rust
// inside sprout_ui_tags
pub fn div() -> Element { Element::new(stringify!(div)) }
```

**Use `tag::whatever()` for everything in the standard HTML tag list** (see Section 5). It's shorter, it's checked by the compiler (a typo'd tag name fails to compile), and it's the syntax this whole project exists to give you.

**Reach for `Element::new("...")` directly only when you need a tag that isn't in `sprout_ui_tags`** — a custom element or web component like `<my-widget>`. It's the escape hatch, not the everyday tool.

---

## 5. Tags Reference

```rust
use sprout_ui_tags as tag;
```

### 5.1 Container tags

Return `Element`, support `.child()` / `.children()`:

```
div, section, nav, main, header, footer, aside, article, address, details, summary, dialog,
h1, h2, h3, h4, h5, h6, p, span, a, strong, em, small, blockquote, pre, code, kbd, sub, sup, mark, time, del, ins,
ul, ol, li, dl, dt, dd,
form, label, textarea, select, option, optgroup, button, fieldset, legend, output, progress, meter,
table, thead, tbody, tfoot, tr, th, td, caption, colgroup,
video, audio, iframe, canvas, picture, map, object,
html, head, body, title, style, script, noscript
```

### 5.2 Void tags

Return `VoidElement`, no `.child()` / `.children()`:

```
br, hr, img, input, link, meta, area, base, col, embed, param, source, track, wbr
```

If a tag is ever accidentally listed in both groups, that's a compile error (duplicate function name) — not a silent bug.

---

## 6. Building Pages

```rust
use sprout_ui_tags as tag;

fn blog_page() -> String {
    let page = tag::main()
        .child(tag::h1().child("Fictreon Blog"))
        .child(
            tag::div().class("post-list").children(vec![
                post_card("First post", "Some body text"),
                post_card("Second post", "More body text"),
            ])
        );

    page.build().into_string()
}

fn post_card(title: &str, body: &str) -> sprout_ui_core::Element {
    tag::div()
        .class("post-card")
        .child(tag::h2().child(title.to_string()))
        .child(tag::p().child(body.to_string()))
}
```

`.build()` is the only place rendering actually happens. Everything before it is just plain Rust values sitting in memory.

---

## 7. Components — What They Are and How to Build Your Own

A component is a regular Rust function returning `Element` (or rarely `VoidElement`). No macros, no traits, no special system.

```rust
// sprout_ui_components/src/lib.rs

use sprout_ui_core::Element;
use sprout_ui_tags as tag;

pub fn navbar() -> Element {
    tag::nav()
        .class("navbar")
        .child(tag::h1().child("SproutUI"))
}

pub fn card(title: &str, body: &str) -> Element {
    tag::div()
        .class("card")
        .child(tag::h1().child(title))
        .child(tag::p().child(body))
        .child(
            tag::button()
                .class("btn-primary")
                .hx_post("/like")
                .hx_target("#likes")
                .child("Like")
        )
}
```

### Building your own

1. Decide what data it needs — function parameters.
2. Return `Element`.
3. Build the tree with `tag::` and `.child()`/`.children()`.
4. Call it wherever you'd otherwise repeat the same markup.

```rust
pub fn badge(label: &str) -> Element {
    tag::span().class("badge").child(label.to_string())
}
```

A component is indistinguishable from a raw tag once built — both are just `Element` values, so they compose identically.

---

## 8. Frontend Behavior: HTMX and Alpine.js

Neither is a special case internally — both are just attributes, and `.attr()` already handles any key/value pair. The shortcut methods exist purely to make common ones typo-resistant.

### 8.1 HTMX

| Method | Renders |
|---|---|
| `.hx_get(url)` | `hx-get="url"` |
| `.hx_post(url)` | `hx-post="url"` |
| `.hx_target(selector)` | `hx-target="selector"` |
| `.hx_swap(mode)` | `hx-swap="mode"` |
| `.hx_trigger(trigger)` | `hx-trigger="trigger"` |

```rust
tag::form()
    .hx_post("/blog/posts")
    .hx_target("#post-list")
    .hx_swap("beforeend")
```

Load the script yourself — sprout doesn't add it automatically:

```rust
tag::script().attr("src", "https://unpkg.com/htmx.org@1.9.10")
```

### 8.2 Alpine.js

| Method | Renders |
|---|---|
| `.x_data(expr)` | `x-data="expr"` |
| `.x_show(expr)` | `x-show="expr"` |
| `.x_model(expr)` | `x-model="expr"` |
| `.x_text(expr)` | `x-text="expr"` |
| `.x_on(event, expr)` | `x-on:event="expr"` |
| `.x_bind(attr, expr)` | `x-bind:attr="expr"` |
| `.x_transition()` | `x-transition=""` |

```rust
tag::div()
    .x_data("{ open: false }")
    .child(tag::button().x_on("click", "open = !open").child("Toggle"))
    .child(tag::div().x_show("open").x_transition().child("Now you see me"))
```

`.x_on()` and `.x_bind()` build their attribute *key* at call time (`x-on:click` vs `x-on:submit` are different keys), unlike the rest, which use fixed key strings. See [Section 11](#11-dynamic-attribute-keys-and-why-cow-matters) for how that's done without leaking memory.

### 8.3 Combining both

```rust
tag::div()
    .x_data("{ liked: false }")
    .hx_post("/like")
    .hx_target("#likes")
    .x_on("click", "liked = !liked")
    .child("❤")
```

Both render through the same escaping path as any other attribute — values with quotes or braces (`x-data="{ count: 0, label: 'hi' }"`) render safely without breaking the surrounding markup.

---

## 9. Inline Styles

```rust
tag::div().style("color: red; font-weight: bold;")
```

This sets the raw `style` attribute. Sprout doesn't parse or validate CSS — it's a plain string, escaped the same way any other attribute value is.

---

## 10. Escaping and Safety

Text content and attribute values are escaped automatically before reaching the output string.

```rust
tag::p().child("<script>alert(1)</script>").build().into_string()
// → "<p>&lt;script&gt;alert(1)&lt;/script&gt;</p>"
```

Text escaping and attribute escaping are *not* identical — this is intentional:

| Character | Escaped in text? | Escaped in attributes? |
|---|---|---|
| `&` | Yes | Yes |
| `<` | Yes | Yes |
| `>` | Yes | Yes |
| `"` | No | Yes |

Quotes only matter inside an attribute value (where they could break out of the `"..."` boundary) — between tags, a literal `"` is harmless, so it's left alone. This means it's safe to put real user input — a blog post title, a comment — directly into `.child()` or `.attr()` without manually escaping it first.

**What this does not protect against:** sprout doesn't check whether an attribute makes sense on a given tag. `tag::img().href("...")` compiles fine even though `<img>` has no `href`. It only guarantees a value can't break out of its position to inject new markup — it doesn't guarantee the markup is semantically correct.

---

## 11. Dynamic Attribute Keys and Why `Cow` Matters

Most attribute methods use a fixed key — `"hx-post"` is always `"hx-post"`. But `.x_on(event, ...)` needs a *different* key depending on the event name (`x-on:click` vs `x-on:submit`), built at call time with `format!()`.

An earlier version of this code used `Box::leak` to turn that runtime `String` into a `&'static str`, which technically worked but permanently leaked memory on every single call — that memory is never freed for the life of the program. Under real traffic, with `.x_on()` called repeatedly across many requests, that's an actual slow leak.

The fix: `Attrs` stores keys as `Cow<'static, str>` instead of a plain `&'static str`:

```rust
pub pairs: Vec<(Cow<'static, str>, String)>,
```

`Cow` can hold either a borrowed `&'static str` (free, like `"hx-post"`) or an owned `String` (normal heap allocation, freed normally when the `Element` is dropped). `.attr()` now accepts `impl Into<Cow<'static, str>>`, so both of these work without any leak:

```rust
.attr("hx-post", url)                  // borrowed, zero-cost
.attr(format!("x-on:{event}"), expr)   // owned, freed normally
```

Nothing about existing static-string call sites changed — they still work exactly as before.

---

## 12. Wiring Into Axum

```rust
use axum::response::Html;
use sprout_ui_tags as tag;

async fn handler() -> Html<String> {
    let page = tag::main().child(tag::h1().child("Hello"));
    Html(page.build().into_string())
}
```

`.build()` returns `maud::Markup`; `.into_string()` gives a plain `String`, which `Html<String>` wraps with the correct `Content-Type` header.

---

## 13. Testing

```bash
cargo test
```

Covers, across `sprout_ui_core`:
- Tag rendering, nesting, ordering of mixed text/element children
- Class deduplication, id overwrite behavior
- Void-element self-closing and child rejection (via `compile_fail` doc-test)
- Text vs attribute escaping, including quotes, angle brackets, and combined "evil string" cases
- HTMX and Alpine sugar methods individually and combined on one element
- Dynamic/owned attribute keys via `Cow`
- Edge cases: empty string attributes, deeply nested trees, mixed iterator children

No new test is required when adding a tag to `sprout_ui_tags` — tag functions are pure wiring; the rendering logic they call into is already covered regardless of tag name.

---

## 14. Known Limitations

Stated plainly:

- **No per-tag attribute validation.** `tag::img().href("...")` compiles even though `<img>` has no `href`. Fixing this needs a distinct type per tag, which would remove the generic `.attr()` this project is built around.
- **No HTML structural validation.** Nothing stops nesting tags in semantically invalid ways (a `<tr>` outside a `<table>`).
- **More allocation than Maud per render.** Each `Element` allocates `Vec`s for children/attrs plus a `Box` per nested child. Negligible at normal page sizes; worth profiling before high-traffic use with deep nesting.
- **Single maintainer, no community review yet.**

---

## 15. Conditional Rendering 

### Why these exist

Two patterns show up constantly once you're building real pages with real data:

1. "Make one element per item in a list" — every blog post, every list item, every table row.
2. "Only show this if some condition is true" — an edit button only for the post's owner, an error message only when validation fails, an admin panel only for admins.

Without these two methods, you'd write both by hand every single time:

```rust
// Pattern 1, the long way
tag::ul().children(
    posts.iter().map(|p| tag::li().child(p.title.clone())).collect::<Vec<_>>()
)

// Pattern 2, the long way
let mut el = tag::div();
if user.owns_post {
    el = el.child(tag::button().child("Edit"));
}
```

Both work fine — nothing is broken about writing it this way. `.child_for()` and `.child_if()` exist purely to remove the repeated boilerplate (`.map().collect::<Vec<_>>()`, and the `let mut` + reassignment dance) around two patterns you'll hit dozens of times across Fictreon, without introducing any new concept you don't already understand.

### `.child_for()` — one element per item

**Signature:**
```rust
pub fn child_for<I, T, F, N>(mut self, items: I, f: F) -> Self
where
    I: IntoIterator<Item = T>,
    F: Fn(T) -> N,
    N: Into<Node>,
```

**In plain words:** "Give me anything iterable (`items`), and a function that turns one item into one piece of HTML (`f`). I'll run that function over every item and add each result as a child, in order."

**Before and after, side by side:**
```rust
// Before
tag::ul().children(
    posts.iter().map(|p| tag::li().child(p.title.clone())).collect::<Vec<_>>()
)

// After — same result, less ceremony
tag::ul().child_for(&posts, |p| tag::li().child(p.title.clone()))
```

**What it does NOT do:** it doesn't filter, sort, or transform your data — it's purely "one item in, one element out, repeated." If you need filtering, do that on the `Vec` before passing it in (`posts.iter().filter(...)`), then hand the already-filtered result to `.child_for()`.

### `.child_if()` — conditional rendering

**Signature:**
```rust
pub fn child_if<F, N>(self, condition: bool, f: F) -> Self
where
    F: FnOnce() -> N,
    N: Into<Node>,
```

**In plain words:** "If `condition` is true, run `f` and add what it builds as a child. If it's false, do nothing — just hand back the element unchanged."

**Before and after:**
```rust
// Before
let mut el = tag::div();
if user.owns_post {
    el = el.child(tag::button().child("Edit"));
}

// After
tag::div().child_if(user.owns_post, || tag::button().child("Edit"))
```

**Why it takes a closure (`|| ...`) instead of just the built element directly:** if you wrote `.child_if(condition, tag::button().child("Edit"))`, that button would get *built* every single time, even when `condition` is false and you're about to throw it away. Wrapping it in `|| ...` means the closure only runs — only actually builds that button — if `condition` is true. Small detail, but it's the difference between "sometimes do unnecessary work" and "only ever do the work you need."

### When NOT to reach for these

If you only have one or two children and no loop or condition involved, plain `.child()` is still simpler and clearer:

```rust
tag::div().child(tag::h1().child("Title")) // no need for child_for or child_if here
```

These two methods solve the repetition that shows up specifically with **lists of data** and **conditional UI** — not a replacement for `.child()` everywhere.

---

## 16. FAQ

**Why not just use Maud?**
Maud's macro syntax was hard to read for the person who built this. Sprout trades some compile-time HTML checking for code that reads as plain method chains.

**Can I mix sprout and Maud?**
Yes — `.build()` returns real `maud::Markup`, so it splices into a Maud `html!{}` block, or vice versa.

**Why two ways to make a tag — `Element::new()` and `tag::div()`?**
`tag::div()` is sugar over `Element::new("div")`. Use `tag::` for everyday tags; reach for `Element::new` only for custom/non-standard tag names not in the list.

**Why does `VoidElement` exist separately from `Element`?**
Earlier, one shared type let you call `.child()` on `<br>`, and the child silently vanished at render. Splitting the types turns that into a compile error.

**Is it safe with real user input?**
Yes for injection — text and attributes are escaped automatically. No for tag-attribute correctness — it won't stop `href` on an `<img>`, just stop that value from breaking the markup.



