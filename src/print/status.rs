use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::Text,
};

use crate::git::status::{FileStatus, GitStatus};

use super::{
    terminal,
    tree::{Tree, TreeItem},
};

pub fn print(
    status: &GitStatus,
    _compact: bool,
) -> anyhow::Result<()> {
    let height = 40;
    let mut terminal = terminal::start(height)?;

    let app = App::new(status);

    terminal.draw(|f| app.draw(f))?;

    terminal::stop()?;

    Ok(())
}

struct App<'a> {
    branch: &'a str,
    statuses: &'a [FileStatus],
}

impl<'a> App<'a> {
    fn new(status: &'a GitStatus) -> Self {
        Self {
            branch: &status.branch_name,
            statuses: &status.statuses,
        }
    }

    fn draw(
        &self,
        frame: &mut Frame,
    ) {
        let [branch_area, statuses_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(frame.area());

        let branch_text = Text::styled(
            self.branch,
            Style::default().fg(Color::Magenta),
        );

        frame.render_widget(branch_text, branch_area);

        let staged_style = Style::default().fg(Color::Green);
        let modified_style = Style::default().fg(Color::Yellow);
        let untracked_style = Style::default().fg(Color::Red);

        let items = vec![
            TreeItem::new(
                "g",
                Text::styled("g", Style::default().fg(Color::Blue)),
                vec![
                    TreeItem::new_leaf("b", " test b"),
                    TreeItem::new_leaf("a", " test a"),
                ],
            )
            .unwrap(),
            TreeItem::new_leaf("cargo", "Cargo.toml"),
        ];

        let tree_widget = Tree::new(&items).expect("unique ids");

        frame.render_widget(tree_widget, statuses_area);
    }
}
