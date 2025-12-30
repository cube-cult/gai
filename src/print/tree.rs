use std::collections::HashSet;

use ratatui::{style::Style, text::Text, widgets::Widget};

// this is a util widget that helps
// pretty printing trees
// it's a modified version of the original
// arena graph that used crossterm
// this time however, it also takes heavy
// inspiration from the `tui-rs-tree-widget`
// crate that implements ratatui's widget trait
//
// the main difference, being this doesn't
// have state as well as only goes two levels deep,
// since i dont plan on having
// interactivity for this. but rather
// the root be exposed as a "list" that
// we can select through

#[derive(Debug, Clone)]
pub struct TreeItem<'text, Identifier> {
    identifier: Identifier,
    children: Vec<Self>,

    text: Text<'text>,
}

#[derive(Debug, Clone)]
pub struct Tree<'a, Identifier> {
    items: &'a [TreeItem<'a, Identifier>],

    style: Style,

    collapsed: bool,

    /// pre - pipe "│"
    other_child: &'a str,

    /// connector - tee "├──"
    other_entry: &'a str,

    /// pre - no more siblings " "
    final_child: &'a str,

    /// connector - elbow "└── "
    final_entry: &'a str,
}

impl<'text, Identifier> TreeItem<'text, Identifier>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    pub fn new_leaf<T>(
        identifier: Identifier,
        text: T,
    ) -> Self
    where
        T: Into<Text<'text>>,
    {
        let text = text.into();
        // todo temp fix, add style to line
        // conversely, user can use Line::styled
        let text = Text::from(
            text.lines
                .into_iter()
                .map(|line| line.patch_style(text.style))
                .collect::<Vec<_>>(),
        );

        Self {
            identifier,
            text,
            children: Vec::new(),
        }
    }

    pub fn new<T>(
        identifier: Identifier,
        text: T,
        children: Vec<Self>,
    ) -> std::io::Result<Self>
    where
        T: Into<Text<'text>>,
    {
        let identifiers: HashSet<_> =
            children.iter().map(|item| &item.identifier).collect();

        if identifiers.len() != children.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "The children contain duplicate identifiers",
            ));
        }

        let text = text.into();
        // todo temp fix, add style to line
        // conversely, user can use Line::styled
        let text = Text::from(
            text.lines
                .into_iter()
                .map(|line| line.patch_style(text.style))
                .collect::<Vec<_>>(),
        );

        Ok(Self {
            identifier,
            text,
            children,
        })
    }

    pub const fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    pub fn children(&self) -> &[Self] {
        &self.children
    }

    pub fn child(
        &self,
        index: usize,
    ) -> Option<&Self> {
        self.children.get(index)
    }

    pub fn child_mut(
        &mut self,
        index: usize,
    ) -> Option<&mut Self> {
        self.children.get_mut(index)
    }

    /// text widget height
    pub fn height(&self) -> usize {
        self.text.height()
    }
}

impl<'a, Identifier> Tree<'a, Identifier>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    pub fn new(
        items: &'a [TreeItem<'a, Identifier>]
    ) -> std::io::Result<Self> {
        let identifiers = items
            .iter()
            .map(|item| &item.identifier)
            .collect::<HashSet<_>>();

        if identifiers.len() != items.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "The items contain duplicate identifiers",
            ));
        }

        let other_child = "│  ";
        let other_entry = "├──";
        let final_child = "   ";
        let final_entry = "└──";

        Ok(Self {
            items,
            style: Style::default(),
            collapsed: false,
            other_child,
            other_entry,
            final_child,
            final_entry,
        })
    }

    pub fn style(
        mut self,
        style: Style,
    ) -> Self {
        self.style = style;
        self
    }

    pub fn toggle_collapse(&mut self) {
        self.collapsed = !self.collapsed;
    }

    pub fn collapsed(
        mut self,
        collapsed: bool,
    ) -> Self {
        self.collapsed = collapsed;
        self
    }

    pub fn other_child(
        mut self,
        other_child: &'a str,
    ) -> Self {
        self.other_child = other_child;
        self
    }

    pub fn other_entry(
        mut self,
        other_entry: &'a str,
    ) -> Self {
        self.other_entry = other_entry;
        self
    }

    pub fn final_child(
        mut self,
        final_child: &'a str,
    ) -> Self {
        self.final_child = final_child;
        self
    }

    pub fn final_entry(
        mut self,
        final_entry: &'a str,
    ) -> Self {
        self.final_entry = final_entry;
        self
    }

    // util create the prefix character
    // based on a flattened item
    fn prefix(
        &self,
        is_last_at_depth: &[bool],
    ) -> String {
        let depth = is_last_at_depth.len();

        if depth == 0 {
            return String::new();
        }

        let mut prefix = String::new();

        // add continuation characters
        for &is_last in &is_last_at_depth[..depth - 1] {
            if is_last {
                prefix.push_str(self.final_child);
            } else {
                prefix.push_str(self.other_child);
            }
        }

        // add connector for curr_level
        if is_last_at_depth[depth - 1] {
            prefix.push_str(self.final_entry);
        } else {
            prefix.push_str(self.other_entry);
        }

        prefix
    }
}

