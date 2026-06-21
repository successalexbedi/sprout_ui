Sprout

Sprout is a small HTML builder for Rust.

It exists because not everyone finds macro-heavy templating pleasant to read. Sprout trades some compile-time guarantees and raw performance for something simpler: HTML that reads top-to-bottom like ordinary Rust.

Instead of:

html! {
    div class="card" {
        h1 { "Hello" }
        p { "Body text" }
    }
}

Sprout uses method chaining:

tag::div()
    .class("card")
    .child(tag::h1().child("Hello"))
    .child(tag::p().child("Body text"))

Neither approach is objectively better. Sprout exists for people who prefer reading normal Rust over reading another syntax hidden inside macros.

---

Why Sprout exists

Most Rust HTML libraries optimize for compile-time guarantees and zero-cost abstractions.

Sprout optimizes for readability.

The goal is simple:

- Make HTML generation feel like ordinary Rust.
- Avoid introducing a mini language.
- Keep components composable.
- Make dynamic rendering straightforward.
- Stay small.

It is not trying to replace Maud or compete with it on performance.

---

Features

- HTML escaping by default.
- Compile-time distinction between normal and void elements.
- Compile-time checked tag names.
- Duplicate CSS classes are automatically deduplicated.
- Attribute order is preserved.
- Built-in helpers for common attributes.
- HTMX support.
- Alpine.js support.
- Dynamic children from iterators.
- Conditional rendering.
- Reusable components.
- No macros required after tag declaration.

---

Installation

[dependencies]
sprout_ui_core = { git = "https://github.com/successalexbedi/sprout", package = "sprout_ui_core" }
sprout_ui_tags = { git = "https://github.com/successalexbedi/sprout", package = "sprout_ui_tags" }
sprout_ui_components = { git = "https://github.com/successalexbedi/sprout", package = "sprout_ui_components" }

---

Quick example

use sprout_ui_tags as tag;

fn page() -> String {
    tag::main()
        .child(tag::h1().child("Fictreon"))
        .child(
            tag::div()
                .class("posts")
                .child_for(posts, |post| {
                    tag::article()
                        .class("post-card")
                        .child(tag::h2().child(post.title))
                })
        )
        .build()
        .into_string()
}

Because components are just values, building larger pages is simply nesting ordinary Rust structures.

---

Workspace layout

"sprout_ui_core"

Contains:

- "Node"
- "Element"
- "VoidElement"
- attribute handling
- escaping
- rendering

This is the only crate that knows how to turn a tree into HTML.

---

"sprout_ui_tags"

Provides one function per HTML tag:

tag::div()
tag::section()
tag::input()
tag::img()

Container tags return "Element".

Void tags return "VoidElement".

If a tag is accidentally declared as both container and void, the duplicate function causes a compile error instead of creating a silent bug.

---

"sprout_ui_components"

Reusable components built on top of the lower layers.

Examples:

navbar()
card()
sidebar()

Components are just Rust functions returning elements.

---

Guarantees

Sprout guarantees several things.

HTML escaping

Text nodes are escaped automatically:

.child("<script>alert(1)</script>")

renders as:

&lt;script&gt;alert(1)&lt;/script&gt;

rather than executable JavaScript.

Attribute values are escaped separately and correctly.

---

Void elements cannot have children

tag::br().child("oops")

doesn't compile.

"VoidElement" simply has no ".child()" method.

A bug that cannot be expressed is better than a bug waiting to happen.

---

Invalid tag names fail at compile time

tag::dvi()

fails because no such function exists.

Tag names are defined once inside "sprout_ui_tags".

---

Duplicate classes are removed

tag::div()
    .class("card")
    .class("card")

renders:

<div class="card">

not:

<div class="card card">

---

ID assignment is predictable

Multiple ".id()" calls overwrite previous values:

.id("first")
.id("second")

produces:

id="second"

---

Attribute order is preserved

Attributes are rendered in the order they were added.

This makes output deterministic and easier to test.

---

Dynamic rendering is straightforward

Iterators can generate children:

.child_for(posts, |post| {
    tag::li().child(post.title)
})

Conditional rendering:

.child_if(show_admin_panel, || {
    admin_panel()
})

No special syntax is required.

---

HTMX and Alpine support

Sprout includes convenience helpers for common attributes.

HTMX:

.hx_get("/refresh")
.hx_target("#posts")
.hx_swap("beforeend")

Alpine:

.x_data("{ open: false }")
.x_show("open")
.x_on("click", "open = !open")

These are only helper methods.

Underneath, everything still uses the same generic ".attr()" system.

---

Non-goals

Sprout intentionally does not attempt to solve everything.

HTML validation

This compiles:

tag::tr()

outside a table.

Sprout checks whether a tag exists and whether it can have children, but it does not attempt to enforce HTML semantics.

---

Attribute validation

This compiles:

tag::img().attr("href", "/")

even though "<img>" doesn't support "href".

Supporting attribute validation would require separate types for every tag and would remove much of the flexibility Sprout is built around.

---

Zero-allocation rendering

Sprout allocates vectors for children and attributes and boxes nested elements.

For typical pages this is insignificant.

Libraries like Maud will generally be faster and allocate less.

Sprout chooses readability over squeezing every possible allocation out of the render path.

---

Editor tooling

Sprout has no custom syntax highlighting or editor plugins.

Everything is ordinary Rust, so existing tooling works automatically.

---

Testing

The core crate currently contains dozens of tests covering:

- escaping
- attribute rendering
- class deduplication
- HTMX helpers
- Alpine helpers
- iterator rendering
- conditional rendering
- nested structures
- deep trees
- text and element ordering

Run them with:

cargo test

---

Status

Sprout is used in production for Fictreon's blog.

It was built to solve one problem:

making HTML generation feel like ordinary Rust instead of a second language hidden inside macros.

---

License

Licensed under either of

- Apache License, Version 2.0
- MIT License

at your option.

Copyright © 2026 Paul Alex Bedi.

Unless explicitly stated otherwise, any contribution intentionally submitted for inclusion in Sprout shall be dual licensed as above without additional terms or conditions.