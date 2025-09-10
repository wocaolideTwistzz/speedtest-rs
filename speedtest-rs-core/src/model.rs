use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub client: Client,
    #[serde(rename = "server-config")]
    pub server_config: ServerConfig,
    pub download: Download,
    pub upload: Upload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Servers {
    #[serde(rename = "servers")]
    pub servers: ServerList,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerList {
    #[serde(rename = "server")]
    pub servers: Vec<Server>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    #[serde(rename = "@url")]
    pub url: String,
    #[serde(rename = "@lat")]
    pub lat: f64,
    #[serde(rename = "@lon")]
    pub lon: f64,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@country")]
    pub country: String,
    #[serde(rename = "@cc")]
    pub cc: String,
    #[serde(rename = "@sponsor")]
    pub sponsor: String,
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@host")]
    pub host: String,
}

impl Config {
    pub fn client_info(&self) -> &Client {
        &self.client
    }

    pub fn ignore_servers(&self) -> impl Iterator<Item = &str> {
        self.server_config.ignoreids.split(',')
    }

    pub fn upload_size_sequence(&self) -> Vec<usize> {
        let mut seq = DefaultSequence::Upload.sequence();

        let ratio = self.upload.ratio as usize;
        if ratio > 0 && ratio < seq.len() {
            seq.drain(0..ratio - 1);
        }
        seq
    }

    pub fn max_download_duration(&self) -> Duration {
        Duration::from_secs(self.download.testlength as u64)
    }

    pub fn max_upload_duration(&self) -> Duration {
        Duration::from_secs(self.upload.testlength as u64)
    }

    pub fn download_size_sequence(&self) -> Vec<usize> {
        DefaultSequence::Download.sequence()
    }

    pub fn threads(&self) -> usize {
        self.server_config.threadcount as usize * 2
    }

    pub fn download_threads(&self) -> usize {
        self.server_config.threadcount as usize * 2
    }

    pub fn download_count_per_url(&self) -> usize {
        self.download.threadsperurl as usize
    }

    pub fn upload_threads(&self) -> usize {
        self.upload.threads as usize
    }

    pub fn upload_count_per_url(&self) -> usize {
        self.upload.threadsperurl as usize
    }

    pub fn max_upload_count(&self) -> usize {
        self.upload.maxchunkcount as usize
    }
}

pub enum DefaultSequence {
    Upload,
    Download,
}

impl DefaultSequence {
    pub fn sequence(&self) -> Vec<usize> {
        match self {
            DefaultSequence::Upload => vec![
                32 * 1024,
                64 * 1024,
                128 * 1024,
                256 * 1024,
                512 * 1024,
                1024 * 1024,
                7 * 1024 * 1024,
            ],
            DefaultSequence::Download => {
                vec![350, 500, 750, 1000, 1500, 2000, 2500, 3000, 3500, 4000]
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    #[serde(rename = "@ip")]
    pub ip: String,
    #[serde(rename = "@lat")]
    pub lat: f64,
    #[serde(rename = "@lon")]
    pub lon: f64,
    #[serde(rename = "@isp")]
    pub isp: String,
    #[serde(rename = "@isprating")]
    pub isprating: f32,
    #[serde(rename = "@rating")]
    pub rating: f32,
    #[serde(rename = "@ispdlavg")]
    pub ispdlavg: f32,
    #[serde(rename = "@ispulavg")]
    pub ispulavg: f32,
    #[serde(rename = "@loggedin")]
    pub loggedin: u8,
    #[serde(rename = "@country")]
    pub country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(rename = "@threadcount")]
    pub threadcount: u32,
    #[serde(rename = "@ignoreids")]
    pub ignoreids: String,
    #[serde(rename = "@notonmap")]
    pub notonmap: String,
    #[serde(rename = "@forcepingid")]
    pub forcepingid: String,
    #[serde(rename = "@preferredserverid")]
    pub preferredserverid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Download {
    #[serde(rename = "@testlength")]
    pub testlength: u32,
    #[serde(rename = "@initialtest")]
    pub initialtest: String,
    #[serde(rename = "@mintestsize")]
    pub mintestsize: String,
    #[serde(rename = "@threadsperurl")]
    pub threadsperurl: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Upload {
    #[serde(rename = "@testlength")]
    pub testlength: u32,
    #[serde(rename = "@ratio")]
    pub ratio: f32,
    #[serde(rename = "@initialtest")]
    pub initialtest: String,
    #[serde(rename = "@mintestsize")]
    pub mintestsize: String,
    #[serde(rename = "@threads")]
    pub threads: u32,
    #[serde(rename = "@maxchunksize")]
    pub maxchunksize: String,
    #[serde(rename = "@maxchunkcount")]
    pub maxchunkcount: u32,
    #[serde(rename = "@threadsperurl")]
    pub threadsperurl: u32,
}

#[cfg(test)]
mod tests {
    const RAW_CONFIG: &str = r#"
<settings>
<client ip="1.1.1.1" lat="65.2842" lon="11.1759" isp="Value" isprating="3.7" rating="0" ispdlavg="0" ispulavg="0" loggedin="0" country="HK"/>
<server-config threadcount="4" ignoreids="683,1525,1716,1758,1762,1816,1834,1839,1840,1850,1854,1859,1860,1861,1871,1873,1875,1880,1902,1913,3280,3383,3448,3695,3696,3697,3698,3699,3725,3726,3727,3728,3729,3730,3731,3733,3788,4140,4533,5085,5086,5087,5894,6130,6285,6397,6398,6412,7326,7334,7529,8591,9123,9466,9816,10221,10226,10556,10557,10558,10561,10562,10563,10564,10565,10566,10567,10901,10923,11201,11736,11737,11792,12688,12689,12861,12862,12863,13362,14209,14445,14446,14448,14804,14805,14806,14807,14808,14809,14810,14811,14812,14813,14814,14880,14881,14882,14883,14884,14908,14909,14910,14911,14946,14972,14981,14982,14983,14984,14985,15012,15030,15034,15035,15036,15037,15079,15080,15081,15115,15181,15182,15262,15316,15359,15668,15845,15949,15950,15951,15952,15953,15954,15955,15956,15957,16030,16136,16275,16340,949,5249" notonmap="10588,16148,13544,11787,10299,4139,4247,5718,10309,4810,11076,12549,4231,14771,12776,15929,15669,2690,6051,16125,15715,5654,4674,2772,7594,2151,7322,8009,8577,1111,4745,5844,1930,6616,5681,7008,11424,6387,7531,10045,6307,10366,8486,11266,6827,6053,6562,12363,8367,4984,9321,15418,15252,13185,10025,9927,2724,5953,2632,10469,2557,4046,6430,5950,6389,11071,4730,11789,9100,3704,4521,4716,3326,4268,7532,10465,14091,1543,6118,9182,12506,11365,10258,5211,13040,9913,16534,15850,13291,4541,1894,6440,4734,13061,2636,10586,14678,15315,4166,15370,3497,15213,2665,5059,7619,5284,10425,6115,11547,10026,6032,8066,8281,10408,12920,7352,7437,5911,12260,16152,15815,16598,6151,2802,3206,2822,5326,6522,15569,5121,6446,8478,5447,9264,12212,6010,6578,6195,7743,16549,2518,9332,8040,16678,12974,10053,9788,8068,4424,11181,8223,8497,9015,3077,15936,8218,15263,16691,16326,8836,12746,1688,10265,11977,16476,15586,10176,11202,6260,6070,9084,3199,6635,15904,16472,16161,3282,4580,4768,13178,11329,6029,8879,6057,15567,5904,7842,16261,13984,9281,4149,16522,16141,2796,7635,10744,5708,11983,16252,13667,6689,4393,12154,12723,1993,11776,8211,5502,3997,10264,11675,5623,16636,9641,2477,1008,10703,11579,14476,4290,7609,9836,6049,4089,16067,9709,4590,11608,3025,3860,2565,9911,5210,826,4049,13238,13454,7537,14089,7829,5818,13898,2978,10290,9035,6316,10097,6375,7440,5299,10754,2214,6973,11348,15808,14761,395,14671,6485,16505,3676,2605,10050,12012,5919,3084,7193,11750,15912,15809,15833,2254,15199,2452,8471,4667,4791,13873,12619,3287,12536,9869,16134,7323,10261,11250,8863,12557,10116,12927,1781,16251,10591,10546,7672,7170,11637,7546,16149,6903,9493,6583,12283,8109,12961,10897,8416,2268,12417,4168,15343,15326,7983,4987,10152,4332,3964,7192,13628,16190,8404,16157,15251,12031,15831,8228,15786,15664,10667,4349,15851,7970,6286,16510,7640,6272,8339,5098,13583,6855,5031,5303,7244,5861,3883,10269,6248,6047,3864,7456,13516,13301,15671,5905,9916,15149,7048,7190,10412,6570,11383,11953,8169,3595,6561,13655,12407,13653,3165,15853,9887,8707,1182,4889,8695,15783,16331,12078,3567,9003,12244,3501,12176,12823,9384,10142,16615,2222,3529,15901,13635,4512,6257,3859,868,7024,5368,5495,16139,11210,10614,13153,10444,5727,8882,7382,9334,10801,8956,12497,12252,8906,10193,11876,8631,15797,15992,15028,16007,9089,16475,9570,10839,15630,7236,13108,12384,15376,13281,6431,10651,6932,12060,9450,15973,11662,10095,15938,16122,15781,6782,5942,7946,8625,12582,2181,2327,7429,2133,5660,11704,12583,5334,6401,9118,2253,8148,15131,7687,2182,4939,4947,3841,7469,12000,2192,2591,4940,5854,8628,5164,7147,10424,9096,3458,15718,11928,4588,5260,4728,7810,4773,12204,12588,8715,8002,12589,8629,12590,2189,11713,15697,16235,7198,7507,9174,6085,11194,2552,11430,6342,7215,8156,10178,13675,10603,8935,5415,12694,8079,7605,9049,6454,15886,12775,4883,15224,5060,8659,12843,8978,10800,7950,16124,9584,6403,6746,15747,16092,6675,10637,13384,16256,3984,5779,12951,12977,10631,9995,15193,5079,5311,12137,4491,7128,6749,11322,16487,6261,9965,5329,6683,14059,4693,9724,16221,15872,8046,7521,10685,1169,7662,11528,5394,7115,11823,5609,12443,16296,15492,9830,5431,6283,2459,9690,10077,9799,8610,8797,12182,9556,11117,11562,7231,15535,2583,6243,5376,9591,11413,10481,16462,10346,4663,6432,11014,7784,15530,11558,9282,6756,15493,16557,10204,15466,9966,10491,12973,15611,8615,10798,2504,12409,13093,16460,11248,10574,1452,8760,6610,9509,3281,13280,13459,15810,10599,12995,2427,2515,11342,12473,8491,12635,9668,9994,12208,7295,3328,4836,13892,12459,6618,8632,12942,4306,15278,7698,2617,6858,13612,3917,9652,11683,11850,10327,4956,8700,5935,5168,4235,5304,7556,10769,11360,6341,7046,16575,5048,10518,8674,11435,11255,8229,8291,11102,7152,8288,13457,8548,13569,12984,15560,6370,16013,10517,16348,12470,15854,13077,10780,10352,6976,13398,10421,11118,7582,9856,282,6612,11033,8874,5356,10239,13954,5248,11052,16039,7410,12936,12909,7680,10730,3174,4506,2329,6480,11319,5909,12273,8261,15711,12207,7292,12197,7370,4958,12006,4909,10370,9044,16546,15344,13874,9104,7267,15085,15467,15462,12045,6535,11960,8370,11937,2693,11840,12561,13566,15254,10454,9221,8927,16082,7059,9218,10587,11735,13582,15443,9767,7201,15983,15836,11012,8152,11995,15958,8719,11996,12178,11314,9266,12332,13043,13029,11499,11175,11064,9339,11211,11549,4133,6773,13982,6533,15933,15071,11488,9000,15798,15147,12322,16438,12996,12668,4450,5281,7254,9227,10990,6246,11310,8732,15532,10312,13229,12109,5889,13780,12088,16373,16660,16203,10333,2963,1714,6200,12018,16371,13278,2171,16208,8017,1858,9222,13065,12732,16060,16041,2582,2173,3505,5744,11767,13474,5666,14329,13425,15899,16429,10893,8894,5921,2962,7318,5868,13921,6093,12373,8453,16195,11480,900,6825,5181,4336,16640,234,4051,16374,5074,8855,7393,13676,5539,12932,10179,5749,5469,9974,8345,9345,12440,16592,6141,4052,13057,15911" forcepingid="" preferredserverid=""/>
<licensekey>f7a45ced624d3a70-1df5b7cd427370f7-b91ee21d6cb22d7b</licensekey>
<customer>speedtest</customer>
<odometer start="19046241464" rate="10"/>
<times dl1="5000000" dl2="35000000" dl3="800000000" ul1="1000000" ul2="8000000" ul3="35000000"/>
<download testlength="10" initialtest="250K" mintestsize="250K" threadsperurl="4"/>
<upload testlength="10" ratio="5" initialtest="0" mintestsize="32K" threads="2" maxchunksize="512K" maxchunkcount="50" threadsperurl="4"/>
<latency testlength="10" waittime="50" timeout="20"/>
<socket-download testlength="15" initialthreads="4" minthreads="4" maxthreads="32" threadratio="750K" maxsamplesize="5000000" minsamplesize="32000" startsamplesize="1000000" startbuffersize="1" bufferlength="5000" packetlength="1000" readbuffer="65536"/>
<socket-upload testlength="15" initialthreads="dyn:tcpulthreads" minthreads="dyn:tcpulthreads" maxthreads="32" threadratio="750K" maxsamplesize="1000000" minsamplesize="32000" startsamplesize="100000" startbuffersize="2" bufferlength="1000" packetlength="1000" disabled="false"/>
<socket-latency testlength="10" waittime="50" timeout="20"/>
<translation lang="xml"> </translation>
</settings>"#;

    const RAW_SERVERS: &str = r#"
<settings>
<servers>
<server url="http://kami.smartone.com:8080/speedtest/upload.php" lat="22.2796" lon="114.1592" name="Hong Kong" country="Hong Kong" cc="HK" sponsor="SmarTone" id="35791" host="kami.smartone.com:8080"/>
<server url="http://ookla-speedtest-central.hgconair.hgc.com.hk:8080/speedtest/upload.php" lat="22.2800" lon="114.1588" name="Hong Kong" country="Hong Kong" cc="HK" sponsor="HGC環電" id="37390" host="ookla-speedtest-central.hgconair.hgc.com.hk:8080"/>
<server url="http://speedtest21.hkbn.net:8080/speedtest/upload.php" lat="22.2500" lon="114.1667" name="Hong Kong" country="Hong Kong" cc="HK" sponsor="HKBN" id="65463" host="speedtest21.hkbn.net:8080"/>
<server url="http://speedtest1c.hkix.net:8080/speedtest/upload.php" lat="22.2500" lon="114.1667" name="Hong Kong" country="Hong Kong" cc="HK" sponsor="HKIX" id="61296" host="speedtest1c.hkix.net:8080"/>
<server url="http://lg-hkg.fdcservers.net:8080/speedtest/upload.php" lat="22.2500" lon="114.1667" name="Hong Kong" country="Hong Kong" cc="HK" sponsor="fdcservers.net" id="28912" host="lg-hkg.fdcservers.net:8080"/>
<server url="http://speedtest.hkg01.node.as9516.com:8080/speedtest/upload.php" lat="22.2500" lon="114.1667" name="Hong Kong" country="Hong Kong" cc="HK" sponsor="SAKURA LINK LTD" id="71487" host="speedtest.hkg01.node.as9516.com:8080"/>
<server url="http://mtelspeedtest1.cyberpig.tech:8080/speedtest/upload.php" lat="22.1987" lon="113.5439" name="Macao" country="Macau" cc="MO" sponsor="MTel Telecommunication Company Ltd." id="71541" host="mtelspeedtest1.cyberpig.tech:8080"/>
<server url="http://ks-speedtest.homeplus.net.tw:8080/speedtest/upload.php" lat="22.6333" lon="120.2667" name="Kaohsiung" country="Taiwan" cc="TW" sponsor="Homeplus" id="8968" host="ks-speedtest.homeplus.net.tw:8080"/>
<server url="http://txg-speedtest.infinirc.com:8080/speedtest/upload.php" lat="24.1471" lon="120.6085" name="Taichung" country="Taiwan" cc="TW" sponsor="Infinirc" id="69301" host="txg-speedtest.infinirc.com:8080"/>
<server url="http://tc1.chtm.hinet.net:8080/speedtest/upload.php" lat="24.1500" lon="120.6667" name="Taichung" country="Taiwan" cc="TW" sponsor="Chunghwa Mobile" id="18456" host="tc1.chtm.hinet.net:8080"/>
</servers>
</settings>"#;

    #[test]
    fn test_deserialize_config() {
        use crate::model::Config;

        let setting: Config = quick_xml::de::from_str(RAW_CONFIG).unwrap();

        println!("{:#?}", setting);

        println!("{:#?}", setting.ignore_servers().collect::<Vec<_>>());

        println!("{:#?}", setting.download_size_sequence());

        println!("{:#?}", setting.upload_size_sequence());
    }

    #[test]
    fn test_deserialize_servers() {
        use crate::model::Servers;

        let _servers: Servers = quick_xml::de::from_str(RAW_SERVERS).unwrap();
    }
}
