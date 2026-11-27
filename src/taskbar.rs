//! Native Windows taskbar progress indicator, driven by live CPU utilization.
use windows::{
    core::{GUID, PCWSTR},
    Win32::{
        System::Com::{
            CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
            COINIT_APARTMENTTHREADED,
        },
        UI::{
            Shell::{ITaskbarList3, TBPF_NOPROGRESS, TBPF_NORMAL},
            WindowsAndMessaging::FindWindowW,
        },
    },
};

const CLSID_TASKBAR_LIST: GUID = GUID::from_u128(0x56fdf344_fd6d_11d0_958a_006097c9a090);

pub fn update(cpu_percent: Option<u32>) -> windows::core::Result<()> {
    // The GUI thread is the only caller. Balance COM initialization when this call initializes it;
    // RPC_E_CHANGED_MODE means the host already initialized this thread in another apartment.
    let init = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
    let should_uninitialize = init.is_ok();
    let result = (|| unsafe {
        let taskbar: ITaskbarList3 =
            CoCreateInstance(&CLSID_TASKBAR_LIST, None, CLSCTX_INPROC_SERVER)?;
        taskbar.HrInit()?;
        let title: Vec<u16> = "M4 PC Monitor\0".encode_utf16().collect();
        let hwnd = FindWindowW(PCWSTR::null(), PCWSTR(title.as_ptr()))?;
        if let Some(value) = cpu_percent {
            taskbar.SetProgressState(hwnd, TBPF_NORMAL)?;
            taskbar.SetProgressValue(hwnd, value.min(100) as u64, 100)?;
        } else {
            taskbar.SetProgressState(hwnd, TBPF_NOPROGRESS)?;
        }
        Ok(())
    })();
    if should_uninitialize {
        unsafe { CoUninitialize() };
    }
    result
}
