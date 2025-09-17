pub mod model;
pub mod speed_tester;
pub mod urls;

// Bytes
pub const KB: usize = 1024;
pub const MB: usize = 1024 * KB;
pub const GB: usize = 1024 * MB;
pub const TB: usize = 1024 * GB;

// Bitrate
pub const K_BITS_PER_SEC: usize = 1000;
pub const M_BITS_PER_SEC: usize = 1000 * K_BITS_PER_SEC;
pub const G_BITS_PER_SEC: usize = 1000 * M_BITS_PER_SEC;
pub const T_BITS_PER_SEC: usize = 1000 * G_BITS_PER_SEC;

pub trait Humanize {
    fn humanize_bytes(&self) -> String;

    fn humanize_bitrate(&self, duration_millis: u64) -> String;

    fn humanize(&self) -> (f64, usize);
}

impl Humanize for usize {
    fn humanize_bytes(&self) -> String {
        let bytes = *self;
        if bytes < KB {
            format!("{bytes} Bytes")
        } else if bytes < MB {
            format!("{:.2} KBytes", bytes as f64 / KB as f64)
        } else if bytes < GB {
            format!("{:.2} MBytes", bytes as f64 / MB as f64)
        } else if bytes < TB {
            format!("{:.2} GBytes", bytes as f64 / GB as f64)
        } else {
            format!("{:.2} TBytes", bytes as f64 / TB as f64)
        }
    }

    fn humanize_bitrate(&self, duration_millis: u64) -> String {
        let bytes = *self;
        // For higher accuracy we are getting the actual millis of the duration rather than the
        // rounded seconds.
        let bits = bytes * 8;
        // rate as fraction in seconds;
        let rate = (bits as f64 / duration_millis as f64) * 1000f64;
        if rate < K_BITS_PER_SEC as f64 {
            format!("{rate} Bits/sec")
        } else if bytes < M_BITS_PER_SEC {
            format!("{:.2} Kbits/sec", rate / K_BITS_PER_SEC as f64)
        } else if bytes < G_BITS_PER_SEC {
            format!("{:.2} Mbits/sec", rate / M_BITS_PER_SEC as f64)
        } else if bytes < T_BITS_PER_SEC {
            format!("{:.2} Gbits/sec", rate / G_BITS_PER_SEC as f64)
        } else {
            format!("{:.2} Tbits/sec", rate / T_BITS_PER_SEC as f64)
        }
    }

    fn humanize(&self) -> (f64, usize) {
        let bytes = *self;
        if bytes < KB {
            (bytes as f64, 1)
        } else if bytes < MB {
            (bytes as f64 / KB as f64, KB)
        } else if bytes < GB {
            (bytes as f64 / MB as f64, MB)
        } else if bytes < TB {
            (bytes as f64 / GB as f64, GB)
        } else {
            (bytes as f64 / TB as f64, TB)
        }
    }
}
