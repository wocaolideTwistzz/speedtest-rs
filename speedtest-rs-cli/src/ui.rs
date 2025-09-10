use ratatui::{
    layout::{Constraint, Layout},
    style::Stylize,
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::app::{App, fetch_config::FetchConfigResult};

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let [
            fetch_config_area,
            fetch_servers_area,
            select_fastest_server_area,
            download_area,
            upload_area,
            footer_area,
        ] = Layout::vertical([
            Constraint::Min(1),
            Constraint::Min(1),
            Constraint::Min(1),
            Constraint::Min(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        self.render_fetch_config(fetch_config_area, buf);
        self.render_foot(footer_area, buf);
    }
}

impl App {
    fn render_fetch_config(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        if self.fetch_config.is_start() {
            let elapsed = self.fetch_config.elapsed().as_secs_f32();

            match self.fetch_config.get_result() {
                Some(FetchConfigResult::Success(config)) => Paragraph::new(Line::from(vec![
                    Span::raw("🎉 Fetch config success! ".to_string()).green(),
                    Span::raw(format!("Elapsed: {elapsed:.1?}s")),
                ]))
                .render(area, buf),

                Some(FetchConfigResult::Error(e)) => {
                    Paragraph::new(Line::from(vec![
                        "❌ Fetching config failed! ".red(),
                        format!("Elapsed: {elapsed:.1?}s").into(),
                    ]))
                    .render(area, buf);
                }

                None => Paragraph::new(Line::from(vec![
                    "🔍 Fetching config... ".into(),
                    format!("Elapsed: {elapsed:.1?}s").into(),
                ]))
                .render(area, buf),
            }
        }
    }

    fn render_foot(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        Paragraph::new("Press 'q' / 'esc' / 'Ctrl + D' / 'Ctrl + C' to quit")
            .centered()
            .render(area, buf);
    }
}