impl<Identifier> Widget for Tree<'_, Identifier>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) where
        Self: Sized,
    {
        buf.set_style(area, self.style);

        if area.width < 1 || area.height < 1 || self.items.is_empty()
        {
            return;
        }

        let flattened = flatten(self.items, &[], self.collapsed, 0);

        let mut y = area.y;

        for flat in flattened.iter() {
            if y >= area.y + area.height {
                break;
            }

            let prefix = self.prefix(&flat.is_last_at_depth);
            let prefix_char_count = prefix.chars().count() as u16;

            // render prefix
            if !prefix.is_empty() {
                buf.set_string(area.x, y, &prefix, self.style);
            }

            // render content
            let text_x = area.x + prefix_char_count;
            let text_width =
                area.width.saturating_sub(prefix_char_count);

            for (i, line) in flat.item.text.lines.iter().enumerate() {
                let line_y = y + i as u16;
                if line_y >= area.y + area.height {
                    break;
                }

                // handle continuing lines dont need a connector
                if i > 0 {
                    let mut prefix = String::new();

                    for &is_last in &flat.is_last_at_depth {
                        if is_last {
                            prefix.push_str(self.final_child);
                        } else {
                            prefix.push_str(self.other_child);
                        }
                    }

                    buf.set_string(
                        area.x, line_y, &prefix, line.style,
                    );
                }

                buf.set_line(text_x, line_y, line, text_width);
            }

            y += flat.item.height() as u16;
        }
    }
}

// util flatten function, compared to tui-rs-tree-widget
// we don't collapse per tree item, the entire tree is collapsed
// but i think keeping the identifier is fine here, in case
// i do want to track which tree item corresponds to whatever
// likely wont use it though

struct Flattened<'text, Identifier> {
    item: &'text TreeItem<'text, Identifier>,
    /// assign the last item for the each depth
    is_last_at_depth: Vec<bool>,
}

fn flatten<'text, Identifier>(
    items: &'text [TreeItem<'text, Identifier>],
    parent_is_last_chain: &[bool],
    collapsed: bool,
    depth: usize,
) -> Vec<Flattened<'text, Identifier>>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    let mut flattened = Vec::new();
    let len = items.len();

    for (i, item) in items.iter().enumerate() {
        let is_last = i == len - 1;

        // Roots (depth 0) have empty is_last_at_depth (no prefix)
        // Children extend parent's chain with their own is_last status
        let is_last_at_depth = if depth == 0 {
            Vec::new()
        } else {
            let mut chain = parent_is_last_chain.to_vec();
            chain.push(is_last);
            chain
        };

        flattened.push(Flattened {
            item,
            is_last_at_depth: is_last_at_depth.clone(),
        });

        // handle children with recursion
        if !collapsed {
            let children = flatten(
                &item.children,
                &is_last_at_depth,
                collapsed,
                depth + 1,
            );

            flattened.extend(children);
        }
    }

    flattened
}
