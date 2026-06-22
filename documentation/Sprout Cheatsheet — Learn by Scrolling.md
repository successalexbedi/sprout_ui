```markdown
# Sprout Cheatsheet — Learn by Scrolling

No theory. Just: "I want to do X — here's exactly how."

---

## Table of Contents

1. [Setup](#1-setup)
2. [The Basic Pattern](#2-the-basic-pattern)
3. [Text Tags](#3-text-tags)
4. [Containers (div, section, etc.)](#4-containers)
5. [Lists](#5-lists)
6. [Links and Images](#6-links-and-images)
7. [Forms](#7-forms)
8. [Tables](#8-tables)
9. [Classes and IDs](#9-classes-and-ids)
10. [Custom Attributes](#10-custom-attributes)
11. [Inline Styles](#11-inline-styles)
12. [HTMX](#12-htmx)
13. [Alpine.js](#13-alpinejs)
14. [Putting Children Inside Things](#14-putting-children-inside-things)
15. [Lists of Data (Loops)](#15-lists-of-data-loops)
16. [Turning It Into Actual HTML Text](#16-turning-it-into-actual-html-text)
17. [Do's and Don'ts — Quick Reference](#17-dos-and-donts--quick-reference)

---

## 1. Setup

At the top of any file you're writing HTML in:

```rust
use sprout_ui_tags as tag;
```

That's it. Now `tag::div()`, `tag::p()`, `tag::img()` etc. are all available.

---

## 2. The Basic Pattern

Every piece of HTML follows the same shape:

```rust
tag::SOMETHING()
    .SOME_SETTING("value")
    .child("what goes inside")
```

Example:
```rust
tag::p().child("Hello world")
```
Produces:
```html
<p>Hello world</p>
```

---

## 3. Text Tags

| You want | You write | You get |
|---|---|---|
| Paragraph | `tag::p().child("text")` | `<p>text</p>` |
| Heading 1 | `tag::h1().child("Title")` | `<h1>Title</h1>` |
| Heading 2 | `tag::h2().child("Subtitle")` | `<h2>Subtitle</h2>` |
| Bold | `tag::strong().child("important")` | `<strong>important</strong>` |
| Italic | `tag::em().child("emphasis")` | `<em>emphasis</em>` |
| Inline span | `tag::span().child("inline text")` | `<span>inline text</span>` |
| Line break | `tag::br()` | `<br>` |

```rust
tag::h1().child("Fictreon Blog")
// <h1>Fictreon Blog</h1>
```

---

## 4. Containers

The "boxes" that hold other things.

| You want | You write |
|---|---|
| Generic box | `tag::div().child(...)` |
| Page section | `tag::section().child(...)` |
| Navigation bar | `tag::nav().child(...)` |
| Main content area | `tag::main().child(...)` |
| Page header | `tag::header().child(...)` |
| Page footer | `tag::footer().child(...)` |

```rust
tag::div()
    .class("card")
    .child(tag::h2().child("Title"))
    .child(tag::p().child("Body text"))
```
Produces:
```html
<div class="card"><h2>Title</h2><p>Body text</p></div>
```

---

## 4.5 Inventing Your Own Tags (Custom Elements)

`sprout_ui_tags` only has functions for standard HTML tags. If you need something that isn't standard HTML — a custom element, a web component, something a JS framework defines like `<my-widget>` or `<ion-button>` — there's no `tag::my_widget()` waiting for you, because sprout has no way to know about a tag it's never heard of.

For that, drop down to `Element::new()` directly and give it whatever tag name you want as a string:

```rust
use sprout_ui_core::Element;

Element::new("my-widget")
    .attr("data-id", "42")
    .child("Custom content")
```
Produces:
```html
<my-widget data-id="42">Custom content</my-widget>
```

It behaves exactly like any `tag::` element from here on — `.class()`, `.child()`, `.children()`, `.attr()`, all the same methods work, because `tag::div()` is just `Element::new("div")` with a shorter name. You're using the exact same constructor, just supplying the tag name yourself instead of letting a pre-written function supply it for you.

**Real example — a custom web component for a like-counter widget:**
```rust
Element::new("like-counter")
    .attr("count", "12")
    .attr("post-id", "458")
