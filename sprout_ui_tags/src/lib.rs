use sprout_ui_core::{Element, VoidElement};

/// Declares every HTML tag in one place, with its kind stated explicitly.
/// If a tag is ever accidentally listed in both groups, that's a real
/// compile error (duplicate function name), not a silent bug.
macro_rules! declare_tags {
    (
        container: { $($c:ident),* $(,)? }
        void: { $($v:ident),* $(,)? }
    ) => {
        $( pub fn $c() -> Element { Element::new(stringify!($c)) } )*
        $( pub fn $v() -> VoidElement { VoidElement::new(stringify!($v)) } )*
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
        svg, path, // <-- Moved 'path' here so it renders as a container with a closing tag
    }
    void: {
        br, hr, img, input, link, meta, area, base, col, embed, param, source, track, wbr,
        // <-- Removed 'path' from here
    }
}

/// Shortcut for a stylesheet <link> tag.
pub fn stylesheet(href: impl Into<String>) -> VoidElement {
    link().attr("rel", "stylesheet").href(href)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stylesheet_helper_builds_correct_link_tag() {
        let html = stylesheet("/static/style.css").build().into_string();
        assert_eq!(html, r#"<link rel="stylesheet" href="/static/style.css">"#);
    }

    #[test]
    fn svg_can_contain_path() {
        let html = svg()
            .attr("viewBox", "0 0 512 512")
            .child(path().attr("d", "M0 0L10 10"))
            .build()
            .into_string();
        
        // Cleaned up the accidental file duplication that was inside this string literal
        assert_eq!(html, r#"<svg viewBox="0 0 512 512"><path d="M0 0L10 10"></path></svg>"#);
    }
}
