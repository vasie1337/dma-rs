use memprocfs::{Vmm, VmmProcess};

pub mod error;

pub struct Dma<'a> {
    vmm: Vmm<'a>,
}

impl<'a> Dma<'a> {
    pub fn new(vmm_dll_path: &str, device: &str) -> Result<Self, error::DmaError> {
        if !std::path::Path::new(vmm_dll_path).exists() {
            return Err(error::DmaError::InitFailed(format!("DLL not found at: {}", vmm_dll_path)));
        }

        let args = vec!["-device", device, "-waitinitialize"];

        let vmm = match Vmm::new(vmm_dll_path, &args) {
            Ok(vmm) => vmm,
            Err(e) => return Err(error::DmaError::InitFailed(e.to_string())),
        };

        Ok(Dma { vmm })
    }

    pub fn list_processes(&self) -> Result<Vec<ProcessInfo>, error::DmaError> {
        let process_list = match self.vmm.process_list() {
            Ok(list) => list,
            Err(e) => return Err(error::DmaError::InitFailed(e.to_string())),
        };

        let mut processes = Vec::new();
        for process in &*process_list {
            let info = match process.info() {
                Ok(i) => i,
                Err(_) => continue,
            };

            let path = process.get_path_kernel().unwrap_or_default();

            processes.push(ProcessInfo {
                pid: process.pid,
                name: info.name,
                ppid: info.ppid,
                path,
            });
        }

        Ok(processes)
    }

    pub fn get_process_by_name(&self, name: &str) -> Result<ProcessInfo, error::DmaError> {
        let process = match self.vmm.process_from_name(name) {
            Ok(p) => p,
            Err(e) => return Err(error::DmaError::InitFailed(format!("Process not found: {}", e))),
        };

        let info = match process.info() {
            Ok(i) => i,
            Err(e) => return Err(error::DmaError::InitFailed(e.to_string())),
        };

        let path = process.get_path_kernel().unwrap_or_default();

        Ok(ProcessInfo {
            pid: process.pid,
            name: info.name,
            ppid: info.ppid,
            path,
        })
    }

    pub fn get_process_by_pid(&self, pid: u32) -> Result<ProcessInfo, error::DmaError> {
        let process = match self.vmm.process_from_pid(pid) {
            Ok(p) => p,
            Err(e) => return Err(error::DmaError::InitFailed(format!("Process not found: {}", e))),
        };

        let info = match process.info() {
            Ok(i) => i,
            Err(e) => return Err(error::DmaError::InitFailed(e.to_string())),
        };

        let path = process.get_path_kernel().unwrap_or_default();

        Ok(ProcessInfo {
            pid: process.pid,
            name: info.name,
            ppid: info.ppid,
            path,
        })
    }

    pub fn attach_process(&self, name: &str) -> Result<VmmProcess, error::DmaError> {
        match self.vmm.process_from_name(name) {
            Ok(p) => Ok(p),
            Err(e) => Err(error::DmaError::InitFailed(format!("Process not found: {}", e))),
        }
    }

    pub fn vmm(&self) -> &Vmm<'a> {
        &self.vmm
    }
}

#[derive(Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub ppid: u32,
    pub path: String,
}
