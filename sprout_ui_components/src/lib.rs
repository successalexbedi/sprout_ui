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