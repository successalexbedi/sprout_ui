use maud::html;
use sprout_ui_core::Element;
use sprout_ui_tags as tag;
use sprout_ui_components as ui;

pub fn popover_panel(id: &str, content: Element) -> Element {
    tag::div()
        .id(id.to_string())
        .attr("popover", "auto")
        .child(content)
}

/// Extracts tags and prints them as <tag> </tag>
fn print_tag_pairs(html: String) -> String {
    let mut result = String::new();
    let mut i = 0;
    let chars: Vec<char> = html.chars().collect();

    while i < chars.len() {
        if chars[i] == '<' && chars[i+1] != '/' {
            // Found opening tag
            let start = i;
            while i < chars.len() && chars[i] != '>' { i += 1; }
            i += 1;
            let open_tag = String::from_iter(&chars[start..i]);
            let tag_name = open_tag.trim_matches(|c| c == '<' || c == '>')
                                   .split_whitespace().next().unwrap_or("");

            // Find closing tag
            result.push_str(&format!("<{}> </{}>\n", tag_name, tag_name));
        }
        i += 1;
    }
    result
}

fn main() {
    let tree = tag::div()
        .class("layout")
        .id("app-root")
        .child(ui::navbar())
        .child(
            tag::section().children(vec![
                ui::card("Hello", "SproutUI works perfectly"),
                ui::card("Second", "True AST, zero noise architecture").class("featured"),
            ])
        )
        .child(popover_panel("user-menu", tag::p().child("Menu items here")));

    let raw_html = html! { (tree) }.into_string();
    println!("{}", print_tag_pairs(raw_html));
}
