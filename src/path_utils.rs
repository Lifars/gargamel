// use std::path::{Path, PathBuf};
//
// #[cfg(not(target_os = "windows"))]
// pub fn adjust_canonicalization<P: AsRef<Path>>(p: P) -> String {
//     p.as_ref().display().to_string()
// }
//
// #[cfg(target_os = "windows")]
// pub fn adjust_canonicalization<P: AsRef<Path>>(p: P) -> String {
//     const VERBATIM_PREFIX: &str = r#"\\?\"#;
//     let p = p.as_ref().display().to_string();
//     if p.starts_with(VERBATIM_PREFIX) {
//         p[VERBATIM_PREFIX.len()..].to_string()
//     } else {
//         p
//     }
// }
