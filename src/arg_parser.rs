use clap::Clap;

#[derive(Clap, Clone)]
#[clap(version = "1.0", author = "LIFARS LLC")]
pub struct Opts {
    #[clap(
    short = "c",
    long = "computer",
    default_value = "127.0.0.1",
    help = "Remote computer address/name. Not setting this equals 127.0.0.1"
    )]
    pub computer: String,

    #[clap(
    short = "u",
    long = "user",
    help = "Remote user name"
    )]
    pub user: String,

    #[clap(
    short = "d",
    long = "domain",
    help = "Optional: Remote domain"
    )]
    pub domain: Option<String>,

    #[clap(
    short = "p",
    long = "password",
    help = "Optional: Remote user password"
    )]
    pub password: Option<String>,
    #[clap(
    short = "o",
    long = "output",
    default_value = "program-output",
    help = "Remote user password"
    )]
    pub store_directory: String,

    #[clap(
    short = "e",
    long = "commands",
    help = "Optional: File with custom commands to execute on remote computer"
    )]
    pub custom_command_path: Option<String>,

    #[clap(
    short = "s",
    long = "search",
    help = "Optional: File with files names to be searched on remote computer. \
    File names supports also `*` and `?` wildcards."
    )]
    pub search_files_path: Option<String>,

    #[clap(
    long = "no-evidence-search",
    help = "Disables acquisition of evidence that can be usually downloaded quickly (like ipconfig, firewall status etc.)"
    )]
    pub disable_evidence_download: bool,

    #[clap(
    long = "no-registry-search",
    help = "Disables registry acquisition"
    )]
    pub disable_registry_download: bool,

    #[clap(
    long = "no-events-search",
    help = "Disables event acquisition"
    )]
    pub disable_event_download: bool,

    #[clap(short = "a", long = "all")]
    pub all: bool,

    #[clap(long = "wmi")]
    pub wmi: bool,

    #[clap(long = "rdp")]
    pub rdp: bool,

    #[clap(long = "psexec")]
    pub psexec: bool,

    #[clap(long = "psrem")]
    pub psrem: bool,

    #[clap(long = "ssh")]
    pub ssh: bool,

    #[clap(long = "local")]
    pub local: bool,

    #[clap(
    short = "m",
    long = "mem-image",
    help = "Optional: Memory dump name"
    )]
    pub image_memory: Option<String>,

    #[clap(
    long = "timeout",
    help = "Optional: Timeout of long running operations. Default is 300 (seconds)",
    default_value = "300",
    )]
    pub timeout: u64,

    #[clap(
    long = "compress_timeout",
    help = "Optional: Timeout of memory image compression running operations. Default is 400 (seconds). Should be around `target_mem_size_in_mb/5`",
    default_value = "400",
    )]
    pub compress_timeout: u64,

    #[clap(long = "key", help = "Optional: SSH private key file")]
    pub ssh_key: Option<String>,

    #[clap(long = "nla", help = "Optional: Use network level authentication for RDP")]
    pub nla: bool,

    #[clap(long = "no-7z", help = "Optional: Disable 7zip compression for registry & memory images.")]
    pub no_compression: bool,

    #[clap(long = "redownload", help =
    "Optional: Download and DELETE specified file from target computer. \
    Use in case of previous failed partially completed operation. \
    For just downloading a file please use a `search` switch.
    If you specify a 7zip chunk (.7z.[chunk-number]), then it will also automatically try to download \
    subsequent chunks.\
    Use with --psexec --psrem, --rdp, --wmi, --all options")]
    pub re_download: Option<String>
}

