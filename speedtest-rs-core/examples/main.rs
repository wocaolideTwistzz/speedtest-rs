use std::{
    sync::{Arc, atomic::AtomicU64},
    time::Instant,
};

use speedtest_rs_core::{Humanize, speed_tester::SpeedTester};

#[tokio::main]
pub async fn main() {
    let mut speed_tester = SpeedTester::default();

    let start = Instant::now();

    speed_tester.initialize().await.expect("initialize failed");

    let download_start = Instant::now();
    println!(
        "Initialize success. cost: {}ms",
        download_start.duration_since(start).as_millis()
    );

    let downloaded = Arc::new(AtomicU64::new(0));
    speed_tester
        .do_download(downloaded.clone())
        .await
        .expect("do download failed");

    let upload_start = Instant::now();

    let download_elapsed_ms = upload_start.duration_since(download_start).as_millis();
    let download_bytes = downloaded.load(std::sync::atomic::Ordering::SeqCst) as usize;

    println!(
        "Download success. cost: {}ms | total: {} | bps: {}",
        download_elapsed_ms,
        download_bytes.humanize_bytes(),
        download_bytes.humanize_bitrate(download_elapsed_ms as u64),
    );

    let uploaded = Arc::new(AtomicU64::new(0));
    speed_tester
        .do_upload(uploaded.clone())
        .await
        .expect("do upload failed");

    let upload_elapsed_ms = upload_start.elapsed().as_millis();
    let upload_bytes = uploaded.load(std::sync::atomic::Ordering::SeqCst) as usize;

    println!(
        "Upload success. cost: {}ms | total: {} | bps: {}",
        upload_elapsed_ms,
        upload_bytes.humanize_bytes(),
        upload_bytes.humanize_bitrate(upload_elapsed_ms as u64),
    );
}
