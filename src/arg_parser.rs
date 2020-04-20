use clap::Clap;

#[derive(Clap, Clone)]
#[clap(version = "1.0", author = "LIFARS LLC")]
pub struct Opts {
    #[clap(
    short = "c",
    long = "computer",
    default_value = "127.0.0.1",
    help = "Remote computer address/name."
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
    help = "Optional: Remote Windows domain"
    )]
    pub domain: Option<String>,

    #[clap(
    short = "p",
    long = "password",
    help = "Optional: Remote user password. Skipping this option will prompt a possibility to put a password in hidden way.\
     To specify an empty password use `-p \"\"`"
    )]
    pub password: Option<String>,

    #[clap(
    short = "o",
    long = "output",
    default_value = "evidence-output",
    help = "Name of local directory to store the evidence"
    )]
    pub local_store_directory: String,

    #[clap(
    short = "r",
    long = "remote-storage",
    default_value = "evidence-output",
    help = "Name of remote directory to be used as a temporary storage. (Windows targets only)",
    default_value = "C:\\Users\\Public"
    )]
    pub remote_store_directory: String,

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
    File names supports also `*` and `?` wildcards on file names (but not yet parent directories)."
    )]
    pub search_files_path: Option<String>,

    #[clap(
    long = "no-evidence-search",
    help = "Disables acquisition of evidence that can be usually downloaded quickly (like ipconfig, firewall status etc..)"
    )]
    pub disable_evidence_download: bool,

    #[clap(
    long = "no-registry-search",
    help = "Disables target registry acquisition."
    )]
    pub disable_registry_download: bool,

    #[clap(
    long = "no-events-search",
    help = "Disables Windows event logs acquisition."
    )]
    pub disable_event_download: bool,

    #[clap(
    short = "a",
    long = "all",
    help = "Acquire evidence from Windows machine using all supported methods (PsExec, PsRemote, WMI, RDP)."
    )]
    pub all: bool,

    #[clap(
    long = "wmi",
    help = "Acquire evidence from Windows machine using WMI. \
    Requires WMImplant.ps1 in the current directory or in the path and PowerShell 3.0+ on the host machine.\
    Note: It is necessary to disable Windows Defender real-time protection (other AVs not tested)."
    )]
    pub wmi: bool,

    #[clap(
    long = "rdp",
    help = "Acquire evidence from Windows machine using RDP. Requires SharpRDP.exe in the current directory or in the path."
    )]
    pub rdp: bool,

    #[clap(
    long = "psexec",
    help = "Acquire evidence from Windows machine using PsExec. Requires both PsExec64.exe and paexec.exe in the current directory or in the path."
    )]
    pub psexec: bool,

    #[clap(
    long = "psrem",
    help = "Acquire evidence from Windows machine using PowerShell. Requires both PsExec64.exe and paexec.exe in the current directory or in the path."
    )]
    pub psrem: bool,

    #[clap(
    long = "ssh",
    help = "Acquire evidence from Linux machine using SSH. Requires both plink.exe and pscp.exe in the current directory or in the path."
    )]
    pub ssh: bool,

    #[clap(long = "local",
    help = "Acquire evidence from local machine.")]
    pub local: bool,

    #[clap(
    short = "m",
    long = "mem-image",
    help = "Optional: Memory dump of a target Windows machine."
    )]
    pub image_memory: bool,

    #[clap(
    long = "timeout",
    help = "Optional: Timeout in seconds for long running operations.\
    This option is a workaround for a bug in WMImplant.ps1 amd SharpRDP.exe where finishing of a long running operation cannot sometimes properly close the connection leaving the Gargamel in seemingly frozen state or executing the next operation with the previous one unfinished on target site.\
    Increasing this timeout may solve issues when acquiring registry or memory image from target machine.",
    default_value = "300",
    )]
    pub timeout: u64,

    #[clap(long = "key", help = "Optional: Name/path of SSH private key file. (Linux target only)")]
    pub ssh_key: Option<String>,

    #[clap(long = "nla", help = "Optional: Use network level authentication when using RDP. (Windows targets only)")]
    pub nla: bool,

    #[clap(long = "no-7z", help = "Optional: Disable 7zip compression for registry & memory images.\
    This will significantly decrease the running time, but WMI and RDP connections will probably not work properly.
    (Windows targets only)")]
    pub no_compression: bool,

    #[clap(long = "redownload", help =
    "Optional: Download and DELETE specified file from target computer. \
    Use this in case of previous failed partially completed operation. \
    For just downloading a file (without deleting it) please use a `search` switch.
    If you specify a 7zip chunk (.7z.[chunk-number], e.g. .7z.004), then it will also automatically try to download \
    subsequent chunks.\
    Use also with --psexec --psrem, --rdp, --wmi, --all")]
    pub re_download: Option<String>
}

