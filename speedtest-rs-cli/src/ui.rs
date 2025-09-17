use ratatui::{
    layout::{Alignment, Constraint, Layout, Margin},
    style::{Color, Style, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{
        Axis, Block, BorderType, Borders, Chart, Dataset, List, ListItem, Padding, Paragraph, Row,
        Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState, Widget,
    },
};
use speedtest_rs_core::Humanize;

use crate::{
    app::{App, progress::Progress},
    event::Status,
};

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let [
            progresses_area,
            information_area,
            download_area,
            upload_area,
            footer_area,
        ] = Layout::vertical([
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(1),
        ])
        .areas(area);

        self.render_progresses(progresses_area, buf);
        self.render_information(information_area, buf);
        self.render_download(download_area, buf);
        self.render_upload(upload_area, buf);
        self.render_foot(footer_area, buf);
    }
}

impl App {
    fn render_progresses(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let block = Block::new()
            .title(Line::raw(" > Progress ").bold())
            .padding(Padding::uniform(1))
            .borders(Borders::all())
            .border_type(BorderType::Thick);

        let list = List::new([
            ListItem::from(&self.fetch_config),
            ListItem::from(&self.fetch_servers),
            ListItem::from(&self.racing_servers),
            ListItem::from(&self.download),
            ListItem::from(&self.upload),
        ])
        .block(block);

        Widget::render(&list, area, buf);
    }

    fn render_information(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let [config_area, servers_area] =
            Layout::horizontal([Constraint::Length(35), Constraint::Fill(1)]).areas(area);

        self.render_config(config_area, buf);
        self.render_servers(servers_area, buf);
    }

    fn render_download(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        if matches!(self.download.status(), Status::Pending | Status::Canceled) {
            self.render_not_ok(area, buf, " > Download ", self.download.status());
            return;
        }

        let block = Block::new()
            .title(" > Download ".bold())
            .padding(Padding::uniform(1))
            .borders(Borders::all())
            .border_type(BorderType::Thick)
            .border_style(Style::default().magenta());

        let inner = block.inner(area);
        let [summary_area, chart_area] =
            Layout::horizontal([Constraint::Length(30), Constraint::Fill(1)]).areas(inner);

        self.render_download_summary(summary_area, buf);
        self.render_download_chart(chart_area, buf);

        block.render(area, buf);
    }

    fn render_upload(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        if matches!(self.upload.status(), Status::Pending | Status::Canceled) {
            self.render_not_ok(area, buf, " > Upload ", self.upload.status());
            return;
        }

        let block = Block::new()
            .title(" > Upload ".bold())
            .padding(Padding::uniform(1))
            .borders(Borders::all())
            .border_type(BorderType::Thick)
            .border_style(Style::default().cyan());

        let inner = block.inner(area);
        let [summary_area, chart_area] =
            Layout::horizontal([Constraint::Length(30), Constraint::Fill(1)]).areas(inner);

        self.render_upload_summary(summary_area, buf);
        self.render_upload_chart(chart_area, buf);

        block.render(area, buf);
    }

    fn render_config(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        match self.fetch_config.status() {
            Status::Ok(config) => {
                let rows = [
                    Row::new([Span::from("IP").bold().yellow(), Span::from(&config.ip)]),
                    Row::new([Span::from("ISP").bold().yellow(), Span::from(&config.isp)]),
                    Row::new([
                        Span::from("Country").bold().yellow(),
                        Span::from(&config.country),
                    ]),
                    Row::new([
                        Span::from("Latitude").bold().yellow(),
                        Span::from(&config.latitude),
                    ]),
                    Row::new([
                        Span::from("Longitude").bold().yellow(),
                        Span::from(&config.longitude),
                    ]),
                ];
                Table::new(rows, [Constraint::Length(10), Constraint::Fill(1)])
                    .block(
                        Block::new()
                            .title(" > Config ".bold())
                            .padding(Padding::uniform(1))
                            .borders(Borders::all())
                            .border_type(BorderType::Thick)
                            .border_style(Style::new().light_cyan()),
                    )
                    .render(area, buf);
            }
            status => self.render_not_ok(area, buf, " > Config ", status),
        };
    }

