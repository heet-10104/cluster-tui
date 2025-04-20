use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};
use rand::{seq::IndexedRandom, Rng};
use serde::Deserialize;
use std::io::stdout;
use std::{thread, time::Duration};

#[derive(Deserialize)]
pub struct Metrics {
    pub server_data: Vec<Payload>,
}

#[derive(Deserialize)]
pub struct Payload {
    pub cpu: f64,
    pub ram: f64,
    pub netspeed: Vec<f64>,
}

pub struct Metric {
    pub cpu: f64,
    pub ram: f64,
    pub netspeed: (f64, f64),
    pub net_history: (Vec<f64>, Vec<f64>),
}

fn clear_screen() {
    let mut stdout = stdout();
    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0)).unwrap();
}

fn draw_bar_chart(values: &[f64], max_val: f64) -> String {
    let bars = ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];
    values
        .iter()
        .map(|&v| {
            let scaled = (v / max_val * (bars.len() as f64 - 1.0)).round();
            let idx = scaled.clamp(0.0, (bars.len() - 1) as f64) as usize;
            bars[idx]
        })
        .collect()
}

fn format_metrics(metrics: &Metric, title: &str) -> Vec<String> {
    let (upload_hist, download_hist) = &metrics.net_history;
    let max_up = upload_hist.iter().cloned().fold(0.0, f64::max).max(1.0);
    let max_down = download_hist.iter().cloned().fold(0.0, f64::max).max(1.0);

    let lines = vec![
        format!("{:^30}", title),
        format!("{:-<30}", ""),
        format!("CPU:     {:>6.1} %", metrics.cpu),
        format!("RAM:     {:>6.2} GB", metrics.ram),
        format!("NetSpeed:"),
        format!(
            "  Upload:   {:>6.2} Mbps   {}",
            metrics.netspeed.0,
            draw_bar_chart(upload_hist, max_up)
        ),
        format!(
            "  Download: {:>6.2} Mbps   {}",
            metrics.netspeed.1,
            draw_bar_chart(download_hist, max_down)
        ),
    ];
    let target_height = 8;
    let mut padded = lines;
    while padded.len() < target_height {
        padded.push("".to_string());
    }

    padded
}

fn merge_columns(columns: Vec<Vec<String>>) -> Vec<String> {
    let height = columns.iter().map(|col| col.len()).max().unwrap_or(0);
    (0..height)
        .map(|i| {
            columns
                .iter()
                .map(|col| col.get(i).cloned().unwrap_or_else(|| " ".repeat(30)))
                .collect::<Vec<_>>()
                .join("                 ")
        })
        .collect()
}

pub async fn render_dash(Json(payload): Json<Metrics>) -> impl IntoResponse {
    let mut upload_histories = vec![vec![0.0; 10]; 3];
    let mut download_histories = vec![vec![0.0; 10]; 3];
    let mut columns = Vec::new();
    let metrics = payload.server_data;
    let n = metrics.len();
    for i in 0..n {
        let cpu = metrics[i].cpu;
        let ram = metrics[i].ram;
        let upload = metrics[i].netspeed[0];
        let download = metrics[i].netspeed[1];

        let upload_hist = &mut upload_histories[i];
        let download_hist = &mut download_histories[i];

        upload_hist.push(upload);
        if upload_hist.len() > 10 {
            upload_hist.remove(0);
        }

        download_hist.push(download);
        if download_hist.len() > 10 {
            download_hist.remove(0);
        }

        let metrics = Metric {
            cpu,
            ram,
            netspeed: (upload, download),
            net_history: (upload_hist.clone(), download_hist.clone()),
        };

        columns.push(format_metrics(&metrics, &format!("System {}", i + 1)));
    }

    clear_screen();
    for line in merge_columns(columns) {
        println!("{}", line);
    }
}

pub async fn data_listener() {
    let app = Router::new().route("/data", post(render_dash));

    let address = "100.104.128.106:3000";
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("failed to listen...");
    println!("server is listening.....");
    axum::serve(listener, app).await.unwrap();
}
