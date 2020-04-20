Gargamel
========

Compile
-------

Assuming you have Rust 1.41+ installed.
Open terminal in the project directory and to compile a release build type

```bash
cargo build --release
```

Debug build can be compiled using

```bash
cargo build
```

Make sure to have the following programs in the same directory as Gargamel.
* `psexec`, [download](https://docs.microsoft.com/en-us/sysinternals/downloads/psexec)
* `paexec`, an open source alternative to PsExec, [download](https://www.poweradmin.com/paexec/)
* `winpmem`, an open source memory image tool, [download](https://github.com/Velocidex/c-aff4/releases).
     * Download the newest executable and rename it to *winpmem.exe*
* `plink` and `pscp`, an open source CLI SSH/SCP clients, [download](https://www.chiark.greenend.org.uk/~sgtatham/putty/latest.html)
* `SharpRDP`, an open source command executor using RDP, [download](https://github.com/vildibald/SharpRDP/releases/tag/v1.0.0)
* `WMImplant`, as open source PowerShell WMI command executor, [download](https://github.com/vildibald/WMImplant)   

### Set log level

If you wish to change the logging level:
* Open `src/main.rs`
* On lines 42 and 43 change `LevelFilter::Info` to (for example) `LevelFilter::Trace` for more detailed logging.
    * Beware that the `LevelFilter::Trace` will log everything including passwords.  

Run
---

Right now, this app works only on Windows and the target computer must use also Windows.

Compiled executable is located at `target/release/gargamel.exe`.

Help:
```bash
USAGE:
    gargamel.exe [FLAGS] [OPTIONS] --user <user>

FLAGS:
    -a, --all                   Acquire evidence from Windows machine using all supported methods (PsExec, PsRemote,
                                WMI, RDP).
        --no-events-search      Disables Windows event logs acquisition.
        --no-evidence-search    Disables acquisition of evidence that can be usually downloaded quickly (like ipconfig,
                                firewall status etc..)
        --no-registry-search    Disables target registry acquisition.
    -h, --help                  Prints help information
    -m, --mem-image             Optional: Memory dump of a target Windows machine.
        --local                 Acquire evidence from local machine.
        --nla                   Optional: Use network level authentication when using RDP. (Windows targets only)
        --no-7z                 Optional: Disable 7zip compression for registry & memory images.This will significantly
                                decrease the running time, but WMI and RDP connections will probably not work properly.
                                    (Windows targets only)
        --psexec                Acquire evidence from Windows machine using PsExec. Requires both PsExec64.exe and
                                paexec.exe in the current directory or in the path.
        --psrem                 Acquire evidence from Windows machine using PowerShell. Requires both PsExec64.exe and
                                paexec.exe in the current directory or in the path.
        --rdp                   Acquire evidence from Windows machine using RDP. Requires SharpRDP.exe in the current
                                directory or in the path.
        --ssh                   Acquire evidence from Linux machine using SSH. Requires both plink.exe and pscp.exe in
                                the current directory or in the path.
    -V, --version               Prints version information
        --wmi                   Acquire evidence from Windows machine using WMI. Requires WMImplant.ps1 in the current
                                directory or in the path and PowerShell 3.0+ on the host machine.Note: It is necessary
                                to disable Windows Defender real-time protection (other AVs not tested).

OPTIONS:
    -c, --computer <computer>                        Remote computer address/name. [default: 127.0.0.1]
    -e, --commands <custom-command-path>             Optional: File with custom commands to execute on remote computer
    -d, --domain <domain>                            Optional: Remote Windows domain
    -o, --output <local-store-directory>
            Name of local directory to store the evidence [default: evidence-output]

    -p, --password <password>
            Optional: Remote user password. Skipping this option will prompt a possibility to put a password in hidden
            way.To specify an empty password use `-p ""`
        --redownload <re-download>
            Optional: Download and DELETE specified file from target computer. Use this in case of previous failed
            partially completed operation. For just downloading a file (without deleting it) please use a `search`
            switch.
                If you specify a 7zip chunk (.7z.[chunk-number], e.g. .7z.004), then it will also automatically try to
            download subsequent chunks.Use also with --psexec --psrem, --rdp, --wmi, --all
    -r, --remote-storage <remote-store-directory>
            Name of remote directory to be used as a temporary storage. (Windows targets only) [default:
            C:\Users\Public]
    -s, --search <search-files-path>
            Optional: File with files names to be searched on remote computer. File names supports also `*` and `?`
            wildcards on file names (but not yet parent directories).
        --key <ssh-key>                              Optional: Name/path of SSH private key file. (Linux target only)
        --timeout <timeout>
            Optional: Timeout in seconds for long running operations.This option is a workaround for a bug in
            WMImplant.ps1 amd SharpRDP.exe where finishing of a long running operation cannot sometimes properly close
            the connection leaving the Gargamel in seemingly frozen state or executing the next operation with the
            previous one unfinished on target site.Increasing this timeout may solve issues when acquiring registry or
            memory image from target machine. [default: 300]
    -u, --user <user>                                Remote user name

```

Known issues
------------
* WMI cannot write its output to file with symbol `_` in its path/name.

Licensing and Copyright
-----------------------
Copyright (C) 2020 LIFARS LLC

All Rights Reserved