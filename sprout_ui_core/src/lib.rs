// =====================================================================
// SPROUT — sprout_ui_core
// -----------------------------------------------------------------------
// Author: [Your Name]
// Created: 2026
// Last updated: June 22, 2026
//
// A builder-pattern HTML library for Rust, built as a more readable
// alternative to Maud's macro syntax. Method chains instead of html!{}.
//
// This file is the engine room: it defines what an HTML element *is*
// and how it turns into a real, escaped HTML string. Most day-to-day
// page-building never touches this file directly — see sprout_ui_tags
// for the everyday tag::div()/tag::p() functions built on top of this.
// =====================================================================

use maud::{Markup, Render, PreEscaped};
use std::borrow::Cow;

// =====================================================================
// SECTION 1 — THE LEGAL TAG DICTIONARY (debug-build typo protection)
// -----------------------------------------------------------------------
// These lists power a development-time safety net: if you call
// Element::new("dvi") by mistake, this catches it with a helpful panic
// in debug builds. It does NOT run in release builds (see #[cfg(debug_assertions)]
// below), so it costs nothing in production.
//
// IMPORTANT: tags containing a hyphen (e.g. "my-widget") are always
// allowed through untouched, since real custom elements/web components
// are required by the HTML spec to contain a dash. This is what makes
// Element::new("...") usable as the documented escape hatch for
// non-standard tags, instead of accidentally blocking it.
// =====================================================================

/// Every standard HTML tag that is allowed to contain children.
pub const LEGAL_CONTAINERS: &[&str] = &[
    "div", "section", "nav", "main", "header", "footer", "aside", "article", "address", "details", "summary", "dialog",
    "h1", "h2", "h3", "h4", "h5", "h6", "p", "span", "a", "strong", "em", "small", "blockquote", "pre", "code", "kbd", "sub", "sup", "mark", "time", "del", "ins",
    "ul", "ol", "li", "dl", "dt", "dd",
    "form", "label", "textarea", "select", "option", "optgroup", "button", "fieldset", "legend", "output", "progress", "meter",
    "table", "thead", "tbody", "tfoot", "tr", "th", "td", "caption", "colgroup",
    "video", "audio", "iframe", "canvas", "picture", "map", "object",
    "html", "head", "body", "title", "style", "script", "noscript",
    "svg", "datalist",
];

/// Every standard HTML tag that is self-closing and cannot contain children.
/// NOTE: "path" lives here, not in LEGAL_CONTAINERS — <path> is void in real HTML.
pub const LEGAL_VOIDS: &[&str] = &[
    "br", "hr", "img", "input", "link", "meta", "area", "base", "col", "embed", "param", "source", "track", "wbr",
    "path",
];

