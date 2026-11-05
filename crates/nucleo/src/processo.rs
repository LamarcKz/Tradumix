//! Garante que o xdelta3 não continue rodando se o app fechar no meio da tradução.
//! O Windows não encerra os processos filhos junto com o pai; um Job Object com
//! "matar ao fechar" faz isso quando o último handle dele (o do app) é fechado.

use std::os::windows::io::AsRawHandle;
use std::process::Child;

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JobObjectExtendedLimitInformation, SetInformationJobObject,
};
use windows::core::PCWSTR;

use crate::Erro;

pub struct GrupoDeProcessos(HANDLE);

// O handle do Job Object pode ser usado de qualquer thread.
unsafe impl Send for GrupoDeProcessos {}

impl GrupoDeProcessos {
    pub fn novo() -> Result<GrupoDeProcessos, Erro> {
        let erro = |e: windows::core::Error| Erro::Io(format!("não foi possível preparar o processo do xdelta3: {e}"));
        let job = unsafe { CreateJobObjectW(None, PCWSTR::null()) }.map_err(erro)?;
        let grupo = GrupoDeProcessos(job);
        let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        unsafe {
            SetInformationJobObject(
                grupo.0,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const std::ffi::c_void,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        }
        .map_err(erro)?;
        Ok(grupo)
    }

    pub fn incluir(&self, filho: &Child) -> Result<(), Erro> {
        let processo = HANDLE(filho.as_raw_handle());
        unsafe { AssignProcessToJobObject(self.0, processo) }
            .map_err(|e| Erro::Io(format!("não foi possível vincular o xdelta3 ao app: {e}")))
    }
}

impl Drop for GrupoDeProcessos {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.0) };
    }
}
