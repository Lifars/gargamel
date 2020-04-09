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

Run
---
Right now, this app works only on Windows.

Compiled executable is located at `target/release/gargamel.exe`.

Make sure to have the following programs in *Path* or in the same directory as Gargamel.
* `wmic`, should be by default on Windows
* `paexec`, an open source alternative to PsExec, [download](https://www.poweradmin.com/paexec/)
* `winpmem`, an open source memory image tool, [download](https://github.com/Velocidex/c-aff4/releases). Download the newest executable and rename it to *winpmem.exe*.
* `plink` and `pscp`, an open source CLI SSH/SCP clients, [download](https://www.chiark.greenend.org.uk/~sgtatham/putty/latest.html)
* `SharpRDP`, an open source command executor using RDP, [download](https://github.com/vildibald/SharpRDP/releases/tag/v1.0.0)
* `WMImplant`, a Powershell script for WMI code execution, [download](https://github.com/vildibald/WMImplant)

Note: WMImplant.ps1 cannot be run with Windows Defender as it detects the script 
as malicious.
Only workaround is to disable the real-time protection of Defender. 
I don't know if other AVs also detect the file.  
It also requires PowerShell 3.0+. 

For example, to run the Gargamel against the computer with params:
* address: *192.168.126.142*
* username: *IEUser*
* password: *nbusr123*

use the command below that will store the results in the newly created directory `testresult`.
```bash
gargamel.exe --c 192.168.126.142 -u IEUser -p nbusr123 -o testresult --all
```

Known issues
------------
* WMI cannot write its output to file with symbol `-` in its path/name.

Licensing and Copyright
-----------------------
Copyright (C) 2020 LIFARS LLC

All Rights Reserved