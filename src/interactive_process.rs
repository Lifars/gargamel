// use std::io::prelude::*;
// use std::process::{Command, Stdio};
// use std::sync::mpsc;
// use std::thread;
// use std::io::BufReader;
// use std::ffi::OsStr;
// use std::ops::Not;
//
// pub struct InteractiveProcess {
//     process: std::process::Child,
//     tx: mpsc::Sender<Option<String>>,
//     rx: mpsc::Receiver<Option<String>>,
// }
//
// impl InteractiveProcess {
//     pub fn new<
//         T: AsRef<str> + AsRef<OsStr>
//     >(command_name: &str,
//       command_args: Option<&[T]>,
//     ) -> InteractiveProcess {
//         let mut command = Command::new(command_name);
//         match command_args {
//             None => {}
//             Some(command_args) => {
//                 if !command_args.is_empty() {
//                     command.args(command_args);
//                 }
//             }
//         }
//         let process = command.stdin(Stdio::piped())
//             .stdout(Stdio::piped())
//             .spawn().unwrap();
//
//         let (tx, rx) = mpsc::channel();
//         InteractiveProcess {
//             process: process,
//             tx: tx,
//             rx: rx,
//         }
//     }
//
//     pub fn run(&mut self) {
//         let tx = self.tx.clone();
//         let stdout = self.process.stdout
//             .take().expect("Cannot take stdout");
//
//         thread::spawn(move || {
//             let reader = BufReader::new(stdout);
//
//             for line in reader.lines() {
//                 tx.send(Some(line.unwrap()));
//             }
//         });
//     }
//
//     pub fn push(&mut self, buf: &[u8]) {
//         let mut stdin = self.process.stdin
//             .as_mut().unwrap();
//
//         stdin.write_all(buf);
//     }
//
//     pub fn responses(&mut self) -> InteractiveProcessIterator {
//         InteractiveProcessIterator {
//             subprocess: self,
//         }
//     }
// }
//
// pub struct InteractiveProcessIterator<'a> {
//     subprocess: &'a mut InteractiveProcess,
// }
//
// impl<'a> Iterator for InteractiveProcessIterator<'a> {
//     type Item = String;
//     fn next(&mut self) -> Option<String> {
//         let data = self.subprocess.rx.try_recv();
//         if data.is_ok() {
//             data.unwrap()
//         } else {
//             None
//         }
//     }
// }
