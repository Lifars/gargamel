use clap::Clap;

#[derive(Clap, Clone)]
#[clap(version = "1.0", author = "LIFARS LLC")]
pub struct Opts {
    /// Remote computer address/name. It may be also a path to a file with list of addresses/names (one per line).
    #[clap(
    short = 'c',
    long = "computer",
    default_value = "127.0.0.1"
    )]
    pub computer: String,

    /// Remote user name.
    #[clap(
    short = 'u',
    long = "user"
    )]
    pub user: Option<String>,

    /// Optional: Remote Windows domain.
    #[clap(
    short = 'd',
    long = "domain"
    )]
    pub domain: Option<String>,

    /// Optional: Remote user password. Skipping this option will prompt for a password during start. To specify an empty password use `-p ""`.
    #[clap(
    short = 'p',
    long = "password"
    )]
    pub password: Option<String>,

    /// Name of local directory to store the evidence.
    #[clap(
    short = 'o',
    long = "output",
    default_value = "evidence-output"
    )]
    pub local_store_directory: String,

    /// Name of remote directory to be used as a temporary storage (Windows targets only).
    #[clap(
    short = 'r',
    long = "remote-storage",
    default_value = "C:\\Users\\Public"
    )]
    pub remote_store_directory: String,

    /// Optional: File with custom commands to execute on a remote computer.
    #[clap(
    short = 'e',
    long = "commands"
    )]
    pub custom_command_path: Option<String>,

    /// Optional: File with files names to be searched on a remote computer. File names support also `*` and `?` wildcards on file names (but not yet parent directories).
    #[clap(
    short = 's',
    long = "search"
    )]
    pub search_files_path: Option<String>,

    /// Disables acquisition of evidence that can be usually downloaded quickly (like ipconfig, firewall status etc.)
    #[clap(
    long = "no-evidence-search"
    )]
    pub disable_evidence_download: bool,

    /// Disables target registry acquisition.
    #[clap(
    long = "no-registry-search"
    )]
    pub disable_registry_download: bool,

    /// Disables Windows event logs acquisition.
    #[clap(
    long = "no-events-search"
    )]
    pub disable_event_download: bool,

    /// Acquire evidence from Windows machine using all supported methods (PsExec, PsRemote, WMI, RDP).
    #[clap(
    short = 'a',
    long = "all"
    )]
    pub all: bool,

    /// Acquire evidence from a remote Windows machine using WMI.
    /// Requires WMImplant.ps1 in the current directory or in the path and PowerShell 3.0+ on the host machine.
    /// Note: It is necessary to disable Windows Defender real-time protection (other AVs not tested).
    #[clap(
    long = "wmi"
    )]
    pub wmi: bool,

    /// Acquire evidence from a remote Windows machine using RDP.
    /// Requires SharpRDP.exe in the current directory or in the path.
    #[clap(
    long = "rdp"
    )]
    pub rdp: bool,

    /// Acquire evidence from a remote Windows machine using PsExec.
    /// Requires both PsExec64.exe and paexec.exe in the current directory or in the path.
    #[clap(
    long = "psexec"
    )]
    pub psexec64: bool,

    /// Acquire evidence from a remote Windows machine using 32 bit PsExec.
    /// Requires both PsExec.exe and paexec.exe in the current directory or in the path.
    #[clap(
    long = "psexec32"
    )]
    pub psexec32: bool,

    /// "Acquire evidence a remote from Windows machine using PowerShell."
    #[clap(
    long = "psrem"
    )]
    pub psrem: bool,

    /// Acquire evidence from a remote Linux machine using SSH.
    /// Requires both plink.exe and pscp.exe in the current directory or in the path.
    #[clap(
    long = "ssh"
    )]
    pub ssh: bool,

    /// Optional: Memory dump of a target Windows machine.
    #[clap(
    short = 'm',
    long = "mem-image"
    )]
    pub image_memory: bool,

    /// Optional: Timeout in seconds for long running operations.
    /// This option is a workaround for a bug in WMImplant.ps1 amd SharpRDP.exe where finishing of a long running operation cannot sometimes properly close the connection.
    /// This leaves the Gargamel in a seemingly frozen state or it may execute the next operation prematurely.
    /// Increasing this timeout may solve issues when acquiring registry or memory image from remote machines.
    #[clap(
    long = "timeout",
    default_value = "300",
    )]
    pub timeout: u64,

    /// Optional: Path to kape configs to be converted and used
    #[clap(long = "use-kape-config")]
    pub kape_config_path: Option<String>,

    /// Optional: Name/path of a SSH private key file. (Linux target only)
    #[clap(long = "key")]
    pub ssh_key: Option<String>,

    /// Optional: Use network level authentication when using RDP. (Windows targets only)
    #[clap(long = "nla")]
    pub nla: bool,

    /// Optional: Disable 7zip compression for registry & memory images.
    /// This will significantly decrease the running time, but WMI and RDP connections will probably not work properly.
    /// (Windows targets only)
    #[clap(long = "no-7z")]
    pub no_compression: bool,

    /// Optional: Download and DELETE specified file from the target computer.
    /// Use this in case of previous failed partially completed operation.
    /// For just downloading a file (without deleting it) please use a `search` switch instead.
    /// If you specify a 7zip chunk (.7z.[chunk-number], e.g. .7z.004), then it will also automatically try to download subsequent chunks.
    /// Use also with --psexec --psrem, --rdp, --wmi, --all
    #[clap(long = "redownload")]
    pub re_download: Option<String>,

    /// Optional: Experimental. Enable parallelism when connecting to more remote computers.
    #[clap(
    long = "in-parallel"
    )]
    pub par: bool
}

