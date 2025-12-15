use memprocfs::{Vmm, VmmProcess, VmmScatterMemory};

pub mod error;
mod embedded;

pub use error::DmaError;

pub struct Dma<'a> {
    vmm: Vmm<'a>,
}

impl<'a> Dma<'a> {
    pub fn new(device: &str) -> Result<Self, DmaError> {
        let vmm_dll_path = embedded::get_vmm_dll_path()?;
        let args = vec!["-device", device, "-waitinitialize"];

        let vmm = Vmm::new(vmm_dll_path, &args)
            .map_err(|e| DmaError::InitFailed(e.to_string()))?;

        Ok(Dma { vmm })
    }

    pub fn list_processes(&self) -> Result<Vec<ProcessInfo>, DmaError> {
        let process_list = self.vmm.process_list()
            .map_err(|e| DmaError::ProcessError(e.to_string()))?;

        let mut processes = Vec::new();
        for proc in &*process_list {
            if let Ok(info) = proc.info() {
                let path = proc.get_path_kernel().unwrap_or_default();
                processes.push(ProcessInfo {
                    pid: proc.pid,
                    name: info.name,
                    ppid: info.ppid,
                    path,
                });
            }
        }

        Ok(processes)
    }

    pub fn attach(&'a self, name: &str) -> Result<Process<'a>, DmaError> {
        let vmm_process = self.vmm.process_from_name(name)
            .map_err(|e| DmaError::ProcessError(format!("Process '{}' not found: {}", name, e)))?;

        Ok(Process::new(vmm_process))
    }

    pub fn attach_pid(&'a self, pid: u32) -> Result<Process<'a>, DmaError> {
        let vmm_process = self.vmm.process_from_pid(pid)
            .map_err(|e| DmaError::ProcessError(format!("PID {} not found: {}", pid, e)))?;

        Ok(Process::new(vmm_process))
    }

    pub fn get_process_info(&self, name: &str) -> Result<ProcessInfo, DmaError> {
        let proc = self.vmm.process_from_name(name)
            .map_err(|e| DmaError::ProcessError(format!("Process '{}' not found: {}", name, e)))?;

        let info = proc.info()
            .map_err(|e| DmaError::ProcessError(e.to_string()))?;

        let path = proc.get_path_kernel().unwrap_or_default();

        Ok(ProcessInfo {
            pid: proc.pid,
            name: info.name,
            ppid: info.ppid,
            path,
        })
    }

    pub fn vmm(&self) -> &Vmm<'a> {
        &self.vmm
    }
}

pub struct Process<'a> {
    inner: VmmProcess<'a>,
}

impl<'a> Process<'a> {
    fn new(inner: VmmProcess<'a>) -> Self {
        Self { inner }
    }

    pub fn pid(&self) -> u32 {
        self.inner.pid
    }

    pub fn info(&self) -> Result<ProcessInfo, DmaError> {
        let info = self.inner.info()
            .map_err(|e| DmaError::ProcessError(e.to_string()))?;

        let path = self.inner.get_path_kernel().unwrap_or_default();

        Ok(ProcessInfo {
            pid: self.inner.pid,
            name: info.name,
            ppid: info.ppid,
            path,
        })
    }

    pub fn read<T: Copy>(&self, address: u64) -> Result<T, DmaError> {
        self.inner.mem_read_as(address, 0)
            .map_err(|e| DmaError::MemoryError(format!("Read failed at 0x{:X}: {}", address, e)))
    }

    pub fn read_bytes(&self, address: u64, size: usize) -> Result<Vec<u8>, DmaError> {
        self.inner.mem_read(address, size)
            .map_err(|e| DmaError::MemoryError(format!("Read {} bytes failed at 0x{:X}: {}", size, address, e)))
    }

    pub fn write<T: Copy>(&self, address: u64, value: &T) -> Result<(), DmaError> {
        self.inner.mem_write_as(address, value)
            .map_err(|e| DmaError::MemoryError(format!("Write failed at 0x{:X}: {}", address, e)))
    }

    pub fn write_bytes(&self, address: u64, data: &[u8]) -> Result<(), DmaError> {
        self.inner.mem_write(address, data)
            .map_err(|e| DmaError::MemoryError(format!("Write {} bytes failed at 0x{:X}: {}", data.len(), address, e)))
    }

    pub fn scatter(&'a self) -> Result<ScatterHandle<'a>, DmaError> {
        let scatter = self.inner.mem_scatter(0)
            .map_err(|e| DmaError::MemoryError(format!("Failed to create scatter handle: {}", e)))?;

        Ok(ScatterHandle { scatter })
    }

    pub fn module_base(&self, module_name: &str) -> Result<u64, DmaError> {
        self.inner.get_module_base(module_name)
            .map_err(|e| DmaError::ModuleError(format!("Module '{}' not found: {}", module_name, e)))
    }

    pub fn proc_address(&self, module_name: &str, function_name: &str) -> Result<u64, DmaError> {
        self.inner.get_proc_address(module_name, function_name)
            .map_err(|e| DmaError::ModuleError(format!("Function '{}' in '{}' not found: {}", function_name, module_name, e)))
    }

    pub fn list_modules(&self) -> Result<Vec<Module>, DmaError> {
        let module_map = self.inner.map_module(false, false)
            .map_err(|e| DmaError::ModuleError(format!("Failed to enumerate modules: {}", e)))?;

        let modules = module_map.iter()
            .map(|m| Module {
                name: m.name.clone(),
                base: m.va_base,
                size: m.image_size as usize,
                entry: m.va_entry,
                path: m.full_name.clone(),
            })
            .collect();

        Ok(modules)
    }

    pub fn read_string(&self, address: u64, max_length: usize) -> Result<String, DmaError> {
        let bytes = self.read_bytes(address, max_length)?;

        let null_pos = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        let string_bytes = &bytes[..null_pos];

        String::from_utf8(string_bytes.to_vec())
            .map_err(|e| DmaError::MemoryError(format!("Invalid UTF-8 string at 0x{:X}: {}", address, e)))
    }

    pub fn inner(&self) -> &VmmProcess<'a> {
        &self.inner
    }
}

pub struct ScatterHandle<'a> {
    scatter: VmmScatterMemory<'a>,
}

impl<'a> ScatterHandle<'a> {
    pub fn prepare_read(&mut self, address: u64, size: usize) {
        let _ = self.scatter.prepare(address, size);
    }

    pub fn prepare_read_ex(&mut self, data: &'a mut (u64, Vec<u8>, u32)) {
        let _ = self.scatter.prepare_ex(data);
    }

    pub fn execute(&mut self) -> Result<(), DmaError> {
        self.scatter.execute()
            .map_err(|e| DmaError::MemoryError(format!("Scatter execute failed: {}", e)))
    }

    pub fn read(&mut self, address: u64, size: usize) -> Result<Vec<u8>, DmaError> {
        self.scatter.read(address, size)
            .map_err(|e| DmaError::MemoryError(format!("Scatter read failed at 0x{:X}: {}", address, e)))
    }

    pub fn read_as<T: Copy>(&mut self, address: u64) -> Result<T, DmaError> {
        self.scatter.read_as(address)
            .map_err(|e| DmaError::MemoryError(format!("Scatter read_as failed at 0x{:X}: {}", address, e)))
    }

    pub fn clear(&mut self) {
        let _ = self.scatter.clear();
    }
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub ppid: u32,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub base: u64,
    pub size: usize,
    pub entry: u64,
    pub path: String,
}
