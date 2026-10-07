use std::path::PathBuf;

pub fn pick_document(owner: usize) -> anyhow::Result<Option<PathBuf>> {
    #[cfg(windows)]
    {
        // Each native dialog gets its own STA; pooled worker threads may already use MTA.
        std::thread::spawn(move || pick_on_sta(owner))
            .join()
            .map_err(|_| anyhow::anyhow!("File selection stopped unexpectedly"))?
    }
    #[cfg(not(windows))]
    {
        let _ = owner;
        anyhow::bail!("Native document selection is available in the Windows app")
    }
}

#[cfg(windows)]
fn pick_on_sta(owner: usize) -> anyhow::Result<Option<PathBuf>> {
    use windows::{
        core::{w, HRESULT},
        Win32::{
            Foundation::{ERROR_CANCELLED, HWND},
            System::Com::{
                CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize,
                CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
            },
            UI::Shell::{
                Common::COMDLG_FILTERSPEC, FileOpenDialog, IFileOpenDialog, FOS_FILEMUSTEXIST,
                FOS_FORCEFILESYSTEM, FOS_PATHMUSTEXIST, SIGDN_FILESYSPATH,
            },
        },
    };
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE).ok()?;
    }
    struct Apartment;
    impl Drop for Apartment {
        fn drop(&mut self) {
            unsafe {
                CoUninitialize();
            }
        }
    }
    let _apartment = Apartment;
    // COM interfaces and their allocated result remain entirely on this STA.
    unsafe {
        let dialog: IFileOpenDialog =
            CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)?;
        dialog.SetOptions(
            dialog.GetOptions()? | FOS_FILEMUSTEXIST | FOS_PATHMUSTEXIST | FOS_FORCEFILESYSTEM,
        )?;
        dialog.SetFileTypes(&[COMDLG_FILTERSPEC {
            pszName: w!("Text, Markdown, JSON and Word DOCX"),
            pszSpec: w!("*.txt;*.md;*.json;*.docx"),
        }])?;
        dialog.SetTitle(w!("Import a local document"))?;
        if let Err(error) = dialog.Show(Some(HWND(owner as *mut core::ffi::c_void))) {
            if error.code() == HRESULT::from_win32(ERROR_CANCELLED.0) {
                return Ok(None);
            }
            return Err(error.into());
        }
        let path = dialog.GetResult()?.GetDisplayName(SIGDN_FILESYSPATH)?;
        let decoded = path.to_string();
        CoTaskMemFree(Some(path.0 as *const core::ffi::c_void));
        Ok(Some(PathBuf::from(decoded?)))
    }
}