```
Produces:
```html
<like-counter count="12" post-id="458"></like-counter>
```

**Do** reach for `Element::new("...")` when the tag genuinely doesn't exist in the standard list.
**Don't** use it for ordinary tags that already have a `tag::` function — `Element::new("div")` works, but `tag::div()` is shorter, and only `tag::div()` catches a typo at compile time. If you typo `Element::new("dvi")`, nothing stops you — it'll just silently render `<dvi>`, an invalid tag, with zero warning. That protection only exists on the `tag::` side.

---

## 5. Lists

```rust
tag::ul().children(vec![
    tag::li().child("First item"),
    tag::li().child("Second item"),
    tag::li().child("Third item"),
])
```
Produces:
```html
<ul><li>First item</li><li>Second item</li><li>Third item</li></ul>
```

Numbered list — same idea, just `ol` instead of `ul`:
```rust
tag::ol().children(vec![
    tag::li().child("Step one"),
    tag::li().child("Step two"),
])
```

---

## 6. Links and Images

**Link:**
```rust
tag::a().href("/blog").child("Go to blog")
// <a href="/blog">Go to blog</a>
```

**Image:**
```rust
tag::img().src("/cat.png").alt("A cat")
// <img src="/cat.png" alt="A cat">
```

Notice `img` has no `.child()` — images can't contain anything. That's on purpose, not a missing feature.

---

## 7. Forms

```rust
tag::form().children(vec![
    tag::input().type_("text").name("title").placeholder("Title here"),
    tag::button().type_("submit").child("Submit"),
])
```
Produces:
```html
<form>
  <input type="text" name="title" placeholder="Title here">
  <button type="submit">Submit</button>
</form>
```

**Textarea:**
```rust
tag::textarea().name("body").placeholder("Write something...")
```

**Label:**
```rust
tag::label().child("Email:")
```

---

## 8. Tables

```rust
tag::table().children(vec![
    tag::tr().children(vec![
        tag::th().child("Name"),
        tag::th().child("Age"),
    ]),
    tag::tr().children(vec![
        tag::td().child("Alex"),
        tag::td().child("30"),
    ]),
])
```
Produces:
```html
<table>
  <tr><th>Name</th><th>Age</th></tr>
  <tr><td>Alex</td><td>30</td></tr>
</table>
```

---

## 9. Classes and IDs

**One class:**
```rust
tag::div().class("card")
// <div class="card">
```

**Multiple classes — just chain `.class()` again:**
```rust
tag::div().class("card").class("featured")
// <div class="card featured">
```

**An ID:**
```rust
tag::div().id("post-list")
// <div id="post-list">
```

⚠️ Calling `.id()` twice doesn't add two ids — the second call replaces the first:
```rust
tag::div().id("first").id("second")
// <div id="second">    <-- "first" is gone
```

---

## 10. Custom Attributes

For anything without a dedicated shortcut method, use `.attr(key, value)`:

```rust
tag::div().attr("data-user-id", "123")
// <div data-user-id="123">
```

---

## 11. Inline Styles

```rust
tag::div().style("color: red; font-weight: bold;")
// <div style="color: red; font-weight: bold;">
```

---

## 12. HTMX

```rust
tag::form()
    .hx_post("/blog/posts")
    .hx_target("#post-list")
    .hx_swap("beforeend")
```
Produces:
```html
<form hx-post="/blog/posts" hx-target="#post-list" hx-swap="beforeend">
```

| Method | Produces |
|---|---|
| `.hx_get(url)` | `hx-get="url"` |
| `.hx_post(url)` | `hx-post="url"` |
| `.hx_target(selector)` | `hx-target="selector"` |
| `.hx_swap(mode)` | `hx-swap="mode"` |
| `.hx_trigger(event)` | `hx-trigger="event"` |

---

## 13. Alpine.js

```rust
tag::div()
    .x_data("{ open: false }")
    .child(tag::button().x_on("click", "open = !open").child("Toggle"))
    .child(tag::div().x_show("open").child("Surprise!"))
```
Produces:
```html
<div x-data="{ open: false }">
  <button x-on:click="open = !open">Toggle</button>
  <div x-show="open">Surprise!</div>
