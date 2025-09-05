#[derive(Debug, Default, Clone)]
pub struct SpeedTestUrl {
    use_tls: bool,

    threads: usize,
}

impl SpeedTestUrl {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn use_tls(mut self, use_tls: bool) -> Self {
        self.use_tls = use_tls;
        self
    }

    pub fn threads(mut self, threads: usize) -> Self {
        self.threads = threads;
        self
    }

    pub fn config_urls(&self) -> impl Iterator<Item = String> {
        SpeedTestHost::all().into_iter().map(|host| {
            if self.use_tls {
                format!("https://{}{}", host.host(), SpeedTestPath::Config.path())
            } else {
                format!("http://{}{}", host.host(), SpeedTestPath::Config.path())
            }
        })
    }

    pub fn server_urls(&self) -> impl Iterator<Item = String> {
        SpeedTestHost::all().into_iter().flat_map(move |host| {
            SpeedTestPath::servers().into_iter().map(move |path| {
                let scheme = if self.use_tls { "https" } else { "http" };
                if self.threads > 0 {
                    format!(
                        "{}://{}{}?threads={}",
                        scheme,
                        host.host(),
                        path.path(),
                        self.threads
                    )
                } else {
                    format!("{}://{}{}", scheme, host.host(), path.path())
                }
            })
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SpeedTestHost {
    Main,
    Backup,
}

impl SpeedTestHost {
    pub fn all() -> [SpeedTestHost; 2] {
        [SpeedTestHost::Main, SpeedTestHost::Backup]
    }

    pub const fn host(&self) -> &'static str {
        match self {
            SpeedTestHost::Main => "www.speedtest.net",
            SpeedTestHost::Backup => "c.speedtest.net",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SpeedTestPath {
    Config,
    Server,
    ServerStatic,
}

impl SpeedTestPath {
    pub fn servers() -> [SpeedTestPath; 2] {
        [SpeedTestPath::Server, SpeedTestPath::ServerStatic]
    }

    pub const fn path(&self) -> &'static str {
        match self {
            SpeedTestPath::Config => "/speedtest-config.php",
            SpeedTestPath::Server => "/speedtest-servers.php",
            SpeedTestPath::ServerStatic => "/speedtest-servers-static.php",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::urls::SpeedTestUrl;

    #[test]
    fn test_config_urls() {
        let urls: Vec<_> = SpeedTestUrl::new().use_tls(true).config_urls().collect();

        assert_eq!(
            urls,
            vec![
                "https://www.speedtest.net/speedtest-config.php",
                "https://c.speedtest.net/speedtest-config.php",
            ]
        );
    }

    #[test]
    fn test_servers_urls() {
        let urls: Vec<_> = SpeedTestUrl::new()
            .use_tls(true)
            .threads(5)
            .server_urls()
            .collect();

        assert_eq!(
            urls,
            vec![
                "https://www.speedtest.net/speedtest-servers.php?threads=5",
                "https://www.speedtest.net/speedtest-servers-static.php?threads=5",
                "https://c.speedtest.net/speedtest-servers.php?threads=5",
                "https://c.speedtest.net/speedtest-servers-static.php?threads=5",
            ]
        );
    }
}
