mod tui;
use crate::tui::{api_dash::render_api_dash, dash_board::data_listener, draw_graph::draw};
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;

enum UI {
    ApiDash,
    Network,
    MetricsDash,
}
impl ToString for UI {
    fn to_string(&self) -> String {
        let string = match self {
            UI::ApiDash => "API Dashboard",
            UI::Network => "Network",
            UI::MetricsDash => "Metrics Dashboard",
        };
        string.into()
    }
}

#[tokio::main]
async fn main() {
    let ui = [UI::ApiDash, UI::Network, UI::MetricsDash];

    let ui_type = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select node type")
        .default(0)
        .items(&ui)
        .interact()
        .unwrap();
    let ui = &ui[ui_type];

    match ui {
        UI::ApiDash => render_api_dash(),
        UI::Network => draw(),
        UI::MetricsDash => data_listener().await,
    }
}
