//! A site's look, as the CSS variables pyle's rules are written against,
//! and the one stylesheet that carries the variables and the rules.

use leptos::prelude::*;

use crate::components;

#[cfg(feature = "stories")]
#[path = "theme.stories.rs"]
pub(crate) mod stories;

/// The colors a pyle component draws with. Each is any CSS color, written
/// into the stylesheet as given.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    /// Behind the whole page.
    pub page: &'static str,
    /// Behind a bar or a panel.
    pub surface: &'static str,
    /// Behind a control that sits on a surface.
    pub surface_strong: &'static str,
    /// Lines around and between things.
    pub border: &'static str,
    pub text: &'static str,
    /// Text that matters less, such as a label.
    pub text_muted: &'static str,
    /// What draws the eye: a notice, a link.
    pub accent: &'static str,
    /// The accent under the pointer.
    pub accent_hover: &'static str,
}

/// A site's look. `<PyleStyles theme=.../>` writes it to `:root` as
/// `--pyle-*` variables, so a site can also set any one of them in its own
/// CSS, outside any layer or in a layer it ranks above `pyle`. A
/// declaration in a layer ranked below `pyle` (a `base` layer, Tailwind's
/// `theme`) loses to the one written here.
///
/// It holds what the components read today. A field arrives with the first
/// component that needs it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Theme {
    pub palette: Palette,
    /// The palette when the visitor's system asks for dark, or `None` for
    /// a site that looks the same either way.
    pub dark: Option<Palette>,
    /// How round a control's corners are, as a CSS length.
    pub radius: &'static str,
    /// The font stack, as CSS `font-family` takes it.
    pub font: &'static str,
}

impl Default for Theme {
    /// A plain look, light or dark with the visitor's system, for a site
    /// that has not chosen one.
    fn default() -> Self {
        DEFAULT
    }
}

/// [`Theme::default`] as a constant, for the theme playground's starting
/// values.
pub(crate) const DEFAULT: Theme = Theme {
    palette: Palette {
        page: "#ffffff",
        surface: "#f6f7f8",
        surface_strong: "#ffffff",
        border: "#d0d4d9",
        text: "#1f2328",
        text_muted: "#59636e",
        accent: "#9a6700",
        accent_hover: "#1f2328",
    },
    dark: Some(Palette {
        page: "#0d1117",
        surface: "#161b22",
        surface_strong: "#0d1117",
        border: "#30363d",
        text: "#e6edf3",
        text_muted: "#9198a1",
        accent: "#d29922",
        accent_hover: "#e6edf3",
    }),
    radius: "0.25rem",
    font: "ui-sans-serif, system-ui, sans-serif",
};

fn colors(palette: &Palette) -> String {
    [
        ("page", palette.page),
        ("surface", palette.surface),
        ("surface-strong", palette.surface_strong),
        ("border", palette.border),
        ("text", palette.text),
        ("text-muted", palette.text_muted),
        ("accent", palette.accent),
        ("accent-hover", palette.accent_hover),
    ]
    .map(|(name, value)| format!("  --pyle-{name}: {value};\n"))
    .concat()
}

/// pyle's whole stylesheet for `theme`: the theme's variables on `:root`,
/// then every component's rules, all inside the cascade layer `pyle`. A site
/// ranks that layer below its own utilities, so a class the site puts on a
/// pyle component wins.
pub fn stylesheet(theme: &Theme) -> String {
    let mut css = format!("@layer pyle {{\n:root {{\n{}", colors(&theme.palette));
    css += &format!("  --pyle-radius: {};\n", theme.radius);
    css += &format!("  --pyle-font: {};\n}}\n", theme.font);
    if let Some(dark) = &theme.dark {
        css += &format!(
            "@media (prefers-color-scheme: dark) {{\n:root {{\n{}}}\n}}\n",
            colors(dark)
        );
        // On a page pyle draws, what the browser draws itself (scrollbars,
        // a menu's list) goes dark with the palette. A site that mounts a
        // component on its own page keeps the scheme it has.
        css += ":root:has(.pyle-page) {\n  color-scheme: light dark;\n}\n";
    }
    for rules in components::CSS {
        css += rules;
    }
    css + "}\n"
}

