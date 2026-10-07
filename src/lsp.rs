use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct LspPanel;

impl LspPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Paragraph::new("").block(Block::default().title(" LSP Lean ").borders(Borders::TOP)),
            area,
        );
    }
}
