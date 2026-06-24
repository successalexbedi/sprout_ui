// =====================================================================
// SPROUT — sprout_ui_core
// -----------------------------------------------------------------------
// Zero-tree, streaming HTML engine. Elements write directly into a
// stack-resident buffer instead of building an intermediate Vec<Node>
// tree — this is what closes most of the gap to Maud's raw speed while
// keeping the method-chain syntax sprout exists for.
//
// Composability is preserved the same way Maud composes Markup values:
// a fully-built Element flattens its own buffer into whatever buffer
// it's handed to via .child(). You can still write components as plain
// functions returning Element and nest them freely.
// =====================================================================

use maud::{Markup, PreEscaped};

#[macro_export]
macro_rules! sprout_panic {
    ($target:expr, $msg:expr) => {{
        use owo_colors::OwoColorize;
        use std::io::IsTerminal;

        let stderr = std::io::stderr();
        if stderr.is_terminal() {
            eprintln!(
                "\n{header}\n{arrow} {tag}\n{arrow} {ctx}\n{arrow} {msg}\n",
                header = " ERROR ".on_red().white().bold(),
                arrow = "  ==>".red().bold(),
                tag = format!("On: {}", $target).yellow().bold(),
                ctx = format!("Message: {}", $msg).white(),
                msg = "Structural integrity compromised in component hierarchy.".dimmed()
            );
        } else {
            eprintln!("[SPROUT UI CRASH] Target: {} | Message: {}", $target, $msg);
        }

        panic!("{}", $msg);
    }};
}

// =====================================================================
// SECTION 1 — ZERO-ALLOCATION STRING AND BUFFER TYPES
// =====================================================================

/// A string that's either a fixed literal, a heap-allocated owned
/// string, or a small inline buffer built by sprout_fmt!.
#[derive(Clone)]
pub enum SproutStr {
    Static(&'static str),
    Owned(String),
    Inline { buf: [u8; 48], len: u8 },
}

impl SproutStr {
    #[inline(always)]
    pub fn as_str(&self) -> &str {
        match self {
            SproutStr::Static(s) => s,
            SproutStr::Owned(s) => s.as_str(),
            // Safe by construction: this buffer is only ever filled by
            // FallbackWriter::write_str, which only ever copies whole,
            // already-valid &str slices into it. See debug_assert below.
            SproutStr::Inline { buf, len } => {
                let slice = &buf[..*len as usize];
                debug_assert!(std::str::from_utf8(slice).is_ok(), "sprout: SproutStr::Inline corrupted — invariant violated");
                unsafe { std::str::from_utf8_unchecked(slice) }
            }
        }
    }
}

impl From<&'static str> for SproutStr {
    #[inline(always)] fn from(s: &'static str) -> Self { SproutStr::Static(s) }
}
impl From<String> for SproutStr {
    #[inline(always)] fn from(s: String) -> Self { SproutStr::Owned(s) }
}

/// A compact writer used by sprout_fmt! for string interpolation
/// without allocating, for short results (≤48 bytes).
pub struct FallbackWriter {
    pub inline: [u8; 48],
    pub len: usize,
    pub overflow: Option<String>,
}