    fn render_servers(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        match self.fetch_servers.status() {
            Status::Ok(servers) => {
                let fastest = if let Status::Ok(server) = self.racing_servers.status() {
                    Some(&server.url)
                } else {
                    None
                };
                let mut rows = vec![];
                for server in servers {
                    if fastest.is_some_and(|v| *v == server.url) {
                        rows.insert(
                            0,
                            Row::new([
                                Span::from("🚀").green(),
                                Span::from(server.name.clone()).green(),
                                Span::from(server.country.clone()).green(),
                                Span::from(server.url.clone()).green(),
                            ]),
                        );
                    } else {
                        rows.push(Row::new([
                            Span::from(" "),
                            Span::from(server.name.clone()),
                            Span::from(server.country.clone()),
                            Span::from(server.url.clone()),
                        ]));
                    }
                }
                let mut table_state = TableState::new().with_offset(self.servers_scroll);
                let mut state =
                    ScrollbarState::new(self.max_servers_scroll).position(self.servers_scroll);

                ratatui::widgets::StatefulWidget::render(
                    Table::new(
                        rows,
                        [
                            Constraint::Length(2),
                            Constraint::Length(15),
                            Constraint::Length(15),
                            Constraint::Fill(1),
                        ],
                    )
                    .header(
                        Row::new([
                            Span::from(""),
                            Span::from("Name"),
                            Span::from("Country"),
                            Span::from("URL"),
                        ])
                        .yellow()
                        .bold(),
                    )
                    .block(
                        Block::new()
                            .title(" > Servers ".bold())
                            .title(Line::from(" Use j k or ▲ ▼  to scroll ").right_aligned())
                            .padding(Padding::uniform(1))
                            .borders(Borders::all())
                            .border_type(BorderType::Thick)
                            .border_style(Style::new().light_cyan()),
                    ),
                    area,
                    buf,
                    &mut table_state,
                );

                ratatui::widgets::StatefulWidget::render(
                    Scrollbar::new(ScrollbarOrientation::VerticalRight),
                    area.inner(Margin::new(1, 0)),
                    buf,
                    &mut state,
                );
            }
            status => self.render_not_ok(area, buf, " > Servers ", status),
        };
    }

    fn render_foot(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        Paragraph::new("Press 'q' / 'esc' / 'Ctrl + D' / 'Ctrl + C' to quit")
            .centered()
            .render(area, buf);
    }

    fn render_not_ok<T>(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        title: &str,
        status: &Status<T>,
    ) {
        let block = Block::new()
            .title(title.bold())
            .padding(Padding::uniform(1))
            .borders(Borders::all())
            .border_type(BorderType::Thick);

        match status {
            Status::Pending => Paragraph::new("Pending")
                .alignment(Alignment::Center)
                .block(block.gray())
                .render(area, buf),
            Status::Start => Paragraph::new("Running...")
                .yellow()
                .alignment(Alignment::Center)
                .block(block.yellow())
                .render(area, buf),
            Status::Err(e) => Paragraph::new(e.to_string())
                .red()
                .alignment(Alignment::Center)
                .block(block.red())
                .render(area, buf),
            Status::Canceled => Paragraph::new("Canceled")
                .alignment(Alignment::Center)
                .block(block)
                .render(area, buf),
            _ => {}
        }
    }

    fn render_download_summary(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let min_data = self.min_download_byte_ps().humanize_bitrate(1000);
        let max_data = self.max_download_byte_ps().humanize_bitrate(1000);
        let latest_data = self.latest_download_byte_ps().humanize_bitrate(1000);
        let avg_data = self.avg_download_byte_ps().humanize_bitrate(1000);

        let total = self.total_download_bytes().humanize_bytes();

        let rows = [
            Row::new([Span::from("Min").bold().yellow(), Span::from(min_data)]),
            Row::new([Span::from("Max").bold().yellow(), Span::from(max_data)]),
            Row::new([
                Span::from("Latest").bold().yellow(),
                Span::from(latest_data),
            ]),
            Row::new([Span::from("Avg").bold().yellow(), Span::from(avg_data)]),
            Row::new([Span::from("Total").bold().yellow(), Span::from(total)]),
        ];

        Table::new(rows, [Constraint::Length(10), Constraint::Length(20)]).render(area, buf);
    }

