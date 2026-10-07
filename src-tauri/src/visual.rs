use base64::{engine::general_purpose::STANDARD, Engine};

pub const MAX_IMAGE_URL: usize = 512 * 1024;
pub const MAX_IMAGE_CONTEXT: usize = 768 * 1024;
// ponytail: conservative allowance for one image scaled to 1,600 pixels;
// replace with a model-specific estimator when the catalog exposes one.
pub const IMAGE_TOKENS: usize = 4096;

pub fn validate_image_url(value: &str) -> Result<(), String> {
    if value.len() > MAX_IMAGE_URL {
        return Err("The image is too large. Copy a smaller image and try again.".into());
    }
    let (encoded, png) = if let Some(data) = value.strip_prefix("data:image/png;base64,") {
        (data, true)
    } else if let Some(data) = value.strip_prefix("data:image/jpeg;base64,") {
        (data, false)
    } else {
        return Err("Only inline PNG and JPEG images can be sent.".into());
    };
    let bytes = STANDARD.decode(encoded).map_err(|_| "Invalid image data")?;
    let valid = if png {
        bytes.starts_with(b"\x89PNG\r\n\x1a\n") && bytes.len() >= 33
    } else {
        bytes.starts_with(&[0xff, 0xd8, 0xff]) && bytes.ends_with(&[0xff, 0xd9])
    };
    if !valid {
        return Err("Invalid image data".into());
    }
    Ok(())
}

#[cfg(windows)]
pub use native::{clipboard_image, screenshot_text};

#[cfg(not(windows))]
pub fn screenshot_text() -> Result<String, String> {
    Err("Screenshot OCR is available in the Windows app.".into())
}

#[cfg(not(windows))]
pub fn clipboard_image() -> Result<String, String> {
    Err("Clipboard images are available in the Windows app.".into())
}

#[cfg(windows)]
mod native {
    use super::*;
    use std::mem::{size_of, zeroed};
    use windows::{
        Graphics::Imaging::{BitmapAlphaMode, BitmapEncoder, BitmapPixelFormat},
        Media::Ocr::OcrEngine,
        Storage::Streams::{DataReader, InMemoryRandomAccessStream},
        Win32::{
            Foundation::POINT,
            Graphics::Gdi::*,
            System::{DataExchange::*, WinRT::*},
            UI::WindowsAndMessaging::GetCursorPos,
        },
    };

    struct Runtime;
    impl Runtime {
        fn new() -> Result<Self, String> {
            unsafe { RoInitialize(RO_INIT_MULTITHREADED) }
                .map_err(|_| "Couldn't initialize Windows image processing")?;
            Ok(Self)
        }
    }
    impl Drop for Runtime {
        fn drop(&mut self) {
            unsafe { RoUninitialize() };
        }
    }

    struct Pixels {
        width: u32,
        height: u32,
        bytes: Vec<u8>,
    }

    fn pixel_bytes(width: i32, height: i32) -> Result<usize, String> {
        if width <= 0 || height <= 0 || width > 8192 || height > 8192 {
            return Err("Image dimensions must be between 1 and 8,192 pixels.".into());
        }
        let count = width as usize * height as usize;
        if count > 16_777_216 {
            return Err("The image exceeds the 16 megapixel capture limit.".into());
        }
        Ok(count * 4)
    }

