use maud::{Markup, Render, PreEscaped};
use std::borrow::Cow;

#[derive(Clone)]
pub enum Node {
    Element(Box<Element>),
    Void(Box<VoidElement>),
    Text(String),
}

impl From<Element> for Node { fn from(e: Element) -> Self { Node::Element(Box::new(e)) } }
impl From<VoidElement> for Node { fn from(e: VoidElement) -> Self { Node::Void(Box::new(e)) } }
impl From<&str> for Node { fn from(s: &str) -> Self { Node::Text(s.to_string()) } }
impl From<String> for Node { fn from(s: String) -> Self { Node::Text(s) } }

#[derive(Clone, Default)]
pub struct Attrs {
    pub classes: Vec<String>,
    pub id: Option<String>,
    pub pairs: Vec<(Cow<'static, str>, String)>,
}

impl Attrs {
    fn render_to(&self, w: &mut String) {
        if !self.classes.is_empty() {
            w.push_str(" class=\"");
            escape_attr(&self.classes.join(" "), w);
            w.push('"');
        }
        if let Some(ref id) = self.id {
            w.push_str(" id=\"");
            escape_attr(id, w);
            w.push('"');
        }
        for (k, v) in &self.pairs {
            w.push(' ');
            w.push_str(&k);
            w.push_str("=\"");
            escape_attr(v, w);
            w.push('"');
        }
    }
}

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

macro_rules! impl_attr_methods {
    ($t:ty) => {
        impl $t {
            pub fn class(mut self, c: impl Into<String>) -> Self {
                let c = c.into();
                if !self.attrs.classes.contains(&c) {
                    self.attrs.classes.push(c);
                }
                self
            }
            pub fn id(mut self, id: impl Into<String>) -> Self {
                self.attrs.id = Some(id.into());
                self
            }
            pub fn attr(mut self, key: impl Into<Cow<'static, str>>, value: impl Into<String>) -> Self {
                self.attrs.pairs.push((key.into(), value.into()));
                self
            }
            // --- Added Style Sugar ---
            pub fn style(self, css: impl Into<String>) -> Self { self.attr("style", css) }

            pub fn src(self, url: impl Into<String>) -> Self { self.attr("src", url) }
            pub fn href(self, url: impl Into<String>) -> Self { self.attr("href", url) }
            pub fn alt(self, text: impl Into<String>) -> Self { self.attr("alt", text) }
            pub fn name(self, n: impl Into<String>) -> Self { self.attr("name", n) }
            pub fn value(self, v: impl Into<String>) -> Self { self.attr("value", v) }
            pub fn placeholder(self, p: impl Into<String>) -> Self { self.attr("placeholder", p) }
            pub fn type_(self, t: impl Into<String>) -> Self { self.attr("type", t) }

            pub fn hx_get(self, url: &'static str) -> Self { self.attr("hx-get", url) }
            pub fn hx_post(self, url: &'static str) -> Self { self.attr("hx-post", url) }
            pub fn hx_target(self, target: &'static str) -> Self { self.attr("hx-target", target) }
            pub fn hx_swap(self, mode: &'static str) -> Self { self.attr("hx-swap", mode) }
            pub fn hx_trigger(self, trigger: &'static str) -> Self { self.attr("hx-trigger", trigger) }

            pub fn x_data(self, expr: impl Into<String>) -> Self { self.attr("x-data", expr) }
            pub fn x_show(self, expr: impl Into<String>) -> Self { self.attr("x-show", expr) }
            pub fn x_if(self, expr: impl Into<String>) -> Self { self.attr("x-if", expr) }
            pub fn x_model(self, expr: impl Into<String>) -> Self { self.attr("x-model", expr) }
            pub fn x_text(self, expr: impl Into<String>) -> Self { self.attr("x-text", expr) }
            
            pub fn x_on(self, event: &str, expr: impl Into<String>) -> Self {
                let key = format!("x-on:{event}");
                self.attr(key, expr)
            }
            pub fn x_bind(self, attribute: &str, expr: impl Into<String>) -> Self {
                let key = format!("x-bind:{attribute}");
                self.attr(key, expr)
            }
            pub fn x_transition(self) -> Self { self.attr("x-transition", "") }
        }
    };
}

impl_attr_methods!(Element);
impl_attr_methods!(VoidElement);

impl Element {
    pub fn new(tag: &'static str) -> Self {
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
    
    
    // --- NEW: build one child per item, no .map().collect() needed ---
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

    // --- NEW: only add a child if a condition is true ---
    pub fn child_if<F, N>(self, condition: bool, f: F) -> Self
    where
        F: FnOnce() -> N,
        N: Into<Node>,
    {
        if condition {
            self.child(f())
        } else {
            self
        }
    }
    
    

    pub fn build(self) -> Markup {
        let mut buf = String::with_capacity(256);
        self.render_to(&mut buf);
        PreEscaped(buf)
    }
}

impl VoidElement {
    pub fn new(tag: &'static str) -> Self {
        Self { tag, attrs: Attrs::default() }
    }

    pub fn build(self) -> Markup {
        let mut buf = String::with_capacity(64);
        self.render_to(&mut buf);
        PreEscaped(buf)
    }
}

fn escape_text(s: &str, out: &mut String) {
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
}

fn escape_attr(s: &str, out: &mut String) {
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
}

impl Render for Node {
    fn render_to(&self, w: &mut String) {
        match self {
            Node::Text(t) => escape_text(t, w),
            Node::Element(e) => e.render_to(w),
            Node::Void(v) => v.render_to(w),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_empty_div() {
        let html = Element::new("div").build().into_string();
        assert_eq!(html, "<div></div>");
    }

    #[test]
    fn renders_single_class() {
        let html = Element::new("div").class("card").build().into_string();
        assert_eq!(html, r#"<div class="card"></div>"#);
    }

    #[test]
    fn renders_style_attribute() {
        let html = Element::new("div").style("color: red;").build().into_string();
        assert_eq!(html, r#"<div style="color: red;"></div>"#);
    }

    #[test]
    fn duplicate_class_is_not_repeated() {
        let html = Element::new("div").class("card").class("card").build().into_string();
        assert_eq!(html, r#"<div class="card"></div>"#);
    }

    #[test]
    fn renders_multiple_distinct_classes_joined_with_space() {
        let html = Element::new("div")
            .class("card")
            .class("featured")
            .build()
            .into_string();
        assert_eq!(html, r#"<div class="card featured"></div>"#);
    }

    #[test]
    fn renders_id() {
        let html = Element::new("div").id("post-list").build().into_string();
        assert_eq!(html, r#"<div id="post-list"></div>"#);
    }

    #[test]
    fn renders_custom_attr() {
        let html = VoidElement::new("input")
            .attr("placeholder", "Title")
            .build()
            .into_string();
        assert_eq!(html, r#"<input placeholder="Title">"#);
    }

    #[test]
    fn placeholder_sugar_matches_attr() {
        let html = VoidElement::new("input")
            .placeholder("Title")
            .build()
            .into_string();
        assert_eq!(html, r#"<input placeholder="Title">"#);
    }

    #[test]
    fn src_and_alt_sugar_on_img() {
        let html = VoidElement::new("img")
            .src("/me.png")
            .alt("profile photo")
            .build()
            .into_string();
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
        let html = Element::new("div")
            .class("a")
            .id("b")
            .attr("data-x", "1")
            .build()
            .into_string();
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
        let html = Element::new("button")
            .x_on("click", "open = !open")
            .child("Toggle")
            .build()
            .into_string();
        assert_eq!(html, r#"<button x-on:click="open = !open">Toggle</button>"#);
    }

    #[test]
    fn x_bind_builds_attribute_specific_key() {
        let html = Element::new("div")
            .x_bind("class", "isOpen ? 'show' : ''")
            .build()
            .into_string();
        assert_eq!(html, r#"<div x-bind:class="isOpen ? 'show' : ''"></div>"#);
    }

    #[test]
    fn x_transition_sets_empty_attribute() {
        let html = Element::new("div").x_transition().build().into_string();
        assert_eq!(html, r#"<div x-transition=""></div>"#);
    }

    #[test]
    fn alpine_value_with_quotes_and_braces_is_escaped() {
        let html = Element::new("div")
            .x_data("{ count: 0, label: 'hi' }")
            .build()
            .into_string();
        assert_eq!(html, r#"<div x-data="{ count: 0, label: 'hi' }"></div>"#);
    }

    #[test]
    fn escapes_ampersand_in_text() {
        let html = Element::new("p").child("Rust & Axum").build().into_string();
        assert_eq!(html, "<p>Rust &amp; Axum</p>");
    }

    #[test]
    fn escapes_script_tag_in_text() {
        let html = Element::new("p")
            .child("<script>alert(1)</script>")
            .build()
            .into_string();
        assert_eq!(html, "<p>&lt;script&gt;alert(1)&lt;/script&gt;</p>");
    }

    #[test]
    fn does_not_escape_quotes_in_text_content() {
        let html = Element::new("p").child("he said \"hi\"").build().into_string();
        assert_eq!(html, "<p>he said \"hi\"</p>");
    }

    #[test]
    fn escapes_quote_in_attribute_value() {
        let html = VoidElement::new("input")
            .attr("placeholder", "say \"hi\"")
            .build()
            .into_string();
        assert_eq!(html, r#"<input placeholder="say &quot;hi&quot;">"#);
    }

    #[test]
    fn escapes_angle_brackets_in_attribute_value() {
        let html = VoidElement::new("input")
            .attr("data-x", "<b>")
            .build()
            .into_string();
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
        let html = VoidElement::new("input")
            .attr("type", "text")
            .attr("name", "title")
            .build()
            .into_string();
        assert_eq!(html, r#"<input type="text" name="title">"#);
    }

    #[test]
    fn img_void_element_with_class() {
        let html = VoidElement::new("img")
            .class("avatar")
            .attr("src", "/me.png")
            .build()
            .into_string();
        assert_eq!(html, r#"<img class="avatar" src="/me.png">"#);
    }

    #[test]
    fn nests_element_inside_element() {
        let html = Element::new("div")
            .child(Element::new("p").child("hello"))
            .build()
            .into_string();
        assert_eq!(html, "<div><p>hello</p></div>");
    }

    #[test]
    fn nests_void_element_inside_element() {
        let html = Element::new("div")
            .child(VoidElement::new("br"))
            .build()
            .into_string();
        assert_eq!(html, "<div><br></div>");
    }

    #[test]
    fn children_preserves_order() {
        let html = Element::new("ul")
            .children(vec![
                Element::new("li").child("first"),
                Element::new("li").child("second"),
                Element::new("li").child("third"),
            ])
            .build()
            .into_string();
        assert_eq!(html, "<ul><li>first</li><li>second</li><li>third</li></ul>");
    }

    #[test]
    fn mixes_text_and_element_children_in_order() {
        let html = Element::new("p")
            .child("Hello, ")
            .child(Element::new("strong").child("world"))
            .child("!")
            .build()
            .into_string();
        assert_eq!(html, "<p>Hello, <strong>world</strong>!</p>");
    }

    #[test]
    fn deeply_nested_structure_renders_correctly() {
        let html = Element::new("div")
            .class("card")
            .child(Element::new("h2").child("Title"))
            .child(
                Element::new("div")
                    .class("body")
                    .child(Element::new("p").child("Nested content")),
            )
            .build()
            .into_string();
        assert_eq!(
            html,
            r#"<div class="card"><h2>Title</h2><div class="body"><p>Nested content</p></div></div>"#
        );
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
    fn id_is_overwritten_by_subsequent_calls() {
        let html = Element::new("div")
            .id("first-id")
            .id("final-id")
            .build()
            .into_string();
        assert_eq!(html, r#"<div id="final-id"></div>"#);
    }

    #[test]
    fn accepts_dynamic_children_from_iterators() {
        let items: Vec<String> = (1..=3).map(|i| format!("Item {}", i)).collect();
        
        let html = Element::new("ul")
            .children(items.into_iter().map(|s| Element::new("li").child(s)))
            .build()
            .into_string();
            
        assert_eq!(html, "<ul><li>Item 1</li><li>Item 2</li><li>Item 3</li></ul>");
    }

    #[test]
    fn handles_empty_string_attributes_gracefully() {
        let html = VoidElement::new("input")
            .class("")
            .id("")
            .attr("disabled", "")
            .attr("required", "")
            .build()
            .into_string();
        
        assert_eq!(html, r#"<input class="" id="" disabled="" required="">"#);
    }

    #[test]
    fn strict_escaping_differences_between_text_and_attributes() {
        let evil_string = r#"<script>alert("hax & stuff")</script>"#;
        
        let html = Element::new("div")
            .attr("data-payload", evil_string)
            .child(evil_string)
            .build()
            .into_string();
            
        assert_eq!(
            html,
            r#"<div data-payload="&lt;script&gt;alert(&quot;hax &amp; stuff&quot;)&lt;/script&gt;">&lt;script&gt;alert("hax &amp; stuff")&lt;/script&gt;</div>"#
        );
    }

    #[test]
    fn cow_attr_accepts_owned_strings_safely() {
        let dynamic_key = format!("data-{}", "user-id");
        let html = Element::new("div")
            .attr(dynamic_key, "123")
            .build()
            .into_string();
            
        assert_eq!(html, r#"<div data-user-id="123"></div>"#);
    }

    #[test]
    fn complex_alpine_expression_escaping() {
        let html = Element::new("button")
            .x_data(r#"{ user: { name: "John & Jane", active: true } }"#)
            .x_on("click", r#"console.log('Clicked "Submit" <here>');"#)
            .build()
            .into_string();
            
        assert_eq!(
            html,
            r#"<button x-data="{ user: { name: &quot;John &amp; Jane&quot;, active: true } }" x-on:click="console.log('Clicked &quot;Submit&quot; &lt;here&gt;');"></button>"#
        );
    }

    #[test]
    fn massive_tree_depth_does_not_panic() {
        let deep_tree = (1..=10).fold(Element::new("span").child("Deepest"), |acc, i| {
            Element::new("div").class(format!("level-{}", i)).child(acc)
        });
        
        let html = Element::new("div")
            .id("root")
            .child(deep_tree)
            .build()
            .into_string();
            
        assert!(html.starts_with(r#"<div id="root"><div class="level-10"><div class="level-9""#));
        assert!(html.contains("<span>Deepest</span>"));
        assert!(html.ends_with("</div></div></div>"));
    }

    #[test]
    fn iterators_can_mix_nodes_and_text() {
        let dynamic_content: Vec<Node> = vec![
            "Prefix text - ".into(),
            Element::new("strong").child("Bold text").into(),
            " - Suffix text".into(),
        ];
        
        let html = Element::new("p")
            .children(dynamic_content)
            .build()
            .into_string();
            
        assert_eq!(html, "<p>Prefix text - <strong>Bold text</strong> - Suffix text</p>");
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
}
