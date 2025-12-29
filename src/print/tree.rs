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
// have state as well as only goes two levels deep,
// since i dont plan on having
// interactivity for this. but rather
// the root be exposed as a "list" that
// we can select through

pub enum TreeDepth {
    Level0 = 0,
    Level1 = 1,
    Level2 = 2,
}

impl From<TreeDepth> for usize {
    fn from(value: TreeDepth) -> Self {
        value as usize
    }
}

pub struct TreeItem<'text> {
    text: Text<'text>,
    depth: TreeDepth,
    is_last: bool,
}

pub struct Tree<'a> {
    root: TreeItem<'a>,

    children: &'a [TreeItem<'a>],

    style: Style,

    highlight_style: Style,

    /// pre - pipe "│"
    other_child: Text<'a>,

    /// connector - tee "├──"
    other_entry: Text<'a>,

    /// pre - no more siblings " "
    final_child: Text<'a>,

    /// connector - elbow "└── "
    final_entry: Text<'a>,
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

impl<'text> TreeItem<'text> {
    /// text widget height
    pub fn height(&self) -> usize {
        self.text.height()
    }
}

impl<'a> Tree<'a> {
    pub fn new(
        root: TreeItem<'a>,
        children: &'a [TreeItem],
    ) -> Self {
        let other_child = Text::raw("│  ");
        let other_entry = Text::raw("├──");
        let final_child = Text::raw("   ");
        let final_entry = Text::raw("└──");

        Self {
            root,
            children,
            style: Style::default(),
            highlight_style: Style::default(),
            other_child,
            other_entry,
            final_child,
            final_entry,
        }
    }

    pub fn items(
        mut self,
        items: &'a [TreeItem],
    ) -> Self {
        self.children = items;
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
        other_child: Text<'a>,
    ) -> Self {
        self.other_child = other_child;
        self
    }

    pub fn other_entry(
        mut self,
        other_entry: Text<'a>,
    ) -> Self {
        self.other_entry = other_entry;
        self
    }

    pub fn final_child(
        mut self,
        final_child: Text<'a>,
    ) -> Self {
        self.final_child = final_child;
        self
    }

    pub fn final_entry(
        mut self,
        final_entry: Text<'a>,
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