/// Puts pyle's stylesheet in the page. Mount it once, above the first pyle
/// component.
///
/// The rules arrive with the first render, so a client-rendered site shows
/// no unstyled frame. They sit in `@layer pyle`, and a layer declared later
/// outranks one declared earlier, so the site's own stylesheet has to name
/// the order before this component runs (the README shows the line).
#[component]
pub fn PyleStyles(
    /// The site's look; the default is a plain one.
    #[prop(optional)]
    theme: Theme,
) -> impl IntoView {
    view! { <style id="pyle-styles" inner_html=stylesheet(&theme)></style> }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLAIN: Palette = Palette {
        page: "p",
        surface: "s",
        surface_strong: "ss",
        border: "b",
        text: "t",
        text_muted: "tm",
        accent: "a",
        accent_hover: "ah",
    };

    fn one_look() -> Theme {
        Theme {
            palette: PLAIN,
            dark: None,
            radius: "2px",
            font: "serif",
        }
    }

    /// The text between the first `{` and the last `}`: what the outermost
    /// block holds.
    fn body(css: &str) -> &str {
        let open = css.find('{').expect("a block");
        let close = css.rfind('}').expect("a closed block");
        &css[open + 1..close]
    }

    /// How deep in blocks the stylesheet is after each character of it.
    fn depths(css: &str) -> Vec<i32> {
        css.chars()
            .scan(0, |depth, c| {
                *depth += match c {
                    '{' => 1,
                    '}' => -1,
                    _ => 0,
                };
                Some(*depth)
            })
            .collect()
    }

    #[test]
    fn the_stylesheet_is_one_block_in_the_pyle_layer() {
        let css = stylesheet(&Theme::default());
        let depths = depths(css.trim_end());
        let closes_early = depths[..depths.len() - 1]
            .iter()
            .skip_while(|d| **d == 0)
            .any(|d| *d == 0);
        assert!(
            css.starts_with("@layer pyle {") && depths.last() == Some(&0) && !closes_early,
            "a rule outside the layer would outrank a site's utilities:\n{css}"
        );
    }

    #[test]
    fn no_rule_is_important() {
        // `!important` reverses layer order, so one would beat the site.
        let css = stylesheet(&Theme::default());
        assert_eq!(leptotes::count(&css, "!important"), 0, "{css}");
    }

    /// What each block of `css` is opened by: every selector, with a list of
    /// them split, and every at-rule.
    fn openers(css: &str) -> Vec<&str> {
        let mut pieces: Vec<&str> = css.split('{').collect();
        pieces.pop(); // What follows the last `{` opens nothing.
        pieces
            .into_iter()
            .filter_map(|before| before.rsplit(['}', ';']).next())
            .flat_map(|opener| opener.split(','))
            .map(str::trim)
            .collect()
    }

    #[test]
    fn a_list_of_selectors_is_read_as_each_of_them() {
        assert_eq!(
            openers(".a,\nselect { color: red; }\n@media print {\n.b { top: 0; }\n}"),
            [".a", "select", "@media print", ".b"]
        );
    }

    // pyle styles its own elements and no others. The exceptions are the
    // root and the body that hold a `LandingPage`, whose page pyle draws.
    #[test]
    fn every_rule_is_a_pyle_class_or_the_root() {
        let css = stylesheet(&Theme::default());
        let strangers: Vec<&str> = openers(body(&css))
            .into_iter()
            .filter(|selector| {
                !(selector.starts_with(".pyle-")
                    || *selector == ":root"
                    || *selector == ":root:has(.pyle-page)"
                    || *selector == "body:has(.pyle-page)"
                    || selector.starts_with("@media"))
            })
            .collect();
        assert_eq!(strangers, Vec::<&str>::new(), "{css}");
    }

    #[test]
    fn every_variable_a_rule_reads_is_one_the_theme_sets() {
        let css = stylesheet(&one_look());
        let unset: Vec<&str> = css
            .split("var(")
            .skip(1)
            .filter_map(|rest| rest.split(')').next())
            .filter(|name| !css.contains(&format!("  {name}: ")))
            .collect();
        assert_eq!(unset, Vec::<&str>::new(), "{css}");
    }

    #[test]
    fn the_theme_s_colors_are_root_variables() {
        assert_eq!(
            colors(&PLAIN),
            "  --pyle-page: p;\n  --pyle-surface: s;\n  --pyle-surface-strong: ss;\n  \
             --pyle-border: b;\n  \
             --pyle-text: t;\n  --pyle-text-muted: tm;\n  --pyle-accent: a;\n  \
             --pyle-accent-hover: ah;\n"
        );
    }

    #[test]
    fn the_radius_and_font_are_root_variables() {
        let css = stylesheet(&one_look());
        assert!(
            css.contains("  --pyle-radius: 2px;\n  --pyle-font: serif;\n}\n"),
            "{css}"
        );
    }

    // Neither the system's query nor a scheme for the browser's own parts.
    #[test]
    fn a_theme_with_one_look_asks_nothing_of_the_system() {
        let css = stylesheet(&one_look());
        assert_eq!(leptotes::count(&css, "color-scheme"), 0, "{css}");
    }

    // Scoped to a page pyle draws: on the bare root it would turn a site's
    // own page dark under a component it mounts. The browser test
    // `the_browser_s_own_parts_follow_the_system_on_a_page_with_a_dark_palette`
    // sees it applied.
    #[test]
    fn a_theme_with_a_dark_palette_lets_the_browser_s_parts_follow_the_system() {
        let css = stylesheet(&Theme {
            dark: Some(PLAIN),
            ..one_look()
        });
        assert_eq!(
            leptotes::count(
                &css,
                ":root:has(.pyle-page) {\n  color-scheme: light dark;\n}\n"
            ),
            1,
            "{css}"
        );
    }

    // Same selector, same layer: the later block wins, so the dark one has
    // to follow the light one. The browser test
    // `a_visitor_who_asks_for_dark_gets_the_dark_palette` sees it applied.
    #[test]
    fn the_dark_palette_is_set_after_the_light_one_under_the_system_s_query() {
        let dark = Palette {
            surface: "dark-s",
            ..PLAIN
        };
        let css = stylesheet(&Theme {
            dark: Some(dark),
            ..one_look()
        });
        let light = css.find("  --pyle-surface: s;\n");
        let dark =
            css.find("@media (prefers-color-scheme: dark) {\n:root {\n  --pyle-page: p;\n  --pyle-surface: dark-s;\n");
        assert!(light.is_some() && light < dark, "{css}");
    }

    #[test]
    fn the_styles_component_carries_the_theme_s_stylesheet() {
        let html = leptotes::render(|| view! { <PyleStyles theme=one_look() /> });
        assert_eq!(
            html,
            format!(
                r#"<style id="pyle-styles">{}</style>"#,
                stylesheet(&one_look())
            )
        );
    }
}