    fn render_download_chart(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let min_data = self.min_download_byte_ps();
        let max_data = self.max_download_byte_ps();

        let (max_data_f64, unit) = max_data.humanize();
        let min_data_f64 = min_data as f64 / unit as f64;

        let min_bound = min_data_f64 * 0.9;
        let max_bound = max_data_f64 * 1.1;

        let min_bound_str = ((min_data as f64 * 0.9) as usize).humanize_bitrate(1000);
        let max_bound_str = ((max_data as f64 * 1.1) as usize).humanize_bitrate(1000);

        let render_data: Vec<(f64, f64)> = self
            .downloaded_data
            .iter()
            .enumerate()
            .map(|(idx, v)| (idx as f64, *v as f64 / unit as f64))
            .collect();

        let dataset = Dataset::default()
            .marker(symbols::Marker::Braille)
            .style(Style::new().fg(Color::Magenta))
            .graph_type(ratatui::widgets::GraphType::Line)
            .data(&render_data);

        Chart::new(vec![dataset])
            .x_axis(Axis::default().bounds([0.0, 19.0]))
            .y_axis(
                Axis::default()
                    .bounds([min_bound, max_bound])
                    .labels([min_bound_str, max_bound_str]),
            )
            .render(area, buf);
    }

    fn render_upload_summary(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let min_data = self.min_upload_byte_ps().humanize_bitrate(1000);
        let max_data = self.max_upload_byte_ps().humanize_bitrate(1000);
        let latest_data = self.latest_upload_byte_ps().humanize_bitrate(1000);
        let avg_data = self.avg_upload_byte_ps().humanize_bitrate(1000);

        let total = self.total_upload_bytes().humanize_bytes();

        let rows = [
            Row::new([Span::from("Min").bold().yellow(), Span::from(min_data)]),
            Row::new([Span::from("Max").bold().yellow(), Span::from(max_data)]),
            Row::new([
                Span::from("Latest").bold().yellow(),
                Span::from(latest_data),
            ]),
            Row::new([Span::from("Avg").bold().yellow(), Span::from(avg_data)]),
            Row::new([Span::from("Total").bold().yellow(), Span::from(total)]),
        ];

        Table::new(rows, [Constraint::Length(10), Constraint::Length(20)]).render(area, buf);
    }

    fn render_upload_chart(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let min_data = self.min_upload_byte_ps();
        let max_data = self.max_upload_byte_ps();

        let (max_data_f64, unit) = max_data.humanize();
        let min_data_f64 = min_data as f64 / unit as f64;

        let min_bound = min_data_f64 * 0.9;
        let max_bound = max_data_f64 * 1.1;

        let min_bound_str = ((min_data as f64 * 0.9) as usize).humanize_bitrate(1000);
        let max_bound_str = ((max_data as f64 * 1.1) as usize).humanize_bitrate(1000);

        let render_data: Vec<(f64, f64)> = self
            .uploaded_data
            .iter()
            .enumerate()
            .map(|(idx, v)| (idx as f64, *v as f64 / unit as f64))
            .collect();

        let dataset = Dataset::default()
            .marker(symbols::Marker::Braille)
            .style(Style::new().fg(Color::Cyan))
            .graph_type(ratatui::widgets::GraphType::Line)
            .data(&render_data);

        Chart::new(vec![dataset])
            .x_axis(Axis::default().bounds([0.0, 19.0]))
            .y_axis(
                Axis::default()
                    .bounds([min_bound, max_bound])
                    .labels([min_bound_str, max_bound_str]),
            )
            .render(area, buf);
    }
}

impl<T> From<&Progress<T>> for ListItem<'_> {
    fn from(value: &Progress<T>) -> Self {
        let elapsed = value.elapsed().as_secs_f32();
        let line = match value.status() {
            Status::Start => Line::from(vec![
                Span::raw(format!("⏳ {:<15} ........ ", value.name())).bold(),
                Span::raw(format!("Elapsed: {elapsed:.1?}s")),
            ]),
            Status::Err(e) => Line::from(vec![
                Span::raw(format!("❌ {:<15} Failed! ", value.name()))
                    .bold()
                    .red(),
                Span::raw(format!("Elapsed: {elapsed:.1?}s")),
                Span::raw(format!(" > {e}")).red(),
            ]),
            Status::Ok(_) => Line::from(vec![
                Span::raw(format!("🎉 {:<15} Success! ", value.name()))
                    .bold()
                    .green(),
                Span::raw(format!("Elapsed: {elapsed:.1?}s")),
            ]),
            Status::Pending => Line::from(vec![
                Span::raw(format!("👻 {:<15} Pending ...... ", value.name())).gray(),
            ]),
            Status::Canceled => Line::from(vec![
                Span::raw(format!("💔 {:<15} Canceled! ", value.name()))
                    .yellow()
                    .bold(),
            ]),
        };
        ListItem::new(line)
    }
}
