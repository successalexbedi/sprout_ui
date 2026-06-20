use maud::html;
use sprout_ui_tags as tag;
use sprout_ui_components as ui;

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
        );

    let page = html! { (tree) };

    println!("{}", page.into_string());
}