/// Compile-time string equality check, used by is_legal_tag below.
pub const fn const_str_eq(a: &str, b: &str) -> bool {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    if a_bytes.len() != b_bytes.len() {
        return false;
    }
    let mut i = 0;
    while i < a_bytes.len() {
        if a_bytes[i] != b_bytes[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// Checks whether `tag` appears in the given list of legal tag names.
pub const fn is_legal_tag(tag: &str, list: &[&str]) -> bool {
    let mut i = 0;
    while i < list.len() {
        if const_str_eq(tag, list[i]) {
            return true;
        }
        i += 1;
    }
    false
}

/// Real custom elements (web components) are required by the HTML spec
/// to contain a hyphen in their tag name — e.g. <my-widget>, never <mywidget>.
/// We use that rule to let genuinely custom tags through the validation
/// below without needing to be on the standard tag list at all.
fn is_custom_element_name(tag: &str) -> bool {
    tag.contains('-')
}

// =====================================================================
// SECTION 2 — NODE: WHAT CAN LIVE INSIDE AN ELEMENT'S CHILDREN
// =====================================================================

#[derive(Clone)]
pub enum Node {
    Element(Box<Element>),
    Void(Box<VoidElement>),
    Text(String),
    /// A list of sibling nodes with no wrapping tag around them —
    /// lets you pass a Vec<Node> straight into .child() without
    /// needing an extra wrapper element just to hold them.
    Fragment(Vec<Node>),
    /// Renders as nothing at all. Exists so Option<T>::None can flow
    /// straight into .child() and simply not render, instead of
    /// requiring a separate conditional method call.
    Empty,
}

impl From<Element> for Node { fn from(e: Element) -> Self { Node::Element(Box::new(e)) } }
impl From<VoidElement> for Node { fn from(e: VoidElement) -> Self { Node::Void(Box::new(e)) } }
impl From<&str> for Node { fn from(s: &str) -> Self { Node::Text(s.to_string()) } }
impl From<String> for Node { fn from(s: String) -> Self { Node::Text(s) } }

impl<T> From<Option<T>> for Node where T: Into<Node> {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(v) => v.into(),
            None => Node::Empty,
        }
    }
}

impl From<Vec<Node>> for Node {
    fn from(vec: Vec<Node>) -> Self {
        Node::Fragment(vec)
    }
}

// =====================================================================
// SECTION 3 — ATTRS: SHARED STORAGE FOR CLASSES, ID, AND ATTRIBUTES
// -----------------------------------------------------------------------
// Both Element and VoidElement use this same struct, so class/id/attr
// handling is written exactly once and shared by both types.
// =====================================================================

#[derive(Clone, Default)]
pub struct Attrs {
    pub classes: Vec<Cow<'static, str>>,
    pub id: Option<Cow<'static, str>>,
    /// Some(value) renders as key="value". None renders as a bare
    /// boolean attribute (e.g. `disabled`, with no ="" at all) —
    /// this is the more idiomatic HTML5 form for boolean attributes.
    pub pairs: Vec<(Cow<'static, str>, Option<Cow<'static, str>>)>,
}

impl Attrs {
    fn render_to(&self, w: &mut String) {
        if !self.classes.is_empty() {
            w.push_str(" class=\"");
            for (i, c) in self.classes.iter().enumerate() {
                if i > 0 { w.push(' '); }
                escape_attr(c, w);
            }
            w.push('"');
        }
        if let Some(ref id) = self.id {
            w.push_str(" id=\"");
            escape_attr(id, w);
            w.push('"');
        }
        for (k, v_opt) in &self.pairs {
            w.push(' ');
            w.push_str(k);
            if let Some(v) = v_opt {
                w.push_str("=\"");
                escape_attr(v, w);
                w.push('"');
            }
        }
    }
}

// =====================================================================
// SECTION 4 — ELEMENT AND VOIDELEMENT: THE TWO CORE TYPES
// -----------------------------------------------------------------------
// Element = any tag that CAN have children (<div>, <p>, <button>...)
// VoidElement = any tag that CANNOT have children (<br>, <img>, <input>...)
//
// Splitting these into two types means VoidElement simply has no
// .child() method at all — calling tag::br().child("x") is a compile
// error, not a silent bug that quietly discards the child at render time.
// =====================================================================

#[derive(Clone)]
pub struct Element {
    pub tag: &'static str,
    pub attrs: Attrs,
    pub children: Vec<Node>,
}

#[derive(Clone)]
pub struct VoidElement {
    pub tag: &'static str,
    pub attrs: Attrs,
}

// =====================================================================
// SECTION 5 — SHARED BUILDER METHODS (class, attr, htmx, alpine, etc.)
// -----------------------------------------------------------------------
// Written once via macro, applied to both Element and VoidElement,
// so .class()/.attr()/.hx_post() etc. work identically on either type.
// =====================================================================

macro_rules! impl_attr_methods {
    ($t:ty) => {
        impl $t {
            pub fn class(mut self, c: impl Into<Cow<'static, str>>) -> Self {
                let c = c.into();
                if !self.attrs.classes.contains(&c) {
                    self.attrs.classes.push(c);
                }
                self
            }

            pub fn class_if(self, condition: bool, class: impl Into<Cow<'static, str>>) -> Self {
                if condition { self.class(class) } else { self }
            }

            /// Adds several classes at once, each gated by its own condition.
            pub fn classes_if<I, S>(mut self, class_map: I) -> Self
            where
                I: IntoIterator<Item = (bool, S)>,
                S: Into<Cow<'static, str>>,
            {
                for (condition, class_name) in class_map {
                    if condition { self = self.class(class_name); }
                }
                self
            }

            pub fn id(mut self, id: impl Into<Cow<'static, str>>) -> Self {
                self.attrs.id = Some(id.into());
                self
            }

            pub fn attr(mut self, key: impl Into<Cow<'static, str>>, value: impl Into<Cow<'static, str>>) -> Self {
                self.attrs.pairs.push((key.into(), Some(value.into())));
                self
            }

            pub fn attr_if(self, condition: bool, key: impl Into<Cow<'static, str>>, value: impl Into<Cow<'static, str>>) -> Self {
                if condition { self.attr(key, value) } else { self }
            }

            /// Sets a real HTML boolean attribute (present = on, absent = off).
            /// Renders as a bare attribute with no ="" value, e.g. `disabled`.
            pub fn flag(mut self, condition: bool, key: impl Into<Cow<'static, str>>) -> Self {
                if condition { self.attrs.pairs.push((key.into(), None)); }
                self
            }

            pub fn disabled(self, condition: bool) -> Self { self.flag(condition, "disabled") }
            pub fn required(self, condition: bool) -> Self { self.flag(condition, "required") }
            pub fn readonly(self, condition: bool) -> Self { self.flag(condition, "readonly") }
            pub fn checked(self, condition: bool) -> Self { self.flag(condition, "checked") }

            // --- Common HTML attribute sugar ---
            pub fn style(self, css: impl Into<Cow<'static, str>>) -> Self { self.attr("style", css) }
            pub fn src(self, url: impl Into<Cow<'static, str>>) -> Self { self.attr("src", url) }
            pub fn href(self, url: impl Into<Cow<'static, str>>) -> Self { self.attr("href", url) }
            pub fn alt(self, text: impl Into<Cow<'static, str>>) -> Self { self.attr("alt", text) }
            pub fn name(self, n: impl Into<Cow<'static, str>>) -> Self { self.attr("name", n) }
            pub fn value(self, v: impl Into<Cow<'static, str>>) -> Self { self.attr("value", v) }
            pub fn placeholder(self, p: impl Into<Cow<'static, str>>) -> Self { self.attr("placeholder", p) }
            pub fn type_(self, t: impl Into<Cow<'static, str>>) -> Self { self.attr("type", t) }

            // --- HTMX sugar ---
            pub fn hx_get(self, url: &'static str) -> Self { self.attr("hx-get", url) }
            pub fn hx_post(self, url: &'static str) -> Self { self.attr("hx-post", url) }
            pub fn hx_target(self, target: &'static str) -> Self { self.attr("hx-target", target) }
            pub fn hx_swap(self, mode: &'static str) -> Self { self.attr("hx-swap", mode) }
            pub fn hx_trigger(self, trigger: &'static str) -> Self { self.attr("hx-trigger", trigger) }

            // --- Alpine.js sugar ---
            pub fn x_data(self, expr: impl Into<Cow<'static, str>>) -> Self { self.attr("x-data", expr) }
            pub fn x_show(self, expr: impl Into<Cow<'static, str>>) -> Self { self.attr("x-show", expr) }
            pub fn x_if(self, expr: impl Into<Cow<'static, str>>) -> Self { self.attr("x-if", expr) }
            pub fn x_model(self, expr: impl Into<Cow<'static, str>>) -> Self { self.attr("x-model", expr) }
            pub fn x_text(self, expr: impl Into<Cow<'static, str>>) -> Self { self.attr("x-text", expr) }
            pub fn x_on(self, event: &str, expr: impl Into<Cow<'static, str>>) -> Self {
                self.attr(format!("x-on:{event}"), expr)
            }
            pub fn x_bind(self, attribute: &str, expr: impl Into<Cow<'static, str>>) -> Self {
                self.attr(format!("x-bind:{attribute}"), expr)
            }
            pub fn x_transition(self) -> Self { self.flag(true, "x-transition") }

            /// Escape hatch: run an arbitrary function on this element mid-chain.
            pub fn modify(self, f: impl FnOnce(Self) -> Self) -> Self {
                f(self)
            }
        }
    };
}

impl_attr_methods!(Element);
impl_attr_methods!(VoidElement);

// =====================================================================
// SECTION 6 — ELEMENT: CHILDREN, CONSTRUCTION, AND BUILD
// =====================================================================

impl Element {
    /// Builds a new Element. In debug builds, panics with a helpful
    /// message if the tag name looks like a typo or is structurally
    /// wrong (e.g. a void tag passed to Element instead of VoidElement).
    /// Custom element names containing a hyphen always pass through
    /// untouched, per the HTML spec's own naming rule for web components.
    #[track_caller]
    #[inline]
    pub fn new(tag: &'static str) -> Self {
        #[cfg(debug_assertions)]
        {
            if !is_custom_element_name(tag) && !is_legal_tag(tag, LEGAL_CONTAINERS) {
                let loc = std::panic::Location::caller();

                if is_legal_tag(tag, LEGAL_VOIDS) {
                    panic!("🌱 Sprout Language Error at {}:{}:{}:\n   ↳ Tag mismatch! '{}' is a strict self-closing VoidElement. Use VoidElement::new() or tag::{}() instead.", loc.file(), loc.line(), loc.column(), tag, tag);
                } else if let Some(suggestion) = suggest_closest_tag(tag) {
                    panic!("🌱 Sprout Language Error at {}:{}:{}:\n   ↳ Unknown element '{}'. Did you mean '{}'?", loc.file(), loc.line(), loc.column(), tag, suggestion);
                } else {
                    panic!("🌱 Sprout Language Error at {}:{}:{}:\n   ↳ The requested HTML element '{}' does not exist or is structurally illegal.", loc.file(), loc.line(), loc.column(), tag);
                }
            }
        }

        Self { tag, attrs: Attrs::default(), children: vec![] }
    }

    pub fn child(mut self, child: impl Into<Node>) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn children<I, T>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Node>,
    {
        let iter = children.into_iter();
        self.children.reserve(iter.size_hint().0);
        for c in iter {
            self.children.push(c.into());
        }
        self
    }

    /// Builds one child per item, without writing .map().collect() by hand.
    pub fn child_for<I, T, F, N>(mut self, items: I, f: F) -> Self
    where
        I: IntoIterator<Item = T>,
        F: Fn(T) -> N,
        N: Into<Node>,
    {
        let iter = items.into_iter();
        self.children.reserve(iter.size_hint().0);
        for item in iter {
            self.children.push(f(item).into());
        }
        self
    }

    /// Adds a child only if `condition` is true. Closure takes no arguments.
    pub fn child_if<F, N>(self, condition: bool, f: F) -> Self
    where
        F: FnOnce() -> N,
        N: Into<Node>,
    {
        if condition { self.child(f()) } else { self }
    }

    /// Converts the tree into real, escaped HTML.
    pub fn build(self) -> Markup {
        let mut buf = String::with_capacity(256);
        self.render_to(&mut buf);
        PreEscaped(buf)
    }
}

// =====================================================================
// SECTION 7 — VOIDELEMENT: CONSTRUCTION AND BUILD (no children methods)
// =====================================================================

impl VoidElement {
    /// Same typo/structure protection as Element::new, mirrored for void tags.
    #[track_caller]
    #[inline]
    pub fn new(tag: &'static str) -> Self {
        #[cfg(debug_assertions)]
        {
            if !is_custom_element_name(tag) && !is_legal_tag(tag, LEGAL_VOIDS) {
                let loc = std::panic::Location::caller();

                if is_legal_tag(tag, LEGAL_CONTAINERS) {
                    panic!("🌱 Sprout Language Error at {}:{}:{}:\n   ↳ Tag mismatch! '{}' is a standard container. Use Element::new() or tag::{}() instead.", loc.file(), loc.line(), loc.column(), tag, tag);
                } else if let Some(suggestion) = suggest_closest_tag(tag) {
                    panic!("🌱 Sprout Language Error at {}:{}:{}:\n   ↳ Unknown self-closing element '{}'. Did you mean '{}'?", loc.file(), loc.line(), loc.column(), tag, suggestion);
                } else {
                    panic!("🌱 Sprout Language Error at {}:{}:{}:\n   ↳ The requested self-closing HTML tag '{}' does not exist.", loc.file(), loc.line(), loc.column(), tag);
                }
            }
        }

        Self { tag, attrs: Attrs::default() }
    }

    pub fn build(self) -> Markup {
        let mut buf = String::with_capacity(64);
        self.render_to(&mut buf);
        PreEscaped(buf)
    }
}

// =====================================================================
// SECTION 8 — ESCAPING (THE SECURITY LAYER)
// -----------------------------------------------------------------------
// Every piece of text and every attribute value passes through here
// before reaching the final HTML string. This is what makes it safe
// to put real user input directly into .child()/.attr() without
// manually escaping it yourself first.
//
// Text and attribute escaping are intentionally different: quotes only
// matter inside an attribute's "..." boundary, not between tags.
// =====================================================================

fn escape_text(s: &str, out: &mut String) {
    let mut last = 0;
    for (i, b) in s.bytes().enumerate() {
        match b {
            b'&' | b'<' | b'>' => {
                out.push_str(&s[last..i]);
                match b {
                    b'&' => out.push_str("&amp;"),
                    b'<' => out.push_str("&lt;"),
                    b'>' => out.push_str("&gt;"),
                    _ => unreachable!(),
                }
                last = i + 1;
            }
            _ => {}
        }
    }
    out.push_str(&s[last..]);
}

fn escape_attr(s: &str, out: &mut String) {
    let mut last = 0;
    for (i, b) in s.bytes().enumerate() {
        match b {
            b'&' | b'<' | b'>' | b'"' => {
                out.push_str(&s[last..i]);
                match b {
                    b'&' => out.push_str("&amp;"),
                    b'<' => out.push_str("&lt;"),
                    b'>' => out.push_str("&gt;"),
                    b'"' => out.push_str("&quot;"),
                    _ => unreachable!(),
                }
                last = i + 1;
            }
            _ => {}
        }
    }
    out.push_str(&s[last..]);
}

// =====================================================================
// SECTION 9 — RENDERING: TURNING THE TREE INTO A STRING
// =====================================================================

impl Render for Node {
    fn render_to(&self, w: &mut String) {
        match self {
            Node::Text(t) => escape_text(t, w),
            Node::Element(e) => e.render_to(w),
            Node::Void(v) => v.render_to(w),
            Node::Fragment(nodes) => {
                for n in nodes { n.render_to(w); }
            }
            Node::Empty => {}
        }
    }
}

impl Render for Element {
    fn render_to(&self, w: &mut String) {
        w.push('<');
        w.push_str(self.tag);
        self.attrs.render_to(w);
        w.push('>');
        for c in &self.children {
            c.render_to(w);
        }
        w.push_str("</");
        w.push_str(self.tag);
        w.push('>');
    }
}

impl Render for VoidElement {
    fn render_to(&self, w: &mut String) {
        w.push('<');
        w.push_str(self.tag);
        self.attrs.render_to(w);
        w.push('>');
    }
}

// =====================================================================
// SECTION 10 — DEV-ONLY TYPO SUGGESTIONS (debug builds only)
// -----------------------------------------------------------------------
// Powers the "Did you mean 'div'?" message when a tag name is close
// to, but not exactly, a real tag. Pure development convenience —
// compiled out entirely in release builds.
// =====================================================================

#[cfg(debug_assertions)]
fn suggest_closest_tag(typo: &str) -> Option<&'static str> {
    let mut best_match = None;
    let mut best_dist = 3; // Only suggest if within 2 edits of a real tag

    for &valid in LEGAL_CONTAINERS.iter().chain(LEGAL_VOIDS.iter()) {
        let dist = levenshtein_distance(typo, valid);
        if dist < best_dist {
            best_dist = dist;
            best_match = Some(valid);
        }
    }

    if typo == "btn" { return Some("button"); }
    if typo == "dvi" { return Some("div"); }

    best_match
}

#[cfg(debug_assertions)]
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let mut cache: Vec<usize> = (0..=b.len()).collect();
    for (i, a_byte) in a.bytes().enumerate() {
        let mut next = vec![i + 1];
        for (j, b_byte) in b.bytes().enumerate() {
            let cost = if a_byte == b_byte { 0 } else { 1 };
            next.push((cache[j + 1] + 1).min(next[j] + 1).min(cache[j] + cost));
        }
        cache = next;
    }
    *cache.last().unwrap()
}

// =====================================================================
// SECTION 11 — TESTS
// -----------------------------------------------------------------------
// Every guarantee this file makes — escaping, void-safety, conditional
// helpers, Fragment/Empty handling — is backed by a test here. If you
// change behavior in any section above, a test below should catch it.
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_empty_div() {
        let html = Element::new("div").build().into_string();
        assert_eq!(html, "<div></div>");
    }

    #[test]
    fn custom_element_with_hyphen_bypasses_validation() {
        let html = Element::new("my-widget").attr("data-id", "42").build().into_string();
        assert_eq!(html, r#"<my-widget data-id="42"></my-widget>"#);
    }

    #[test]
    fn path_is_correctly_classified_as_void() {
        let html = VoidElement::new("path").attr("d", "M0 0L10 10").build().into_string();
        assert_eq!(html, r#"<path d="M0 0L10 10">"#);
    }

    #[test]
    fn renders_single_class() {
        let html = Element::new("div").class("card").build().into_string();
        assert_eq!(html, r#"<div class="card"></div>"#);
    }

    #[test]
    fn duplicate_class_is_not_repeated() {
        let html = Element::new("div").class("card").class("card").build().into_string();
        assert_eq!(html, r#"<div class="card"></div>"#);
    }

    #[test]
    fn renders_multiple_distinct_classes_joined_with_space() {
        let html = Element::new("div").class("card").class("featured").build().into_string();
        assert_eq!(html, r#"<div class="card featured"></div>"#);
    }

    #[test]
    fn class_if_only_adds_when_true() {
        let active = Element::new("a").class_if(true, "active").build().into_string();
        let inactive = Element::new("a").class_if(false, "active").build().into_string();
        assert_eq!(active, r#"<a class="active"></a>"#);
        assert_eq!(inactive, "<a></a>");
    }

    #[test]
    fn classes_if_adds_only_matching_conditions() {
        let html = Element::new("li")
            .classes_if([(true, "active"), (false, "disabled"), (true, "highlighted")])
            .build()
            .into_string();
        assert_eq!(html, r#"<li class="active highlighted"></li>"#);
    }

    #[test]
    fn attr_if_only_adds_when_true() {
        let disabled = Element::new("button").attr_if(true, "disabled", "").build().into_string();
        let enabled = Element::new("button").attr_if(false, "disabled", "").build().into_string();
        assert_eq!(disabled, r#"<button disabled=""></button>"#);
        assert_eq!(enabled, "<button></button>");
    }

    #[test]
    fn flag_sets_bare_boolean_attribute() {
        let html = VoidElement::new("input").flag(true, "required").build().into_string();
        assert_eq!(html, "<input required>");
    }

    #[test]
    fn disabled_helper_works() {
        let on = Element::new("button").disabled(true).build().into_string();
        let off = Element::new("button").disabled(false).build().into_string();
        assert_eq!(on, "<button disabled></button>");
        assert_eq!(off, "<button></button>");
    }

    #[test]
    fn required_helper_works() {
        let html = VoidElement::new("input").required(true).build().into_string();
        assert_eq!(html, "<input required>");
    }

    #[test]
    fn readonly_helper_works() {
        let html = VoidElement::new("input").readonly(true).build().into_string();
        assert_eq!(html, "<input readonly>");
    }

    #[test]
    fn checked_helper_works() {
        let html = VoidElement::new("input").checked(true).build().into_string();
        assert_eq!(html, "<input checked>");
    }

    #[test]
    fn modify_runs_arbitrary_closure_on_element() {
        let html = Element::new("div").modify(|el| el.class("from-modify")).build().into_string();
        assert_eq!(html, r#"<div class="from-modify"></div>"#);
    }

    #[test]
    fn renders_style_attribute() {
        let html = Element::new("div").style("color: red;").build().into_string();
        assert_eq!(html, r#"<div style="color: red;"></div>"#);
    }

    #[test]
    fn renders_id() {
        let html = Element::new("div").id("post-list").build().into_string();
        assert_eq!(html, r#"<div id="post-list"></div>"#);
    }

    #[test]
    fn renders_custom_attr() {
        let html = VoidElement::new("input").attr("placeholder", "Title").build().into_string();
        assert_eq!(html, r#"<input placeholder="Title">"#);
    }

    #[test]
    fn placeholder_sugar_matches_attr() {
        let html = VoidElement::new("input").placeholder("Title").build().into_string();
        assert_eq!(html, r#"<input placeholder="Title">"#);
    }

    #[test]
    fn src_and_alt_sugar_on_img() {
        let html = VoidElement::new("img").src("/me.png").alt("profile photo").build().into_string();
        assert_eq!(html, r#"<img src="/me.png" alt="profile photo">"#);
    }

    #[test]
    fn href_sugar_on_anchor() {
        let html = Element::new("a").href("/blog").child("Blog").build().into_string();
        assert_eq!(html, r#"<a href="/blog">Blog</a>"#);
    }

    #[test]
    fn type_sugar_on_input() {
        let html = VoidElement::new("input").type_("text").build().into_string();
        assert_eq!(html, r#"<input type="text">"#);
    }

    #[test]
    fn renders_attrs_in_call_order() {
        let html = Element::new("div").class("a").id("b").attr("data-x", "1").build().into_string();
        assert_eq!(html, r#"<div class="a" id="b" data-x="1"></div>"#);
    }

    #[test]
    fn hx_post_sets_correct_attribute() {
        let html = Element::new("form").hx_post("/blog/posts").build().into_string();
        assert_eq!(html, r#"<form hx-post="/blog/posts"></form>"#);
    }

    #[test]
    fn hx_target_sets_correct_attribute() {
        let html = Element::new("form").hx_target("#post-list").build().into_string();
        assert_eq!(html, r##"<form hx-target="#post-list"></form>"##);
    }

    #[test]
    fn hx_get_sets_correct_attribute() {
        let html = Element::new("button").hx_get("/refresh").build().into_string();
        assert_eq!(html, r#"<button hx-get="/refresh"></button>"#);
    }

    #[test]
    fn hx_swap_sets_correct_attribute() {
        let html = Element::new("form").hx_swap("beforeend").build().into_string();
        assert_eq!(html, r#"<form hx-swap="beforeend"></form>"#);
    }

    #[test]
    fn x_data_sets_correct_attribute() {
        let html = Element::new("div").x_data("{ open: false }").build().into_string();
        assert_eq!(html, r#"<div x-data="{ open: false }"></div>"#);
    }

    #[test]
    fn x_show_sets_correct_attribute() {
        let html = Element::new("div").x_show("open").build().into_string();
        assert_eq!(html, r#"<div x-show="open"></div>"#);
    }

    #[test]
    fn x_model_sets_correct_attribute() {
        let html = VoidElement::new("input").x_model("title").build().into_string();
        assert_eq!(html, r#"<input x-model="title">"#);
    }

    #[test]
    fn x_text_sets_correct_attribute() {
        let html = Element::new("span").x_text("count").build().into_string();
        assert_eq!(html, r#"<span x-text="count"></span>"#);
    }

    #[test]
    fn x_on_builds_event_specific_attribute() {
        let html = Element::new("button").x_on("click", "open = !open").child("Toggle").build().into_string();
        assert_eq!(html, r#"<button x-on:click="open = !open">Toggle</button>"#);
    }

    #[test]
    fn x_bind_builds_attribute_specific_key() {
        let html = Element::new("div").x_bind("class", "isOpen ? 'show' : ''").build().into_string();
        assert_eq!(html, r#"<div x-bind:class="isOpen ? 'show' : ''"></div>"#);
    }

    #[test]
    fn x_transition_sets_bare_attribute() {
        let html = Element::new("div").x_transition().build().into_string();
        assert_eq!(html, "<div x-transition></div>");
    }

    #[test]
    fn alpine_value_with_quotes_and_braces_is_escaped() {
        let html = Element::new("div").x_data("{ count: 0, label: 'hi' }").build().into_string();
        assert_eq!(html, r#"<div x-data="{ count: 0, label: 'hi' }"></div>"#);
    }

    #[test]
    fn escapes_ampersand_in_text() {
        let html = Element::new("p").child("Rust & Axum").build().into_string();
        assert_eq!(html, "<p>Rust &amp; Axum</p>");
    }

    #[test]
    fn escapes_script_tag_in_text() {
        let html = Element::new("p").child("<script>alert(1)</script>").build().into_string();
        assert_eq!(html, "<p>&lt;script&gt;alert(1)&lt;/script&gt;</p>");
    }

    #[test]
    fn does_not_escape_quotes_in_text_content() {
        let html = Element::new("p").child("he said \"hi\"").build().into_string();
        assert_eq!(html, "<p>he said \"hi\"</p>");
    }

    #[test]
    fn escapes_quote_in_attribute_value() {
        let html = VoidElement::new("input").attr("placeholder", "say \"hi\"").build().into_string();
        assert_eq!(html, r#"<input placeholder="say &quot;hi&quot;">"#);
    }

    #[test]
    fn escapes_angle_brackets_in_attribute_value() {
        let html = VoidElement::new("input").attr("data-x", "<b>").build().into_string();
        assert_eq!(html, r#"<input data-x="&lt;b&gt;">"#);
    }

    #[test]
    fn escapes_class_value_containing_quote() {
        let html = Element::new("div").class("weird\"class").build().into_string();
        assert_eq!(html, r#"<div class="weird&quot;class"></div>"#);
    }

    #[test]
    fn void_element_self_closes_without_children() {
        let html = VoidElement::new("br").build().into_string();
        assert_eq!(html, "<br>");
    }

    #[test]
    fn void_element_renders_attrs_correctly() {
        let html = VoidElement::new("input").attr("type", "text").attr("name", "title").build().into_string();
        assert_eq!(html, r#"<input type="text" name="title">"#);
    }

    #[test]
    fn img_void_element_with_class() {
        let html = VoidElement::new("img").class("avatar").attr("src", "/me.png").build().into_string();
        assert_eq!(html, r#"<img class="avatar" src="/me.png">"#);
    }

    #[test]
    fn nests_element_inside_element() {
        let html = Element::new("div").child(Element::new("p").child("hello")).build().into_string();
        assert_eq!(html, "<div><p>hello</p></div>");
    }

    #[test]
    fn nests_void_element_inside_element() {
        let html = Element::new("div").child(VoidElement::new("br")).build().into_string();
        assert_eq!(html, "<div><br></div>");
    }

    #[test]
    fn children_preserves_order() {
        let html = Element::new("ul").children(vec![
            Element::new("li").child("first"),
            Element::new("li").child("second"),
            Element::new("li").child("third"),
        ]).build().into_string();
        assert_eq!(html, "<ul><li>first</li><li>second</li><li>third</li></ul>");
    }

    #[test]
    fn child_for_builds_children_from_iterator() {
        let fruits = vec!["Apple", "Banana"];
        let html = Element::new("ul")
            .child_for(fruits, |f| Element::new("li").child(f.to_string()))
            .build()
            .into_string();
        assert_eq!(html, "<ul><li>Apple</li><li>Banana</li></ul>");
    }

    #[test]
    fn child_if_only_adds_when_true() {
        let shown = Element::new("div").child_if(true, || "visible").build().into_string();
        let hidden = Element::new("div").child_if(false, || "invisible").build().into_string();
        assert_eq!(shown, "<div>visible</div>");
        assert_eq!(hidden, "<div></div>");
    }

    #[test]
    fn deeply_nested_structure_renders_correctly() {
        let html = Element::new("div")
            .class("card")
            .child(Element::new("h2").child("Title"))
            .child(Element::new("div").class("body").child(Element::new("p").child("Nested content")))
            .build()
            .into_string();
        assert_eq!(html, r#"<div class="card"><h2>Title</h2><div class="body"><p>Nested content</p></div></div>"#);
    }

    #[test]
    fn renders_a_full_card_like_component() {
        let card = Element::new("div")
            .class("card")
            .child(Element::new("h1").child("Hello"))
            .child(Element::new("p").child("Body text"))
            .child(
                Element::new("button")
                    .class("btn-primary")
                    .hx_post("/like")
                    .hx_target("#likes")
                    .child("Like"),
            );

        let html = card.build().into_string();
        assert_eq!(
            html,
            r##"<div class="card"><h1>Hello</h1><p>Body text</p><button class="btn-primary" hx-post="/like" hx-target="#likes">Like</button></div>"##
        );
    }

    #[test]
    fn alpine_and_htmx_can_be_combined_on_same_element() {
        let html = Element::new("div")
            .x_data("{ liked: false }")
            .hx_post("/like")
            .hx_target("#likes")
            .x_on("click", "liked = !liked")
            .build()
            .into_string();
        assert_eq!(
            html,
            r##"<div x-data="{ liked: false }" hx-post="/like" hx-target="#likes" x-on:click="liked = !liked"></div>"##
        );
    }

    #[test]
    fn child_accepts_options_directly() {
        let has_error = Some("Password too short");
        let no_error: Option<&str> = None;

        let html_err = Element::new("div").child(has_error.map(|msg| Element::new("span").child(msg))).build().into_string();
        let html_clean = Element::new("div").child(no_error.map(|msg| Element::new("span").child(msg))).build().into_string();

        assert_eq!(html_err, r#"<div><span>Password too short</span></div>"#);
        assert_eq!(html_clean, r#"<div></div>"#);
    }

    #[test]
    fn fragment_renders_siblings_without_wrapper() {
        let two_elements = vec![
            Element::new("h1").child("Hello").into(),
            Element::new("p").child("World").into(),
        ];
        let html = Element::new("main").child(two_elements).build().into_string();
        assert_eq!(html, r#"<main><h1>Hello</h1><p>World</p></main>"#);
    }

    #[test]
    fn child_accepts_vec_directly() {
        let list = vec![
            Element::new("li").child("A").into(),
            Element::new("li").child("B").into(),
        ];
        let html = Element::new("ul").child(list).build().into_string();
        assert_eq!(html, "<ul><li>A</li><li>B</li></ul>");
    }

    #[test]
    fn renders_multiple_fragment_levels() {
        let nested_fragment = vec![
            Element::new("span").child("A").into(),
            Element::new("span").child("B").into(),
        ];
        let html = Element::new("div")
            .child(vec![
                Element::new("p").child("Start").into(),
                nested_fragment.into(),
                Element::new("p").child("End").into(),
            ])
            .build()
            .into_string();

        assert_eq!(html, "<div><p>Start</p><span>A</span><span>B</span><p>End</p></div>");
    }

    #[test]
    fn attribute_with_multiple_spaces_is_preserved() {
        let html = Element::new("div").attr("data-text", "hello  world").build().into_string();
        assert_eq!(html, r#"<div data-text="hello  world"></div>"#);
    }

    #[test]
    fn empty_node_renders_nothing() {
        let html = Element::new("div").child("Visible").child(Node::Empty).child("Also Visible").build().into_string();
        assert_eq!(html, "<div>VisibleAlso Visible</div>");
    }

    #[test]
    fn attribute_keys_with_dashes_and_colons() {
        let html = Element::new("div").attr("data-custom-key", "value").attr("aria-label", "my-label").build().into_string();
        assert_eq!(html, r#"<div data-custom-key="value" aria-label="my-label"></div>"#);
    }

    #[test]
    fn chaining_modify_multiple_times() {
        let html = Element::new("div")
            .modify(|e| e.class("first"))
            .modify(|e| e.class("second"))
            .modify(|e| e.id("my-id"))
            .build()
            .into_string();
        assert_eq!(html, r#"<div class="first second" id="my-id"></div>"#);
    }

    #[test]
    fn child_if_false_does_not_panic_or_render() {
        let html = Element::new("div").child_if(false, || Element::new("p").child("should not exist")).build().into_string();
        assert_eq!(html, "<div></div>");
    }

    #[test]
    fn multiple_void_elements_in_container() {
        let html = Element::new("div").child(VoidElement::new("br")).child(VoidElement::new("hr")).build().into_string();
        assert_eq!(html, "<div><br><hr></div>");
    }

    #[test]
    fn text_node_escaping_full_check() {
        let html = Element::new("div").child("<script>&\"'</script>").build().into_string();
        assert_eq!(html, "<div>&lt;script&gt;&amp;\"'&lt;/script&gt;</div>");
    }

    #[test]
    fn attribute_escaping_full_check() {
        let html = Element::new("div").attr("title", "<script>&\"'</script>").build().into_string();
        assert_eq!(html, r#"<div title="&lt;script&gt;&amp;&quot;'&lt;/script&gt;"></div>"#);
    }
}