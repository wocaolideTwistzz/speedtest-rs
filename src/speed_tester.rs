use std::{
    net::IpAddr,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use bytes::Bytes;
use futures::{Stream, StreamExt, TryStreamExt, stream};
use reqwest::{IntoUrl, header::CONTENT_LENGTH};
use serde::de::DeserializeOwned;

use crate::{
    model::{Config, Server, Servers},
    urls::SpeedTestUrl,
};

const UPLOAD_CHUNK: [u8; 1024 * 16] = [0; 1024 * 16];

#[derive(Debug, Clone)]
pub struct SpeedTester {
    urls: SpeedTestUrl,
    client: reqwest::Client,
    request_timeout: Duration,
    compare_times: usize,
    compare_interval: Duration,

    config: Option<Config>,
    server: Option<Server>,
}

impl Default for SpeedTester {
    fn default() -> Self {
        Self::new(reqwest::Client::default())
    }
}

impl SpeedTester {
    pub fn new(client: reqwest::Client) -> Self {
        Self {
            urls: SpeedTestUrl::default(),
            client,
            config: None,
            server: None,
            request_timeout: Duration::from_secs(10),
            compare_times: 3,
            compare_interval: Duration::from_millis(200),
        }
    }

    pub fn new_with_local_addr(local_addr: IpAddr) -> Self {
        let client = reqwest::Client::builder()
            .local_address(local_addr)
            .build()
            .unwrap();

        Self::new(client)
    }

    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        if self.config.is_some() && self.server.is_some() {
            tracing::debug!("SpeedTester already initialized.");
            return Ok(());
        }

        tracing::debug!("SpeedTester fetch config...");
        let config = self.fetch_config().await?;

        tracing::debug!("SpeedTester fetch config success {:?}", config);

        tracing::debug!("SpeedTester fetch servers...");
        let mut servers = self.fetch_servers(config.threads()).await?;
        tracing::debug!("SpeedTester fetch servers success {:?}", servers);

        self.filter_ignored_servers(&mut servers.servers.servers, &config);

        self.config = Some(config);

        let fastest_server = self.select_fastest_server(servers.servers.servers).await?;
        tracing::debug!(
            "SpeedTester select fastest server success: {:?}",
            fastest_server.url
        );

        self.server = Some(fastest_server);

        Ok(())
    }

    pub async fn do_download(&mut self, downloaded: Arc<AtomicU64>) -> anyhow::Result<()> {
        self.initialize().await?;

        let config = self.get_config()?;
        let server = self.get_server()?;

        self.download(config, server, downloaded).await;
        Ok(())
    }

    pub async fn do_upload(&mut self, uploaded: Arc<AtomicU64>) -> anyhow::Result<()> {
        self.initialize().await?;

        let config = self.get_config()?;
        let server = self.get_server()?;

        self.upload(config, server, uploaded).await;
        Ok(())
    }

    pub async fn fetch_config(&self) -> anyhow::Result<Config> {
        for url in self.urls.config_urls() {
            match self.get_xml(url).await {
                Ok(settings) => return Ok(settings),
                Err(e) => tracing::debug!("failed to fetch config: {}", e),
            }
        }

        anyhow::bail!("all fetch config failed")
    }

    pub async fn fetch_servers(&self, threads: usize) -> anyhow::Result<Servers> {
        let urls = self.urls.clone().threads(threads);
        for url in urls.server_urls() {
            match self.get_xml(url).await {
                Ok(servers) => return Ok(servers),
                Err(e) => tracing::debug!("failed to fetch servers: {}", e),
            }
        }
        anyhow::bail!("all fetch servers failed")
    }

    pub async fn select_fastest_server(&self, servers: Vec<Server>) -> anyhow::Result<Server> {
        if servers.is_empty() {
            anyhow::bail!("no servers");
        }

        let times = self.compare_times;
        let interval = self.compare_interval;
        let timeout = self.request_timeout;
        let (tx, mut rx) = tokio::sync::mpsc::channel(servers.len());
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

        for server in servers {
            let client = self.client.clone();
            let tx = tx.clone();
            let mut shutdown = shutdown_rx.clone();

            tokio::spawn(async move {
                let mut delay = Duration::default();
                for i in 0..times {
                    tokio::select! {
                        _ = shutdown.changed() => {
                            return;
                        }
                        current_delay = SpeedTester::get_server_delay(&client, &server, timeout) => {
                            delay += current_delay;
                        }
                    }
                    if i < times - 1 {
                        tokio::time::sleep(interval).await;
                    }
                }
                _ = tx.send((server, delay)).await;
            });
        }

        let (server, delay) = rx.recv().await.unwrap();

        if delay < timeout * 2 {
            _ = shutdown_tx.send(true);
            return Ok(server);
        }
        let mut server_delays = vec![];
        server_delays.push((server, delay));

        while let Some((server, delay)) = rx.recv().await {
            server_delays.push((server, delay));
        }

        server_delays.sort_by(|a, b| a.1.cmp(&b.1));

        if server_delays[0].1 < timeout * 2 * times as u32 {
            return Ok(server_delays[0].0.clone());
        }
        anyhow::bail!("all servers failed")
    }

    pub async fn download(&self, config: &Config, server: &Server, downloaded: Arc<AtomicU64>) {
        let seq = config.download_size_sequence();

        let max_download_count = config.download_count_per_url() * seq.len();

        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

        let tasks = stream::iter(0..max_download_count).for_each_concurrent(
            config.download_threads(),
            |i| {
                let size = seq[i % seq.len()];
                let url = format!("{}/random{}x{}.jpg", server.url, size, size);
                let client = self.client.clone();
                let downloaded = downloaded.clone();
                let shutdown = shutdown_rx.clone();

                async move { Self::single_download(client, url, downloaded, shutdown).await }
            },
        );

        tokio::select! {
            biased;
            _ = tokio::time::sleep(config.max_download_duration()) => {
                _ = shutdown_tx.send(true);
            }
            _ = tasks => {
            }
        }
    }

    pub async fn upload(&self, config: &Config, server: &Server, uploaded: Arc<AtomicU64>) {
        let seq = config.upload_size_sequence();

        let max_upload_count = config.max_upload_count();
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

        let tasks =
            stream::iter(0..max_upload_count).for_each_concurrent(config.upload_threads(), |i| {
                let size = seq[i % seq.len()];
                let url = server.url.clone();
                let client = self.client.clone();
                let uploaded = uploaded.clone();
                let shutdown = shutdown_rx.clone();

                async move { Self::single_upload(client, url, size, uploaded, shutdown).await }
            });

        tokio::select! {
            biased;
            _ = tokio::time::sleep(config.max_upload_duration()) => {
                _ = shutdown_tx.send(true);
            }
            _ = tasks => {
            }
        }
    }

    pub fn get_config(&self) -> anyhow::Result<&Config> {
        self.config.as_ref().ok_or(anyhow::anyhow!(
            "config is empty. maybe call initialize first"
        ))
    }

    pub fn get_server(&self) -> anyhow::Result<&Server> {
        self.server.as_ref().ok_or(anyhow::anyhow!(
            "server is empty. maybe call initialize first"
        ))
    }

    pub fn filter_ignored_servers(&self, servers: &mut Vec<Server>, config: &Config) {
        let ignore_ids = config.ignore_servers().collect::<Vec<_>>();

        servers.retain(|s| !ignore_ids.contains(&s.id.as_str()));
    }

    async fn get_xml<T, U>(&self, url: U) -> anyhow::Result<T>
    where
        T: DeserializeOwned,
        U: IntoUrl,
    {
        let resp = self
            .client
            .get(url)
            .timeout(self.request_timeout)
            .send()
            .await?;
        let status = resp.status();

        if status.is_success() {
            let xml = resp.text().await?;
            let ret: T = quick_xml::de::from_str(xml.as_str())?;
            Ok(ret)
        } else {
            anyhow::bail!("status: {}", status);
        }
    }

    async fn get_server_delay(
        client: &reqwest::Client,
        server: &Server,
        timeout: Duration,
    ) -> Duration {
        let start = Instant::now();

        match client.get(&server.url).timeout(timeout).send().await {
            Ok(resp) => {
                if resp.bytes().await.is_ok() {
                    return start.elapsed();
                }
            }
            Err(e) => {
                tracing::debug!("get server delay for {} failed: {}", server.url, e);
            }
        }
        timeout * 2
    }

    async fn single_download(
        client: reqwest::Client,
        url: String,
        downloaded: Arc<AtomicU64>,
        mut shutdown: tokio::sync::watch::Receiver<bool>,
    ) {
        let mut resp = match client
            .get(&url)
            .header("user-agent", "SPEED-TESTER-RS")
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                tracing::debug!("download {} failed: {}", url, e);
                return;
            }
        };

        tokio::select! {
            biased;
            _ = shutdown.changed() => {
            }
            _ = async {
                while let Ok(Some(chunk)) = resp.chunk().await {
                    _ = downloaded.fetch_add(chunk.len() as u64, Ordering::Relaxed);
                }
            } => {}
        }
    }

    async fn single_upload(
        client: reqwest::Client,
        url: String,
        size: usize,
        uploaded: Arc<AtomicU64>,
        mut shutdown: tokio::sync::watch::Receiver<bool>,
    ) {
        let body = Self::create_zero_stream(size, uploaded);

        tokio::select! {
            biased;
            _ = shutdown.changed() => {
            }
            _ = client
                .post(url)
                .body(reqwest::Body::wrap_stream(body))
                .header(CONTENT_LENGTH, size)
                .send() => {}
        }
        // client.post(url).body(body)
    }

    fn create_zero_stream(
        size: usize,
        uploaded: Arc<AtomicU64>,
    ) -> impl Stream<Item = Result<Bytes, std::io::Error>> {
        stream::unfold(size, |remaining| async move {
            if remaining == 0 {
                None
            } else {
                let chunk_size = remaining.min(UPLOAD_CHUNK.len());

                let chunk = Bytes::from_static(&UPLOAD_CHUNK[..chunk_size]);

                let next_state = remaining - chunk_size;

                Some((Ok(chunk), next_state))
            }
        })
        .inspect_ok(move |chunk| {
            uploaded.fetch_add(chunk.len() as u64, Ordering::Relaxed);
        })
    }
}

#[cfg(test)]
mod tests {

    use std::sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    };

    use futures::StreamExt;

    use crate::speed_tester::SpeedTester;

    #[tokio::test]
    async fn test_create_zero_stream() {
        let size = 16 * 16 * 1025;
        let recorded = Arc::new(AtomicU64::new(0));
        let mut total = 0;
        let mut bytes_stream = Box::pin(SpeedTester::create_zero_stream(size, recorded.clone()));

        while let Some(Ok(chunk)) = bytes_stream.next().await {
            total += chunk.len();
        }

        assert_eq!(total, size);
        assert_eq!(recorded.load(Ordering::Relaxed), size as u64);
    }
}
