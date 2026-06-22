use sprout_ui_core::{Element, VoidElement};
use std::borrow::Cow;

// ==========================================
// 5. SYNTAX DEFINITIONS & SHORTCUT HELPERS
// ==========================================

macro_rules! declare_tags {
    (
        container: { $($c:ident),* $(,)? }
        void: { $($v:ident),* $(,)? }
    ) => {
        $( 
            #[track_caller]
            pub fn $c() -> Element { Element::new(stringify!($c)) } 
        )*
        $( 
            #[track_caller]
            pub fn $v() -> VoidElement { VoidElement::new(stringify!($v)) } 
        )*
    };
}

declare_tags! {
    container: {
        div, section, nav, main, header, footer, aside, article, address, details, summary, dialog,
        h1, h2, h3, h4, h5, h6, p, span, a, strong, em, small, blockquote, pre, code, kbd, sub, sup, mark, time, del, ins,
        ul, ol, li, dl, dt, dd,
        form, label, textarea, select, option, optgroup, button, fieldset, legend, output, progress, meter,
        table, thead, tbody, tfoot, tr, th, td, caption, colgroup,
        video, audio, iframe, canvas, picture, map, object,
        html, head, body, title, style, script, noscript,
        svg, datalist, 
    }
    void: {
        br, hr, img, input, link, meta, area, base, col, embed, param, source, track, wbr, 
        path, 
    }
}

pub fn stylesheet(href: impl Into<Cow<'static, str>>) -> VoidElement {
    link().attr("rel", "stylesheet").href(href)
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macro_generates_correct_container_tags() {
        // Test a few samples from the container list
        let div_html = div().child("test").build().into_string();
        let h1_html = h1().child("title").build().into_string();
        let span_html = span().class("text").build().into_string();

        assert_eq!(div_html, "<div>test</div>");
        assert_eq!(h1_html, "<h1>title</h1>");
        assert_eq!(span_html, r#"<span class="text"></span>"#);
    }

    #[test]
    fn macro_generates_correct_void_tags() {
        // Test a few samples from the void list
        let br_html = br().build().into_string();
        let img_html = img().src("test.jpg").build().into_string();
        let hr_html = hr().class("divider").build().into_string();

        assert_eq!(br_html, "<br>");
        assert_eq!(img_html, r#"<img src="test.jpg">"#);
        assert_eq!(hr_html, r#"<hr class="divider">"#);
    }

    #[test]
    fn stylesheet_helper_generates_correct_link_tag() {
        let html = stylesheet("style.css").build().into_string();
        assert_eq!(html, r#"<link rel="stylesheet" href="style.css">"#);
    }

    #[test]
    fn can_nest_macro_generated_tags() {
        // Verifying the integration between generated functions
        let html = div()
            .child(
                ul().child(li().child("item 1"))
            )
            .build()
            .into_string();
        
        assert_eq!(html, "<div><ul><li>item 1</li></ul></div>");
    }

    #[test]
    fn svg_and_path_integration() {
        let html = svg()
            .attr("viewBox", "0 0 100 100")
            .child(path().attr("d", "M10 10H90V90H10Z"))
            .build()
            .into_string();
            
        assert_eq!(
            html, 
            r#"<svg viewBox="0 0 100 100"><path d="M10 10H90V90H10Z"></path></svg>"#
        );
    }

    #[test]
    fn complex_form_structure_with_helpers() {
        let html = form()
            .child(label().child("Name"))
            .child(input().type_("text").name("username"))
            .child(button().child("Submit"))
            .build()
            .into_string();
            
        assert_eq!(
            html,
            r#"<form><label>Name</label><input type="text" name="username"><button>Submit</button></form>"#
        );
    }
}