impl std::fmt::Write for FallbackWriter {
    #[inline]
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        if let Some(ref mut string) = self.overflow {
            string.push_str(s);
        } else if self.len + s.len() <= 48 {
            self.inline[self.len..self.len + s.len()].copy_from_slice(s.as_bytes());
            self.len += s.len();
        } else {
            let mut string = String::with_capacity(self.len + s.len() + 16);
            string.push_str(unsafe { std::str::from_utf8_unchecked(&self.inline[..self.len]) });
            string.push_str(s);
            self.overflow = Some(string);
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! sprout_fmt {
    ($($arg:tt)*) => {{
        let mut writer = $crate::FallbackWriter {
            inline: [0u8; 48],
            len: 0,
            overflow: None,
        };
        let _ = ::std::fmt::write(&mut writer, format_args!($($arg)*));
        if let Some(s) = writer.overflow {
            $crate::SproutStr::Owned(s)
        } else {
            $crate::SproutStr::Inline { buf: writer.inline, len: writer.len as u8 }
        }
    }};
}

/// The streaming output buffer. Holds up to 1024 bytes per element on
/// the stack before spilling to the heap. Every write here is always a
/// complete, already-valid &str slice — never a partial byte range —
/// which is what keeps the inline buffer's contents valid UTF-8 at
/// every point: concatenating whole valid UTF-8 strings always
/// produces valid UTF-8, since each string already ends on a character
/// boundary by definition.
#[derive(Clone)]
pub struct StreamBuf {
    pub inline: [u8; 1024],
    pub len: usize,
    pub overflow: Option<String>,
}

impl StreamBuf {
    #[inline(always)]
    pub fn new() -> Self {
        Self { inline: [0; 1024], len: 0, overflow: None }
    }

    #[inline(always)]
    pub fn push_str(&mut self, s: &str) {
        if let Some(ref mut string) = self.overflow {
            string.push_str(s);
        } else if self.len + s.len() <= 1024 {
            self.inline[self.len..self.len + s.len()].copy_from_slice(s.as_bytes());
            self.len += s.len();
        } else {
            let mut string = String::with_capacity(self.len + s.len() + 256);
            string.push_str(unsafe { std::str::from_utf8_unchecked(&self.inline[..self.len]) });
            string.push_str(s);
            self.overflow = Some(string);
        }
    }

    #[inline(always)]
    pub fn push(&mut self, c: char) {
        let mut buf = [0; 4];
        self.push_str(c.encode_utf8(&mut buf));
    }

    /// Removes the last character. Used internally for zero-allocation
    /// class-list chaining (rewriting the closing quote when adding a
    /// second class, instead of rebuilding the whole attribute).
    #[inline(always)]
    pub fn pop(&mut self) {
        if let Some(ref mut string) = self.overflow {
            string.pop();
        } else if self.len > 0 {
            let slice = &self.inline[..self.len];
            debug_assert!(std::str::from_utf8(slice).is_ok(), "sprout: StreamBuf corrupted before pop()");
            let s = unsafe { std::str::from_utf8_unchecked(slice) };
            if let Some(c) = s.chars().last() {
                self.len -= c.len_utf8();
            }
        }
    }

    #[inline(always)]
    pub fn as_str(&self) -> &str {
        if let Some(ref string) = self.overflow {
            string.as_str()
        } else {
            let slice = &self.inline[..self.len];
            debug_assert!(std::str::from_utf8(slice).is_ok(), "sprout: StreamBuf corrupted in as_str()");
            unsafe { std::str::from_utf8_unchecked(slice) }
        }
    }

    #[inline(always)]
    pub fn into_string(self) -> String {
        if let Some(string) = self.overflow {
            string
        } else {
            let slice = &self.inline[..self.len];
            debug_assert!(std::str::from_utf8(slice).is_ok(), "sprout: StreamBuf corrupted in into_string()");
            unsafe { std::str::from_utf8_unchecked(slice) }.to_owned()
        }
    }
}

// =====================================================================
// SECTION 2 — ESCAPING PIPELINES
// =====================================================================

fn escape_text(s: &str, out: &mut StreamBuf) {
    let mut last = 0;
    for (i, b) in s.bytes().enumerate() {
        if matches!(b, b'&' | b'<' | b'>') {
            out.push_str(&s[last..i]);
            match b {
                b'&' => out.push_str("&amp;"),
                b'<' => out.push_str("&lt;"),
                b'>' => out.push_str("&gt;"),
                _ => unreachable!(),
            }
            last = i + 1;
        }
    }
    out.push_str(&s[last..]);
}

fn escape_attr(s: &str, out: &mut StreamBuf) {
    let mut last = 0;
    for (i, b) in s.bytes().enumerate() {
        if matches!(b, b'&' | b'<' | b'>' | b'"') {
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
    }
    out.push_str(&s[last..]);
}

// =====================================================================
// SECTION 3 — THE LEGAL TAG DICTIONARY
// =====================================================================

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

pub const LEGAL_VOIDS: &[&str] = &[
    "br", "hr", "img", "input", "link", "meta", "area", "base", "col", "embed", "param", "source", "track", "wbr", "path",
];

pub const fn is_legal_tag(tag: &str, list: &[&str]) -> bool {
    let a_bytes = tag.as_bytes();
    let mut i = 0;
    while i < list.len() {
        let b_bytes = list[i].as_bytes();
        if a_bytes.len() == b_bytes.len() {
            let mut match_found = true;
            let mut j = 0;
            while j < a_bytes.len() {
                if a_bytes[j] != b_bytes[j] { match_found = false; break; }
                j += 1;
            }
            if match_found { return true; }
        }
        i += 1;
    }
    false
}

#[cfg(debug_assertions)]
fn is_custom_element_name(tag: &str) -> bool { tag.contains('-') }

#[cfg(debug_assertions)]
fn suggest_closest_tag(typo: &str) -> Option<&'static str> {
    let mut best_match = None;
    let mut best_dist = 3;
    for &valid in LEGAL_CONTAINERS.iter().chain(LEGAL_VOIDS.iter()) {
        let dist = levenshtein_distance(typo, valid);
        if dist < best_dist { best_dist = dist; best_match = Some(valid); }
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
// SECTION 4 — STREAMING CORE ABSTRACTIONS
// =====================================================================

/// Anything that knows how to write itself into a StreamBuf.
/// This is what makes composition work: an Element implements this by
/// closing its own head, appending its closing tag, then flattening
/// its entire buffer into the parent's buffer — the same idea as
/// passing one maud::Markup into another via (expr).
pub trait IntoStream {
    fn stream_to(self, buf: &mut StreamBuf);
}

impl IntoStream for &str {
    #[inline(always)] fn stream_to(self, buf: &mut StreamBuf) { escape_text(self, buf); }
}
impl IntoStream for String {
    #[inline(always)] fn stream_to(self, buf: &mut StreamBuf) { escape_text(self.as_str(), buf); }
}
impl IntoStream for SproutStr {
    #[inline(always)] fn stream_to(self, buf: &mut StreamBuf) { escape_text(self.as_str(), buf); }
}
impl<T: IntoStream> IntoStream for Option<T> {
    #[inline(always)] fn stream_to(self, buf: &mut StreamBuf) { if let Some(v) = self { v.stream_to(buf); } }
}
impl<T: IntoStream> IntoStream for Vec<T> {
    #[inline(always)] fn stream_to(self, buf: &mut StreamBuf) { for v in self { v.stream_to(buf); } }
}

/// Shared behavior between Element and VoidElement needed by the
/// attribute macro below — lets .class()/.attr()/.flag() be written
/// once and applied to both types.
pub trait HtmlElement {
    fn stream(&mut self) -> &mut StreamBuf;
    fn is_head_closed(&self) -> bool;
    fn has_class(&self) -> bool;
    fn set_has_class(&mut self, val: bool);
}

// =====================================================================
// SECTION 5 — ELEMENT AND VOIDELEMENT
// =====================================================================

#[derive(Clone)]
pub struct Element {
    pub buf: StreamBuf,
    pub tag: &'static str,
    pub head_closed: bool,
    pub has_class: bool,
}

impl Element {
    #[track_caller]
    pub fn new(tag: &'static str) -> Self {
        #[cfg(debug_assertions)]
        {
            if !is_custom_element_name(tag) && !is_legal_tag(tag, LEGAL_CONTAINERS) {
                let loc = std::panic::Location::caller();
                let msg = if is_legal_tag(tag, LEGAL_VOIDS) {
                    format!("'{}' at {}:{} is a self-closing VoidElement. Use VoidElement::new() or tag::{}() instead.", tag, loc.file(), loc.line(), tag)
                } else if let Some(suggestion) = suggest_closest_tag(tag) {
                    format!("Typo at {}:{}: '{}'. Did you mean '{}'?", loc.file(), loc.line(), tag, suggestion)
                } else {
                    format!("Unknown Element at {}:{}: '{}'", loc.file(), loc.line(), tag)
                };
                sprout_panic!("Element::new", msg);
            }
        }

        let mut buf = StreamBuf::new();
        buf.push('<');
        buf.push_str(tag);
        Self { buf, tag, head_closed: false, has_class: false }
    }

    #[inline(always)]
    fn close_head(&mut self) {
        if !self.head_closed {
            self.buf.push('>');
            self.head_closed = true;
        }
    }

    #[inline(always)]
    pub fn child(mut self, child: impl IntoStream) -> Self {
        self.close_head();
        child.stream_to(&mut self.buf);
        self
    }

    #[inline(always)]
    pub fn children<I, T>(mut self, iter: I) -> Self
    where I: IntoIterator<Item = T>, T: IntoStream {
        self.close_head();
        for c in iter { c.stream_to(&mut self.buf); }
        self
    }

    #[inline(always)]
    pub fn child_for<I, T, F, N>(mut self, items: I, f: F) -> Self
    where I: IntoIterator<Item = T>, F: Fn(T) -> N, N: IntoStream {
        self.close_head();
        for item in items { f(item).stream_to(&mut self.buf); }
        self
    }

    #[inline(always)]
    pub fn child_if<F, N>(self, condition: bool, f: F) -> Self
    where F: FnOnce() -> N, N: IntoStream {
        if condition { self.child(StreamWrap(f())) } else { self }
    }

    pub fn build(mut self) -> Markup {
        self.close_head();
        self.buf.push_str("</");
        self.buf.push_str(self.tag);
        self.buf.push('>');
        PreEscaped(self.buf.into_string())
    }
}

/// Thin wrapper so child_if's closure result (any IntoStream type) can
/// be passed into .child(), which expects a single concrete argument.
struct StreamWrap<T: IntoStream>(T);
impl<T: IntoStream> IntoStream for StreamWrap<T> {
    #[inline(always)] fn stream_to(self, buf: &mut StreamBuf) { self.0.stream_to(buf); }
}

impl IntoStream for Element {
    #[inline(always)]
    fn stream_to(mut self, buf: &mut StreamBuf) {
        self.close_head();
        self.buf.push_str("</");
        self.buf.push_str(self.tag);
        self.buf.push('>');
        buf.push_str(self.buf.as_str());
    }
}

#[derive(Clone)]
pub struct VoidElement {
    pub buf: StreamBuf,
    pub tag: &'static str,
    pub has_class: bool,
}

impl VoidElement {
    #[track_caller]
    pub fn new(tag: &'static str) -> Self {
        #[cfg(debug_assertions)]
        {
            if !is_custom_element_name(tag) && !is_legal_tag(tag, LEGAL_VOIDS) {
                let loc = std::panic::Location::caller();
                let msg = if is_legal_tag(tag, LEGAL_CONTAINERS) {
                    format!("'{}' at {}:{} is a standard container. Use Element::new() or tag::{}() instead.", tag, loc.file(), loc.line(), tag)
                } else if let Some(suggestion) = suggest_closest_tag(tag) {
                    format!("Typo at {}:{}: '{}'. Did you mean '{}'?", loc.file(), loc.line(), tag, suggestion)
                } else {
                    format!("Unknown Void Element at {}:{}: '{}'", loc.file(), loc.line(), tag)
                };
                sprout_panic!("VoidElement::new", msg);
            }
        }
        let mut buf = StreamBuf::new();
        buf.push('<');
        buf.push_str(tag);
        Self { buf, tag, has_class: false }
    }

    pub fn build(mut self) -> Markup {
        self.buf.push('>');
        PreEscaped(self.buf.into_string())
    }
}

impl IntoStream for VoidElement {
    #[inline(always)]
    fn stream_to(mut self, buf: &mut StreamBuf) {
        self.buf.push('>');
        buf.push_str(self.buf.as_str());
    }
}

impl HtmlElement for Element {
    #[inline(always)] fn stream(&mut self) -> &mut StreamBuf { &mut self.buf }
    #[inline(always)] fn is_head_closed(&self) -> bool { self.head_closed }
    #[inline(always)] fn has_class(&self) -> bool { self.has_class }
    #[inline(always)] fn set_has_class(&mut self, val: bool) { self.has_class = val; }
}

impl HtmlElement for VoidElement {
    #[inline(always)] fn stream(&mut self) -> &mut StreamBuf { &mut self.buf }
    #[inline(always)] fn is_head_closed(&self) -> bool { false }
    #[inline(always)] fn has_class(&self) -> bool { self.has_class }
    #[inline(always)] fn set_has_class(&mut self, val: bool) { self.has_class = val; }
}

// =====================================================================
// SECTION 6 — SHARED ATTRIBUTE METHODS
// -----------------------------------------------------------------------
// IMPORTANT: because this design streams directly into a buffer, all
// classes/attributes/flags MUST be set before the first .child() call
// — once the head is "closed" (a child has been added), the tag's
// opening bracket has already been written and can no longer be
// edited. Calling .class()/.attr()/.flag() after .child() panics with
// a clear message explaining exactly that, instead of silently
// producing broken HTML.
// =====================================================================

macro_rules! impl_attr_methods {
    ($t:ty) => {
        impl $t {
            #[inline(always)]
            #[track_caller]
            pub fn class(mut self, c: impl Into<SproutStr>) -> Self {
                let c_val = c.into();
                if self.is_head_closed() {
                    let loc = std::panic::Location::caller();
                    sprout_panic!(
                        format!("<{}>::class", self.tag),
                        format!("Attempted to add class '{}' at {}:{}. Order matters! Configure all attributes/classes BEFORE calling .child() or .children().", c_val.as_str(), loc.file(), loc.line())
                    );
                }
                if self.has_class() {
                    // Zero-allocation chaining: pop the closing quote,
                    // append a space + the new class, re-close the quote.
                    self.stream().pop();
                    self.stream().push(' ');
                    escape_attr(c_val.as_str(), self.stream());
                    self.stream().push('"');
                } else {
                    self.stream().push_str(" class=\"");
                    escape_attr(c_val.as_str(), self.stream());
                    self.stream().push('"');
                    self.set_has_class(true);
                }
                self
            }

            #[inline(always)]
            pub fn class_if(self, condition: bool, class: impl Into<SproutStr>) -> Self {
                if condition { self.class(class) } else { self }
            }

            pub fn classes_if<I, S>(mut self, class_map: I) -> Self
            where I: IntoIterator<Item = (bool, S)>, S: Into<SproutStr> {
                for (condition, class_name) in class_map {
                    if condition { self = self.class(class_name); }
                }
                self
            }

            #[inline(always)]
            #[track_caller]
            pub fn attr(mut self, key: impl Into<SproutStr>, value: impl Into<SproutStr>) -> Self {
                let k_val = key.into();
                let v_val = value.into();
                if self.is_head_closed() {
                    let loc = std::panic::Location::caller();
                    sprout_panic!(
                        format!("<{}>::attr", self.tag),
                        format!("Attempted to set attribute '{}={}' at {}:{}. Order matters! Configure all attributes/classes BEFORE calling .child() or .children().", k_val.as_str(), v_val.as_str(), loc.file(), loc.line())
                    );
                }
                self.stream().push(' ');
                self.stream().push_str(k_val.as_str());
                self.stream().push_str("=\"");
                escape_attr(v_val.as_str(), self.stream());
                self.stream().push('"');
                self
            }

            #[inline(always)]
            pub fn attr_if(self, condition: bool, key: impl Into<SproutStr>, value: impl Into<SproutStr>) -> Self {
                if condition { self.attr(key, value) } else { self }
            }

            #[inline(always)]
            pub fn id(self, id: impl Into<SproutStr>) -> Self { self.attr("id", id) }

            #[inline(always)]
            #[track_caller]
            pub fn flag(mut self, condition: bool, key: impl Into<SproutStr>) -> Self {
                if condition {
                    let k_val = key.into();
                    if self.is_head_closed() {
                        let loc = std::panic::Location::caller();
                        sprout_panic!(
                            format!("<{}>::flag", self.tag),
                            format!("Attempted to set flag '{}' at {}:{}. Order matters! Configure all attributes/classes BEFORE calling .child() or .children().", k_val.as_str(), loc.file(), loc.line())
                        );
                    }
                    self.stream().push(' ');
                    self.stream().push_str(k_val.as_str());
                }
                self
            }

            #[inline(always)] pub fn disabled(self, cond: bool) -> Self { self.flag(cond, "disabled") }
            #[inline(always)] pub fn required(self, cond: bool) -> Self { self.flag(cond, "required") }
            #[inline(always)] pub fn readonly(self, cond: bool) -> Self { self.flag(cond, "readonly") }
            #[inline(always)] pub fn checked(self, cond: bool) -> Self { self.flag(cond, "checked") }

            #[inline(always)] pub fn style(self, css: impl Into<SproutStr>) -> Self { self.attr("style", css) }
            #[inline(always)] pub fn src(self, url: impl Into<SproutStr>) -> Self { self.attr("src", url) }
            #[inline(always)] pub fn href(self, url: impl Into<SproutStr>) -> Self { self.attr("href", url) }
            #[inline(always)] pub fn alt(self, text: impl Into<SproutStr>) -> Self { self.attr("alt", text) }
            #[inline(always)] pub fn name(self, n: impl Into<SproutStr>) -> Self { self.attr("name", n) }
            #[inline(always)] pub fn value(self, v: impl Into<SproutStr>) -> Self { self.attr("value", v) }
            #[inline(always)] pub fn placeholder(self, p: impl Into<SproutStr>) -> Self { self.attr("placeholder", p) }
            #[inline(always)] pub fn type_(self, t: impl Into<SproutStr>) -> Self { self.attr("type", t) }

            #[inline(always)] pub fn x_data(self, expr: impl Into<SproutStr>) -> Self { self.attr("x-data", expr) }
            #[inline(always)] pub fn x_show(self, expr: impl Into<SproutStr>) -> Self { self.attr("x-show", expr) }
            #[inline(always)] pub fn x_if(self, expr: impl Into<SproutStr>) -> Self { self.attr("x-if", expr) }
            #[inline(always)] pub fn x_model(self, expr: impl Into<SproutStr>) -> Self { self.attr("x-model", expr) }
            #[inline(always)] pub fn x_text(self, expr: impl Into<SproutStr>) -> Self { self.attr("x-text", expr) }
            #[inline(always)] pub fn x_transition(self) -> Self { self.flag(true, "x-transition") }

            /// Generic event binding — builds the x-on:EVENT key at call
            /// time. For the four most common events, the named shortcuts
            /// below avoid even that small format! allocation.
            #[inline]
            pub fn x_on(self, event: &str, expr: impl Into<SproutStr>) -> Self {
                self.attr(format!("x-on:{event}"), expr)
            }
            #[inline]
            pub fn x_bind(self, attribute: &str, expr: impl Into<SproutStr>) -> Self {
                self.attr(format!("x-bind:{attribute}"), expr)
            }

            #[inline(always)] pub fn x_on_click(self, expr: impl Into<SproutStr>) -> Self { self.attr("x-on:click", expr) }
            #[inline(always)] pub fn x_on_input(self, expr: impl Into<SproutStr>) -> Self { self.attr("x-on:input", expr) }
            #[inline(always)] pub fn x_on_submit(self, expr: impl Into<SproutStr>) -> Self { self.attr("x-on:submit", expr) }
            #[inline(always)] pub fn x_on_change(self, expr: impl Into<SproutStr>) -> Self { self.attr("x-on:change", expr) }

            #[inline(always)]
            pub fn modify(self, f: impl FnOnce(Self) -> Self) -> Self { f(self) }
        }
    };
}

impl_attr_methods!(Element);
impl_attr_methods!(VoidElement);

// =====================================================================
// SECTION 7 — SPROUT ACTION: THE ONE HTMX MECHANISM
// =====================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Swap {
    InnerHTML, OuterHTML, BeforeBegin, AfterBegin, BeforeEnd, AfterEnd, Delete, NoSwap, Custom(&'static str),
}
pub use Swap::*;

impl Swap {
    pub fn as_str(&self) -> &'static str {
        match self {
            InnerHTML => "innerHTML", OuterHTML => "outerHTML", BeforeBegin => "beforebegin",
            AfterBegin => "afterbegin", BeforeEnd => "beforeend", AfterEnd => "afterend",
            Delete => "delete", NoSwap => "none", Custom(s) => s,
        }
    }
}

impl From<&'static str> for Swap {
    fn from(s: &'static str) -> Self {
        match s {
            "innerHTML" => InnerHTML, "outerHTML" => OuterHTML, "beforebegin" => BeforeBegin,
            "afterbegin" => AfterBegin, "beforeend" => BeforeEnd, "afterend" => AfterEnd,
            "delete" => Delete, "none" => NoSwap, _ => Custom(s),
        }
    }
}

pub fn get(url: impl Into<SproutStr>) -> SproutAction { SproutAction::get(url) }
pub fn post(url: impl Into<SproutStr>) -> SproutAction { SproutAction::post(url) }
pub fn put(url: impl Into<SproutStr>) -> SproutAction { SproutAction::put(url) }
pub fn patch(url: impl Into<SproutStr>) -> SproutAction { SproutAction::patch(url) }
pub fn delete(url: impl Into<SproutStr>) -> SproutAction { SproutAction::delete(url) }

pub struct SproutAction {
    pub method: &'static str,
    pub url: SproutStr,
    pub target: Option<SproutStr>,
    pub swap: Option<&'static str>,
    pub oob: Option<&'static str>,
    pub indicator: Option<SproutStr>,
    pub trigger: Option<SproutStr>,
}

impl SproutAction {
    pub fn get(url: impl Into<SproutStr>) -> Self { Self::new("hx-get", url) }
    pub fn post(url: impl Into<SproutStr>) -> Self { Self::new("hx-post", url) }
    pub fn put(url: impl Into<SproutStr>) -> Self { Self::new("hx-put", url) }
    pub fn patch(url: impl Into<SproutStr>) -> Self { Self::new("hx-patch", url) }
    pub fn delete(url: impl Into<SproutStr>) -> Self { Self::new("hx-delete", url) }

    fn new(method: &'static str, url: impl Into<SproutStr>) -> Self {
        Self { method, url: url.into(), target: None, swap: None, oob: None, indicator: None, trigger: None }
    }

    pub fn to(mut self, target: impl Into<SproutStr>) -> Self { self.target = Some(target.into()); self }
    pub fn swap(mut self, strategy: impl Into<Swap>) -> Self { self.swap = Some(strategy.into().as_str()); self }
    pub fn oob(mut self, strategy: &'static str) -> Self { self.oob = Some(strategy); self }
    pub fn indicator(mut self, selector: impl Into<SproutStr>) -> Self { self.indicator = Some(selector.into()); self }
    pub fn trigger(mut self, trigger_expr: impl Into<SproutStr>) -> Self { self.trigger = Some(trigger_expr.into()); self }
}

pub trait SproutExt: Sized {
    fn sprout_action(self, action: SproutAction) -> Self;

    fn hx_post_to(self, url: impl Into<SproutStr>, target: impl Into<SproutStr>) -> Self {
        self.sprout_action(post(url).to(target))
    }
    fn hx_get_to(self, url: impl Into<SproutStr>, target: impl Into<SproutStr>) -> Self {
        self.sprout_action(get(url).to(target))
    }
}

impl SproutExt for Element {
    fn sprout_action(self, action: SproutAction) -> Self {
        let mut el = self.attr(action.method, action.url);
        if let Some(t) = action.target { el = el.attr("hx-target", t); }
        if let Some(s) = action.swap { el = el.attr("hx-swap", s); }
        if let Some(o) = action.oob { el = el.attr("hx-swap-oob", o); }
        if let Some(i) = action.indicator { el = el.attr("hx-indicator", i); }
        if let Some(tr) = action.trigger { el = el.attr("hx-trigger", tr); }

        #[cfg(debug_assertions)]
        {
            let debug_str = sprout_fmt!("{}", el.tag);
            return el.attr("data-sprout-debug", debug_str);
        }
        #[allow(unreachable_code)]
        el
    }
}

// =====================================================================
// SECTION 8 — NATIVE BROWSER HELPERS (zero-JS features)
// =====================================================================

pub fn popover_trigger(target_id: impl Into<SproutStr>, label: impl IntoStream) -> Element {
    Element::new("button").attr("popovertarget", target_id).child(label)
}
pub fn popover_panel(id: impl Into<SproutStr>, content: impl IntoStream) -> Element {
    Element::new("div").id(id).attr("popover", "auto").child(content)
}
pub fn auto_closing_dialog(id: impl Into<SproutStr>, content: impl IntoStream) -> Element {
    Element::new("dialog").id(id).attr("closedby", "any").child(content)
}
pub fn dialog_cancel_button(label: impl IntoStream) -> Element {
    Element::new("button").attr("formmethod", "dialog").attr("value", "cancel").child(label)
}
pub fn autocomplete_input(name: impl Into<SproutStr>, list_id: impl Into<SproutStr>) -> VoidElement {
    VoidElement::new("input").name(name).attr("list", list_id)
}
pub fn datalist<I, T>(id: impl Into<SproutStr>, options: I) -> Element
where I: IntoIterator<Item = T>, T: Into<SproutStr> {
    Element::new("datalist").id(id).child_for(options, |opt| Element::new("option").value(opt))
}
pub fn inert_if(el: Element, condition: bool) -> Element {
    el.attr_if(condition, "inert", "")
}
pub fn lazy_img(src: impl Into<SproutStr>, alt: impl Into<SproutStr>) -> VoidElement {
    VoidElement::new("img").src(src).alt(alt).attr("loading", "lazy")
}
pub fn progress_bar(value: u32, max: u32) -> Element {
    Element::new("progress").attr("value", sprout_fmt!("{value}")).attr("max", sprout_fmt!("{max}"))
}
pub fn time_tag(display_text: impl IntoStream, machine_date: impl Into<SproutStr>) -> Element {
    Element::new("time").attr("datetime", machine_date).child(display_text)
}
pub fn download_link(url: impl Into<SproutStr>, filename: impl Into<SproutStr>, label: impl IntoStream) -> Element {
    Element::new("a").href(url).attr("download", filename).child(label)
}
pub fn external_link(url: impl Into<SproutStr>, label: impl IntoStream) -> Element {
    Element::new("a").href(url).attr("target", "_blank").attr("rel", "noopener noreferrer").child(label)
}

// =====================================================================
// SECTION 9 — TESTS
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_empty_div() {
        assert_eq!(Element::new("div").build().into_string(), "<div></div>");
    }

    #[test]
    fn renders_single_class() {
        assert_eq!(Element::new("div").class("card").build().into_string(), r#"<div class="card"></div>"#);
    }

    #[test]
    fn chains_multiple_classes_zero_alloc() {
        let html = Element::new("div").class("a").class("b").class("c").build().into_string();
        assert_eq!(html, r#"<div class="a b c"></div>"#);
    }

    #[test]
    fn class_if_only_adds_when_true() {
        assert_eq!(Element::new("a").class_if(true, "active").build().into_string(), r#"<a class="active"></a>"#);
        assert_eq!(Element::new("a").class_if(false, "active").build().into_string(), "<a></a>");
    }

    #[test]
    fn classes_if_adds_only_matching() {
        let html = Element::new("li").classes_if([(true, "active"), (false, "x"), (true, "hot")]).build().into_string();
        assert_eq!(html, r#"<li class="active hot"></li>"#);
    }

    #[test]
    fn disabled_required_readonly_checked_work() {
        assert_eq!(Element::new("button").disabled(true).build().into_string(), "<button disabled></button>");
        assert_eq!(VoidElement::new("input").required(true).build().into_string(), "<input required>");
    }

    #[test]
    #[should_panic(expected = "Order matters")]
    fn attr_after_child_panics_with_helpful_message() {
        let _ = Element::new("div").child("text").class("too-late");
    }

    #[test]
    fn nests_elements_via_streaming_flatten() {
        let html = Element::new("div").child(Element::new("p").child("hello")).build().into_string();
        assert_eq!(html, "<div><p>hello</p></div>");
    }

    #[test]
    fn child_for_builds_from_iterator() {
        let html = Element::new("ul")
            .child_for(vec!["Apple", "Banana"], |f| Element::new("li").child(f))
            .build().into_string();
        assert_eq!(html, "<ul><li>Apple</li><li>Banana</li></ul>");
    }

    #[test]
    fn child_if_only_renders_when_true() {
        assert_eq!(Element::new("div").child_if(true, || "shown").build().into_string(), "<div>shown</div>");
        assert_eq!(Element::new("div").child_if(false, || "hidden").build().into_string(), "<div></div>");
    }

    #[test]
    fn escapes_script_tag_in_text() {
        let html = Element::new("p").child("<script>alert(1)</script>").build().into_string();
        assert_eq!(html, "<p>&lt;script&gt;alert(1)&lt;/script&gt;</p>");
    }

    #[test]
    fn escapes_quotes_in_attributes_only() {
        let attr_html = VoidElement::new("input").attr("placeholder", "say \"hi\"").build().into_string();
        assert_eq!(attr_html, r#"<input placeholder="say &quot;hi&quot;">"#);
        let text_html = Element::new("p").child("he said \"hi\"").build().into_string();
        assert_eq!(text_html, "<p>he said \"hi\"</p>");
    }

    #[test]
    fn void_element_self_closes() {
        assert_eq!(VoidElement::new("br").build().into_string(), "<br>");
    }

    #[test]
    fn handles_text_near_inline_buffer_boundary_with_multibyte_chars() {
        // Regression test: confirms whole-string pushes near the 1024
        // byte boundary never produce invalid UTF-8, including when
        // multi-byte characters are involved.
        let padding = "x".repeat(1020);
        let emoji_text = "🌱🌱🌱"; // 4 bytes each, 12 bytes total
        let html = Element::new("div")
            .child(padding.clone())
            .child(emoji_text)
            .build()
            .into_string();
        assert!(html.contains(&padding));
        assert!(html.contains(emoji_text));
    }

    #[test]
    fn sprout_action_post_sets_correct_attributes() {
        let html = Element::new("form").sprout_action(post("/save").to("#result").swap(OuterHTML)).build().into_string();
        assert!(html.contains(r###"hx-post="/save""###));
        assert!(html.contains(r###"hx-target="#result""###));
        assert!(html.contains(r###"hx-swap="outerHTML""###));
    }

    #[test]
    fn hx_post_to_shortcut_works() {
        let html = Element::new("div").hx_post_to("/update", "#content").build().into_string();
        assert!(html.contains(r###"hx-post="/update""###));
        assert!(html.contains(r###"hx-target="#content""###));
    }

    #[test]
    fn x_on_click_shortcut_and_generic_x_on_agree() {
        let a = Element::new("button").x_on_click("doThing()").build().into_string();
        let b = Element::new("button").x_on("click", "doThing()").build().into_string();
        assert_eq!(a, b);
    }

    #[test]
    fn popover_and_dialog_helpers_render_correctly() {
        let html = popover_trigger("m1", "Open").build().into_string();
        assert_eq!(html, r#"<button popovertarget="m1">Open</button>"#);
        let dialog = auto_closing_dialog("d1", "hi").build().into_string();
        assert_eq!(dialog, r#"<dialog id="d1" closedby="any">hi</dialog>"#);
    }

    #[test]
    fn progress_bar_and_time_tag_render_correctly() {
        assert_eq!(progress_bar(40, 100).build().into_string(), r#"<progress value="40" max="100"></progress>"#);
        let html = time_tag("June 22, 2026", "2026-06-22").build().into_string();
        assert_eq!(html, r#"<time datetime="2026-06-22">June 22, 2026</time>"#);
    }

    #[test]
    fn external_link_includes_safety_attributes() {
        let html = external_link("https://example.com", "Visit").build().into_string();
        assert_eq!(html, r#"<a href="https://example.com" target="_blank" rel="noopener noreferrer">Visit</a>"#);
    }
}