![alt text](logo.png "Gargamel")

Gargamel
========

Compile
-------

Compiled and tested with Rust 1.50+. Open terminal in the project directory and to compile a release
build type

```bash
cargo build --release
```

Debug build can be compiled using

```bash
cargo build
```

Compiled executable is located at `target/release/gargamel.exe` or `target/debug/gargamel.exe`,
respectively.

### Set log level

If you wish to change the logging level:

* Open `src/main.rs`
* On lines 42 and 43 change `LevelFilter::Info` to (for example) `LevelFilter::Trace` for more
  detailed logging.
    * Beware that the `LevelFilter::Trace` will log everything including passwords.

User guide
----------

Right now, this app works only on Windows and the target computer must use Windows or Linux.

Make sure to have the following programs in the same directory as Gargamel.

* `psexec`, [download](https://docs.microsoft.com/en-us/sysinternals/downloads/psexec)
* `paexec`, an open source alternative to PsExec, [download](https://www.poweradmin.com/paexec/)
* `winpmem`, an open source memory image
  tool, [download](https://github.com/Velocidex/c-aff4/releases).
    * Download the newest executable and rename it to *winpmem.exe*
* `plink` and `pscp`, an open source CLI SSH/SCP
  clients, [download](https://www.chiark.greenend.org.uk/~sgtatham/putty/latest.html)
* `SharpRDP`, an open source command executor using
  RDP, [download](https://github.com/vildibald/SharpRDP/releases/tag/v1.0.0)
* `WMImplant`, as open source PowerShell WMI command
  executor, [download](https://github.com/vildibald/WMImplant)
* `7za.exe`, a standalone console version of 7zip
  archiver, [download](https://www.7-zip.org/download.html)

Note: We need both the `psexec` and `paexec`. Although both applications are supposed to be
functionally equivalent they actually both have different behavior under some circumstances.

### Unleashing the power of Gargamel

Gargamel needs to be launched from an elevated terminal to be fully functional. Currently it does
not support the UAC dialog nor any kind of notification when running with limited privileges. When
running with limited user privileges, then some operations like target memory dumping will not work.

#### Basic example

Assume you want to connect to a computer with the following parameters:

* address `192.168.42.47`
* username `Jano`
* password `nbusr123`

The following command will acquire firewall state, network state, logged users, running processes,
active network connections, registry, system & application event logs using PsExec method. Evidence
will be stored in the `testResults` directory relative to the location of Gargamel.

```bash
gargamel.exe -c 192.168.42.47 -u Jano --psexec -o testResults
```

Gargamel will ask you for password of the remote user, in our example the password is `nbusr123`.
Note that password will be hidden when typing.

It is also possible to specify the password directly as program argument.

```bash
gargamel.exe -c 192.168.42.47 -u Jano --psexec -p nbusr123 -o testResults
```

#### Domain example

Assume you want to connect to a computer in a domain with the following parameters:

* domain `WORKSPACE`
* computer name `JanovPC`
* username `Jano`
* password `nbusr123`

The following command will acquire firewall state, network state, logged users, running processes,
active network connections, registry, system & application event logs using PsExec method.

```bash
gargamel.exe -c JanovPC -u WORKSPACE\Jano --psexec -o testResults
```

Or to skip password prompting specify the password directly.

```bash
gargamel.exe -c JanovPC -u WORKSPACE\Jano --psexec -p nbusr123 -o testResults
```

#### Other connection methods

PsExec is one of the 5 supported connection methods. You can replace the `--psexec` with the
following options:

* `--psexec`
* `--psrem`, if PowerShell remoting is configured on the target machine.
* `--rdp`, if RDP is enabled on the target machine.
* `--wmi`.
* `--ssh`, if the target machine uses Linux.

It is possible to use several methods at once. For example to use both PsExec and RDP one can use
the following command.

```bash
gargamel.exe -c 192.168.42.47 -u Jano --psexec --rdp -o testResults
```

There is also a special switch `--all` that is equal to specifying `--psexec --rdp --psrem --wmi`.

Note: Launch parameters are order-agnostic, i.e. it does not matter in which order the parameters
are specified.

#### Acquire memory

To acquire also memory dump, then simply add the `-m` flag to the program parameters, i.e.

```bash
gargamel.exe -c 192.168.42.47 -u Jano --psexec -o testResults -m
```

If you wish to acquire ONLY the memory dump without other evidence then use the following command.

```bash
gargamel.exe -c 192.168.42.47 -u Jano --psexec -o testResults -m --no-events-search --no-evidence-search --no-registry-search                                                          
```

This functionality is available only for Windows targets.

#### Run custom commands

Gargamel may run custom Windows CMD or Linux shell commands on remote machine.

First create a file `custom-commands.txt` with the following content.

```bash
# Will be run using any method
ipconfig
# Will run only when launching with at least one of --all, --psexec, --wmi methods
:psexec:wmi ipconfig -all
```  

Results of the above commands will be stored in the directory specified by `-o` option.

To run the above commands written in `custom-commands.txt` use the `-e` switch, i.e.

```bash
gargamel.exe -c 192.168.42.47 -u Jano --psexec -o testResults -e custom-commands.txt                                                           
```

#### Download custom files

Gargamel is able to download remote files.

First create a file `custom-files.txt` with the following content.

```bash
C:\Users\Public\sss*
C:\Users\Jano\danove.pdf 
# This line and the next one will be ignored
# C:\Users\Jano\somBajecny.pptx  
```  

###### Note: Wildcards * and ? are supported but currently only in filenames, not parent directories, i.e. C:\Users\J*\danove.pdf will most likely not work.

Results of the above commands will be stored in the directory specified by `-o` option.

To run the above commands written in `custom-files.txt` use the `-s` switch, i.e.

```bash
gargamel.exe -c 192.168.42.47 -u Jano --psexec -o testResults -s custom-files.txt                                                           
```

#### All options

All supported switches are described below.

```bash
gargamel 1.1

LIFARS LLC

USAGE:
    gargamel.exe [FLAGS] [OPTIONS]

FLAGS:
    -a, --all                     Acquire evidence from Windows machine using all supported methods
                                  (PsExec, PsRemote, WMI, RDP)
        --disable-log-colors      Optional: Disable colored log messages
    -h, --help                    Prints help information
        --in-parallel             Optional: Experimental. Enable parallelism when connecting to more
                                  remote computers
        --local                   Acquire evidence from a local Windows machine
    -m, --mem-image               Optional: Memory dump of a target Windows machine
        --nla                     Optional: Use network level authentication when using RDP.
                                  (Windows targets only)
        --no-7z                   Optional: Disable 7zip compression for registry & memory images.
                                  This will significantly decrease the running time, but WMI and RDP
                                  connections will probably not work properly. (Windows targets
                                  only)
        --no-events-search        Disables Windows event logs acquisition
        --no-evidence-search      Disables acquisition of evidence that can be usually downloaded
                                  quickly (like ipconfig, firewall status etc.)
        --no-predefined-search    Disables predefined acquisition. It is equal to --no-evidence-
                                  search --no-registry-search --no-events-search
        --no-registry-search      Disables target registry acquisition
        --psexec                  Acquire evidence from a remote Windows machine using PsExec.
                                  Requires both PsExec64.exe and paexec.exe in the current directory
                                  or in the path
        --psexec32                Acquire evidence from a remote Windows machine using 32 bit
                                  PsExec. Requires both PsExec.exe and paexec.exe in the current
                                  directory or in the path
        --psrem                   "Acquire evidence a remote from Windows machine using PowerShell."
        --rdp                     Acquire evidence from a remote Windows machine using RDP. Requires
                                  SharpRDP.exe in the current directory or in the path
        --reverse-share           Optional: Dangerous!. Reverse copy operations for --psexec and
                                  --psrem. By default, Gargamel creates temporary (or uses) shared
                                  folders to target's C drive. This option creates a temporary
                                  shared folder of the host's C drive and exposes it to the target
        --shadow                  Optional: Acquire system files from shadow volume
        --ssh                     Acquire evidence from a remote Linux machine using SSH. Requires
                                  both plink.exe and pscp.exe in the current directory or in the
                                  path
        --svi                     Optional: Acquire .lnk files from System Volume Information
    -V, --version                 Prints version information
        --wmi                     Acquire evidence from a remote Windows machine using WMI. Requires
                                  WMImplant.ps1 in the current directory or in the path and
                                  PowerShell 3.0+ on the host machine. Note: It is necessary to
                                  disable Windows Defender real-time protection (other AVs not
                                  tested)

OPTIONS:
    -c, --computer <computer>
            "Remote computer address/name. It may be also a path to a file with list of
            addresses/names (one per line in form 'address domain\\user password')." [default:
            127.0.0.1]

    -e, --commands <custom-command-path>
            Optional: File with custom commands to execute on a remote computer

        --key <ssh-key>
            Optional: Name/path of a SSH private key file. (Linux target only)

    -o, --output <local-store-directory>
            Name of local directory to store the evidence [default: evidence-output]

    -p, --password <password>
            Optional: Remote user password. Skipping this option will prompt for a password during
            start. To specify an empty password use `-p ""`

    -r, --remote-storage <remote-store-directory>
            Name of remote directory to be used as a temporary storage (Windows targets only)
            [default: C:\]

        --redownload <re-download>
            Optional: Download and DELETE specified file from the target computer. Use this in case
            of previous failed partially completed operation. For just downloading a file (without
            deleting it) please use a `search` switch instead. If you specify a 7zip chunk
            (.7z.[chunk-number], e.g. .7z.004), then it will also automatically try to download
            subsequent chunks. Use also with --psexec --psrem, --rdp, --wmi, --all

    -s, --search <search-files-path>
            Optional: File with files names to be searched on a remote computer. File names support
            also `*` and `?` wildcards on file names (but only single * is allowed yet in parent
            directories). When placing here value 'EMBEDDED' then it will download the paths
            declared in 'src/embedded_search_list.rs'.

        --share <share>
            Optional: Use custom shared folder

        --timeout <timeout>
            Optional: Timeout in seconds for long running operations. This option is a workaround
            for a bug in WMImplant.ps1 amd SharpRDP.exe where finishing of a long running operation
            cannot sometimes properly close the connection. This leaves the Gargamel in a seemingly
            frozen state or it may execute the next operation prematurely. Increasing this timeout
            may solve issues when acquiring registry or memory image from remote machines [default:
            300]

    -u, --user <user>
            Remote user name

    --use-kape-config <kape-config-path>
            Optional: Path to kape configs to be converted and used.
```

Known issues
------------

* WMI cannot write its output to a file with symbol `_` in its path/name.

Licensing and Copyright
-----------------------
Copyright (C) 2020 LIFARS LLC

All Rights Reserved