    unsafe fn read_bitmap(dc: HDC, bitmap: HBITMAP) -> Result<Pixels, String> {
        let mut object: BITMAP = zeroed();
        if GetObjectW(
            HGDIOBJ(bitmap.0),
            size_of::<BITMAP>() as i32,
            Some((&mut object as *mut BITMAP).cast()),
        ) == 0
        {
            return Err("Couldn't read the image dimensions.".into());
        }
        let len = pixel_bytes(object.bmWidth, object.bmHeight)?;
        let mut info: BITMAPINFO = zeroed();
        info.bmiHeader = BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: object.bmWidth,
            biHeight: -object.bmHeight,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        };
        let mut bytes = vec![0; len];
        if GetDIBits(
            dc,
            bitmap,
            0,
            object.bmHeight as u32,
            Some(bytes.as_mut_ptr().cast()),
            &mut info,
            DIB_RGB_COLORS,
        ) != object.bmHeight
        {
            return Err("Couldn't read the image pixels.".into());
        }
        Ok(Pixels {
            width: object.bmWidth as u32,
            height: object.bmHeight as u32,
            bytes,
        })
    }

    fn capture(clipboard: bool) -> Result<Pixels, String> {
        unsafe {
            let screen = GetDC(None);
            if screen.is_invalid() {
                return Err("Couldn't access the display.".into());
            }
            let result = if clipboard {
                (|| {
                    OpenClipboard(None).map_err(|_| "The clipboard is busy. Try again.")?;
                    let result = (|| {
                        // CF_BITMAP: Windows also synthesizes it from copied DIB images.
                        let handle = GetClipboardData(2).map_err(|_| {
                            "Copy an image to the clipboard first (for example with Win+Shift+S)."
                        })?;
                        read_bitmap(screen, HBITMAP(handle.0))
                    })();
                    let _ = CloseClipboard();
                    result
                })()
            } else {
                (|| {
                    let mut point = POINT::default();
                    GetCursorPos(&mut point).map_err(|_| "Couldn't locate the active display.")?;
                    let monitor = MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST);
                    let mut info = MONITORINFO {
                        cbSize: size_of::<MONITORINFO>() as u32,
                        ..Default::default()
                    };
                    if !GetMonitorInfoW(monitor, &mut info).as_bool() {
                        return Err("Couldn't read the display bounds.".into());
                    }
                    let width = info.rcMonitor.right - info.rcMonitor.left;
                    let height = info.rcMonitor.bottom - info.rcMonitor.top;
                    pixel_bytes(width, height)?;
                    let dc = CreateCompatibleDC(Some(screen));
                    if dc.is_invalid() {
                        return Err("Couldn't prepare screenshot capture.".into());
                    }
                    let bitmap = CreateCompatibleBitmap(screen, width, height);
                    let result = if bitmap.is_invalid() {
                        Err("Couldn't allocate a screenshot.".into())
                    } else {
                        let old = SelectObject(dc, HGDIOBJ(bitmap.0));
                        let copied = BitBlt(
                            dc,
                            0,
                            0,
                            width,
                            height,
                            Some(screen),
                            info.rcMonitor.left,
                            info.rcMonitor.top,
                            SRCCOPY | CAPTUREBLT,
                        );
                        SelectObject(dc, old);
                        let result = copied
                            .map_err(|_| "Couldn't capture the display.".into())
                            .and_then(|_| read_bitmap(screen, bitmap));
                        let _ = DeleteObject(HGDIOBJ(bitmap.0));
                        result
                    };
                    let _ = DeleteDC(dc);
                    result
                })()
            };
            ReleaseDC(None, screen);
            result
        }
    }

    pub fn screenshot_text() -> Result<String, String> {
        let pixels = capture(false)?;
        let _runtime = Runtime::new()?;
        let engine = OcrEngine::TryCreateFromUserProfileLanguages()
            .map_err(|_| "Install a Windows OCR language pack in Settings → Time & language → Language & region, then try again.")?;
        let result = (|| -> windows::core::Result<String> {
            let max = OcrEngine::MaxImageDimension()?;
            let stream = encode(&pixels, false, max)?;
            let decoder =
                windows::Graphics::Imaging::BitmapDecoder::CreateAsync(&stream)?.join()?;
            let bitmap = decoder
                .GetSoftwareBitmapConvertedAsync(BitmapPixelFormat::Bgra8, BitmapAlphaMode::Ignore)?
                .join()?;
            let result = engine.RecognizeAsync(&bitmap)?.join()?;
            let mut lines = Vec::new();
            for line in result.Lines()? {
                lines.push(line.Text()?.to_string());
            }
            Ok(lines.join("\n"))
        })()
        .map_err(|_| "Couldn't recognize text in the screenshot.")?;
        if result.trim().is_empty() {
            return Err("No text was found on the display under your pointer.".into());
        }
        if result.len() > 64 * 1024 {
            return Err("The screenshot contains too much text. Capture a smaller display.".into());
        }
        Ok(result)
    }

    fn encode(
        pixels: &Pixels,
        jpeg: bool,
        max_side: u32,
    ) -> windows::core::Result<InMemoryRandomAccessStream> {
        let stream = InMemoryRandomAccessStream::new()?;
        let id = if jpeg {
            BitmapEncoder::JpegEncoderId()?
        } else {
            BitmapEncoder::PngEncoderId()?
        };
        let encoder = BitmapEncoder::CreateAsync(id, &stream)?.join()?;
        encoder.SetPixelData(
            BitmapPixelFormat::Bgra8,
            BitmapAlphaMode::Ignore,
            pixels.width,
            pixels.height,
            96.0,
            96.0,
            &pixels.bytes,
        )?;
        let longest = pixels.width.max(pixels.height);
        if longest > max_side {
            let transform = encoder.BitmapTransform()?;
            transform.SetScaledWidth(
                (u64::from(pixels.width) * u64::from(max_side) / u64::from(longest)).max(1) as u32,
            )?;
            transform.SetScaledHeight(
                (u64::from(pixels.height) * u64::from(max_side) / u64::from(longest)).max(1) as u32,
            )?;
        }
        encoder.FlushAsync()?.join()?;
        Ok(stream)
    }

    pub fn clipboard_image() -> Result<String, String> {
        let pixels = capture(true)?;
        let _runtime = Runtime::new()?;
        let result = (|| -> windows::core::Result<Option<String>> {
            for jpeg in [false, true] {
                let stream = encode(&pixels, jpeg, 1600)?;
                let size = stream.Size()?;
                if size > ((MAX_IMAGE_URL - 32) / 4 * 3) as u64 {
                    continue;
                }
                let reader = DataReader::CreateDataReader(&stream.GetInputStreamAt(0)?)?;
                reader.LoadAsync(size as u32)?.join()?;
                let mut bytes = vec![0; size as usize];
                reader.ReadBytes(&mut bytes)?;
                let format = if jpeg { "jpeg" } else { "png" };
                return Ok(Some(format!(
                    "data:image/{format};base64,{}",
                    STANDARD.encode(bytes)
                )));
            }
            Ok(None)
        })()
        .map_err(|_| "Couldn't prepare the clipboard image.")?;
        let value = result.ok_or("The image is too large. Copy a smaller region and try again.")?;
        validate_image_url(&value)?;
        Ok(value)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub fn image() -> String {
        // A real 1×1 PNG; never use a remote URL in a message.
        "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aP1sAAAAASUVORK5CYII=".into()
    }

    #[test]
    fn image_boundary_rejects_urls_corruption_and_oversize() {
        assert!(validate_image_url(&image()).is_ok());
        for bad in [
            "https://example.org/image.png",
            "data:image/png;base64,dGV4dA==",
            "data:image/jpeg;base64,?",
        ] {
            assert!(validate_image_url(bad).is_err());
        }
        assert!(validate_image_url(&format!(
            "data:image/png;base64,{}",
            "A".repeat(MAX_IMAGE_URL)
        ))
        .is_err());
    }
}
