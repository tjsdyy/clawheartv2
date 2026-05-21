//! 进程枚举抽象 — macOS/Linux 用 `ps`；Windows 用 `tasklist`。
//!
//! W17 实现真实进程扫描；当前留接口签名。

pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub exe_path: Option<String>,
}

pub fn list_processes() -> std::io::Result<Vec<ProcessInfo>> {
    // alpha: 返回空。W17 用 sysinfo crate（评估 R2 时一并定）
    Ok(Vec::new())
}
