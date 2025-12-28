use ratatui::{
    style::Style,
    text::Text,
    widgets::{StatefulWidget, Widget},
};

// this is a util widget that helps
// pretty printing trees
// it's a modified version of the original
// arena graph that used crossterm
// this time however, it also takes heavy
// inspiration from the `tui-rs-tree-widget`
// crate that implements ratatui's widget trait
//
// the main difference, being this doesn't
// have state, since i dont plan on having
// interactivity for this. but rather
// the root be exposed as a "list" that
// we can select through

pub struct TreeItem<'text> {
    text: Text<'text>,
    children: Vec<Self>,
}

pub struct Tree<'a> {
    items: &'a [TreeItem<'a>],

    style: Style,

    highlight_style: Style,

    /// pre - pipe "│"
    other_child: &'a str,

    /// connector - tee "├──"
    other_entry: &'a str,

    /// pre - no more siblings " "
    final_child: &'a str,

    /// connector - elbow "└── "
    final_entry: &'a str,
}

#[derive(Default, Clone)]
pub struct TreeState {
    /// optional selection
    /// this should select over
    /// the root tree nodes
    /// and not go any deeper
    /// i dont think ill support that
    /// for now
    pub selected: Option<usize>,

    /// collapse trees
    /// onto a single line
    pub collapsed: bool,
}

impl TreeState {
    pub fn flatten<'a>(
        &self,
        items: &'a [TreeItem<'a>],
    ) -> Vec<Flattened<'a>> {
        todo!()
    }

    pub fn select_next(&mut self) {
        self.selected =
            Some(self.selected.map_or(0, |i| i.saturating_add(1)));
    }

    pub fn select_prev(&mut self) {
        self.selected =
            Some(self.selected.map_or(0, |i| i.saturating_sub(1)));
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }
}

/// from tui-rs-tree-widget
/// modified wihtout the Identifier
/// and since we dont have any of the identifiers
/// gonna need to know depth,
/// likely stored here,
/// as well as which root this belongs too
pub struct Flattened<'text> {
    pub item: &'text TreeItem<'text>,

    pub depth: usize,
    pub root: usize,

    /// is this last in the tree?
    pub is_last: bool,
}

impl<'text> TreeItem<'text> {
    /// create new treeitem, needs children
    pub fn new<T>(
        text: T,
        children: Vec<Self>,
    ) -> Self
    where
        T: Into<Text<'text>>,
    {
        Self {
            text: text.into(),
            children,
        }
    }

    /// create new leaf, no children
    pub fn new_leaf<T>(text: T) -> Self
    where
        T: Into<Text<'text>>,
    {
        Self {
            text: text.into(),
            children: Vec::new(),
        }
    }

    pub fn children(&self) -> &[Self] {
        &self.children
    }

    /// add a child to treeitem
    pub fn add_child(
        &mut self,
        child: Self,
    ) {
        self.children.push(child);
    }

    /// Get a reference to a child by index.
    pub fn child(
        &self,
        index: usize,
    ) -> Option<&Self> {
        self.children.get(index)
    }

    /// Get a mutable reference to a child by index.
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

impl<'a> Tree<'a> {
    pub fn new(items: &'a [TreeItem]) -> Self {
        Self {
            items,
            style: Style::default(),
            highlight_style: Style::default(),
            other_child: "│ ",
            other_entry: "├──",
            final_child: "  ",
            final_entry: "└──",
        }
    }

    pub fn items(
        mut self,
        items: &'a [TreeItem],
    ) -> Self {
        self.items = items;
        self
    }

    pub fn style(
        mut self,
        style: Style,
    ) -> Self {
        self.style = style;
        self
    }

    pub fn highlight_style(
        mut self,
        style: Style,
    ) -> Self {
        self.highlight_style = style;
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
}

impl StatefulWidget for Tree<'_> {
    type State = TreeState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        todo!()
    }
}

impl Widget for Tree<'_> {
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) where
        Self: Sized,
    {
        let mut state = TreeState::default();
        StatefulWidget::render(self, area, buf, &mut state);
    }
}
