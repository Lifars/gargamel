pub fn embedded_search_list() -> Vec<String> {
    vec![
        "C:\\Windows\\System32\\winevt\\Logs\\*.evtx".to_string(),
        "C:\\Windows.old\\Windows\\System32\\winevt\\Logs\\*.evtx".to_string(),
        "C:\\Users\\*\\AppData\\Roaming\\Microsoft\\Windows\\Recent\\*.lnk".to_string(),
        "C:\\Users\\*\\AppData\\Roaming\\Microsoft\\Office\\Recent\\*.lnk".to_string(),
        "C:\\Users\\*\\Desktop\\*.lnk".to_string(),
        "C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs\\*.lnk".to_string(),
        "C:\\Users\\*\\AppData\\Roaming\\Microsoft\\Word\\".to_string(),
        "C:\\Users\\*\\AppData\\Roaming\\Microsoft\\Excel\\".to_string(),
        "C:\\Users\\*\\AppData\\Roaming\\Microsoft\\Publisher\\".to_string(),
        "C:\\Users\\*\\NTUSER.DAT".to_string(),
        "C:\\Users\\*\\NTUSER.DAT.LOG*".to_string(),
        "C:\\Windows\\System32\\config\\DEFAULT".to_string(),
        "C:\\Windows.old\\Windows\\System32\\config\\DEFAULT".to_string(),
        "C:\\Windows\\System32\\config\\DEFAULT.LOG*".to_string(),
        "C:\\Windows.old\\Windows\\System32\\config\\DEFAULT.LOG*".to_string(),
        "C:\\Users\\*\\AppData\\Local\\Microsoft\\Windows\\UsrClass.dat".to_string(),
        "C:\\Users\\*\\AppData\\Local\\Microsoft\\Windows\\UsrClass.dat.LOG*".to_string(),
        "C:\\ProgramData\\Microsoft\\Search\\Data\\Applications\\Windows\\Windows.edb".to_string(),
    ]
}