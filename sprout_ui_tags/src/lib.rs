use sprout_ui_core::{Element, VoidElement, SproutStr};

macro_rules! declare_tags {
    (
        container: { $($c:ident),* $(,)? }
        void: { $($v:ident),* $(,)? }
    ) => {
        $(
            #[track_caller]
            #[inline(always)]
            pub fn $c() -> Element { Element::new(stringify!($c)) }
        )*
        $(
            #[track_caller]
            #[inline(always)]
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

#[inline(always)]
pub fn stylesheet(href: impl Into<SproutStr>) -> VoidElement {
    link().attr("rel", "stylesheet").href(href)
}
