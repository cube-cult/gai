use ratatui::{style::Style, widgets::Widget};

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

pub struct TreeItem {}

pub struct Tree {
    items: Vec<TreeItem>,

    style: Style,

    /// pre - pipe "│"
    other_child: String,

    /// connector - tee "├──"
    other_entry: String,

    /// pre - no more siblings
    final_child: String,

    /// connector - elbow
    final_entry: String,
}

impl Widget for Tree {
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) where
        Self: Sized,
    {
        todo!()
    }
}