</div>
```

| Method | Produces |
|---|---|
| `.x_data(expr)` | `x-data="expr"` |
| `.x_show(expr)` | `x-show="expr"` |
| `.x_model(expr)` | `x-model="expr"` |
| `.x_text(expr)` | `x-text="expr"` |
| `.x_on(event, expr)` | `x-on:event="expr"` |
| `.x_bind(attr, expr)` | `x-bind:attr="expr"` |
| `.x_transition()` | `x-transition=""` |

---

## 14. Putting Children Inside Things

**One child:**
```rust
tag::div().child("just text")
tag::div().child(tag::p().child("a whole element"))
```

**Many children at once:**
```rust
tag::div().children(vec![
    tag::p().child("First"),
    tag::p().child("Second"),
])
```

**Mixing text and elements:**
```rust
tag::p()
    .child("Hello, ")
    .child(tag::strong().child("world"))
    .child("!")
// <p>Hello, <strong>world</strong>!</p>
```

---

## 15. Lists of Data (Loops)

When you have real data — like blog posts from a database — and want one element per item:

```rust
let posts: Vec<Post> = get_posts_from_database();

tag::div().class("post-list").children(
    posts.iter().map(|post| {
        tag::div()
            .class("post-card")
            .child(tag::h2().child(post.title.clone()))
            .child(tag::p().child(post.body.clone()))
    }).collect::<Vec<_>>()
)
```

This is the pattern you'll use constantly — `.children()` plus `.map()` over real data.

---

 ## 15.5. Loops and Conditions — Without Writing `.map().collect()` Every Time
 
```rust 
**One element per item in a list:**
```rust
tag::ul().child_for(&posts, |p| tag::li().child(p.title.clone()))
```
Same as 
```rust
```.children(posts.iter().map(...).collect::<Vec<_>>())`, just shorter.
```


if else
```rust
**Only show something if a condition is true:**
```rust
tag::div().child_if(user.owns_post, || tag::button().child("Edit"))
```
If the condition is `false`, nothing is added — no `if/else` needed.

⚠️ Note the `||` before the element in `.child_if()` — that's required, not decoration. It means the element only gets built if the condition is actually true.


`.build()` turns the tree into HTML. `.into_string()` turns that into a plain `String` you can send to a browser.

---

## 17. Do's and Don'ts — Quick Reference

### ✅ DO chain `.class()` for multiple classes
```rust
tag::div().class("card").class("featured")
```

### ❌ DON'T put multiple classes in one `.class()` call expecting separation
```rust
tag::div().class("card featured") // works, but it's ONE string with a space in it, not two checked classes
```
(This actually still renders fine since classes get joined with a space anyway — but it bypasses the duplicate-class protection, since sprout only checks exact matches.)

---

### ✅ DO use `.child()` on container tags
```rust
tag::div().child("text")
```

### ❌ DON'T try `.child()` on void tags — it won't compile
```rust
tag::img().child("oops") // compile error — img has no .child()
```
Void tags (`img`, `br`, `input`, `hr`, etc.) physically cannot contain anything in real HTML. The compiler is reminding you, not getting in your way.

---

### ✅ DO put real user input straight into `.child()` or `.attr()`
```rust
tag::p().child(post.title.clone()) // safe — gets escaped automatically
```

### ❌ DON'T manually escape it yourself first
```rust
tag::p().child(escape_html(post.title.clone())) // unnecessary — sprout already does this
```

---

### ✅ DO use `tag::div()` for everyday HTML
```rust
tag::div().class("card")
```

### ❌ DON'T reach for `Element::new()` unless the tag isn't in the standard list
```rust
Element::new("div").class("card") // works, but tag::div() is shorter and typo-checked
```
Save `Element::new("...")` for genuinely custom tags like `<my-widget>`.

---

### ✅ DO set an id once
```rust
tag::div().id("post-list")
```

### ❌ DON'T expect calling `.id()` twice to keep both
```rust
tag::div().id("a").id("b") // renders id="b" only — "a" is overwritten, not kept
```

---

### ✅ DO use `.children(vec)` or `.children(iter.map(...))` for dynamic lists
```rust
tag::ul().children(posts.iter().map(|p| tag::li().child(p.title.clone())).collect::<Vec<_>>())
```

### ❌ DON'T call `.child()` in a loop expecting it to work the same way
```rust
let mut list = tag::ul();
for post in posts {
    list = list.child(tag::li().child(post.title)); // works, but awkward — .children() is built for exactly this
}
```

---

### ✅ DO finish every page/component with `.build().into_string()`
```rust
let html = page.build().into_string();
```

### ❌ DON'T forget `.build()` and try to send the `Element` directly
```rust
Html(page) // won't compile — Element isn't a String, .build() is required first
```


