use sprout_ui_core::{Element, VoidElement};
use sprout_ui_tags as tag;

// --- Popover (dropdown menus, no JS) ---

pub fn popover_trigger(target_id: &str, label: &str) -> Element {
    tag::button()
        .attr("popovertarget", target_id.to_string())
        .child(label.to_string())
}

pub fn popover_panel(id: &str, content: Element) -> Element {
    tag::div()
        .id(id.to_string())
        .attr("popover", "auto")
        .child(content)
}

// --- Dialog helpers ---

pub fn auto_closing_dialog(id: &str, content: Element) -> Element {
    tag::dialog()
        .id(id.to_string())
        .attr("closedby", "any")
        .child(content)
}

pub fn dialog_cancel_button(label: &str) -> Element {
    tag::button()
        .attr("formmethod", "dialog")
        .attr("value", "cancel")
        .child(label.to_string())
}

// --- Autocomplete ---

pub fn autocomplete_input(name: &str, list_id: &str) -> VoidElement {
    tag::input()
        .name(name.to_string())
        .attr("list", list_id.to_string())
}

pub fn datalist(id: &str, options: &[&str]) -> Element {
    tag::datalist()
        .id(id.to_string())
        .child_for(options, |opt| {
            tag::option().attr("value", opt.to_string())
        })
}

// --- Background interaction control ---

pub fn inert_if(el: Element, condition: bool) -> Element {
    el.attr_if(condition, "inert", "")
}

// --- Performance ---

pub fn lazy_img(src: &str, alt: &str) -> VoidElement {
    tag::img()
        .src(src.to_string())
        .alt(alt.to_string())
        .attr("loading", "lazy")
}

// --- Progress / rank bars ---

pub fn progress_bar(value: u32, max: u32) -> Element {
    tag::progress()
        .attr("value", value.to_string())
        .attr("max", max.to_string())
}

// --- Semantic dates ---

pub fn time_tag(display_text: &str, machine_date: &str) -> Element {
    tag::time()
        .attr("datetime", machine_date.to_string())
        .child(display_text.to_string())
}

// --- Files and links ---

pub fn download_link(url: &str, filename: &str, label: &str) -> Element {
    tag::a()
        .href(url.to_string())
        .attr("download", filename.to_string())
        .child(label.to_string())
}

pub fn external_link(url: &str, label: &str) -> Element {
    tag::a()
        .href(url.to_string())
        .attr("target", "_blank")
        .attr("rel", "noopener noreferrer")
        .child(label.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popover_trigger_builds_correct_button() {
        let html = popover_trigger("menu1", "Options").build().into_string();
        assert_eq!(html, r#"<button popovertarget="menu1">Options</button>"#);
    }

    #[test]
    fn popover_panel_builds_correct_div() {
        let html = popover_panel("menu1", Element::new("p").child("Item")).build().into_string();
        assert_eq!(html, r#"<div id="menu1" popover="auto"><p>Item</p></div>"#);
    }

    #[test]
    fn auto_closing_dialog_sets_closedby() {
        let html = auto_closing_dialog("myDialog", Element::new("p").child("Hi")).build().into_string();
        assert_eq!(html, r#"<dialog id="myDialog" closedby="any"><p>Hi</p></dialog>"#);
    }

    #[test]
    fn dialog_cancel_button_builds_correctly() {
        let html = dialog_cancel_button("Cancel").build().into_string();
        assert_eq!(html, r#"<button formmethod="dialog" value="cancel">Cancel</button>"#);
    }

    #[test]
    fn autocomplete_input_links_to_datalist() {
        let html = autocomplete_input("genre", "genreOptions").build().into_string();
        assert_eq!(html, r#"<input name="genre" list="genreOptions">"#);
    }

    #[test]
    fn datalist_builds_options() {
        let html = datalist("genreOptions", &["Fantasy", "Sci-Fi"]).build().into_string();
        assert_eq!(html, r#"<datalist id="genreOptions"><option value="Fantasy"></option><option value="Sci-Fi"></option></datalist>"#);
    }

    #[test]
    fn inert_if_applies_when_true() {
        let html = inert_if(Element::new("div"), true).build().into_string();
        assert_eq!(html, r#"<div inert=""></div>"#);
    }

    #[test]
    fn lazy_img_sets_loading_attribute() {
        let html = lazy_img("/cover.png", "Book cover").build().into_string();
        assert_eq!(html, r#"<img src="/cover.png" alt="Book cover" loading="lazy">"#);
    }

    #[test]
    fn progress_bar_sets_value_and_max() {
        let html = progress_bar(40, 100).build().into_string();
        assert_eq!(html, r#"<progress value="40" max="100"></progress>"#);
    }

    #[test]
    fn time_tag_sets_datetime() {
        let html = time_tag("June 22, 2026", "2026-06-22").build().into_string();
        assert_eq!(html, r#"<time datetime="2026-06-22">June 22, 2026</time>"#);
    }

    #[test]
    fn download_link_sets_download_attribute() {
        let html = download_link("/contract.pdf", "fictreon-contract.pdf", "Download Contract").build().into_string();
        assert_eq!(html, r#"<a href="/contract.pdf" download="fictreon-contract.pdf">Download Contract</a>"#);
    }

    #[test]
    fn external_link_sets_safe_target_attributes() {
        let html = external_link("https://example.com", "Visit").build().into_string();
        assert_eq!(html, r#"<a href="https://example.com" target="_blank" rel="noopener noreferrer">Visit</a>"#);
    }
}
