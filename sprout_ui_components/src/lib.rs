// Remove SproutExt from the import list
use sprout_ui_core::{Element, SproutStr, post}; 
use sprout_ui_tags as tag;
use sprout_ui_core::SproutExt;



pub fn navbar() -> Element {
    tag::nav()
        .class("navbar")
        .child(tag::h1().child("SproutUI"))
}

pub fn card(title: impl Into<SproutStr>, body: impl Into<SproutStr>) -> Element {
    tag::div()
        .class("card")
        .child(tag::h1().child(title.into()))
        .child(tag::p().child(body.into()))
        .child(
            tag::button()
                .class("btn-primary")
                .sprout_action(post("/like").to("#likes"))
                .child("Like")
        )
}

// ... rest of your tests remain unchanged

// =====================================================================
// TESTS
// =====================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navbar_renders_correctly() {
        let html = navbar().build().into_string();
        assert_eq!(html, r#"<nav class="navbar"><h1>SproutUI</h1></nav>"#);
    }

    #[test]
    fn card_renders_with_like_action() {
        let html = card("Hello", "Body text").build().into_string();
        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("<p>Body text</p>"));
        assert!(html.contains(r##"hx-post="/like""##));
        assert!(html.contains(r##"hx-target="#likes""##));
    }
}
