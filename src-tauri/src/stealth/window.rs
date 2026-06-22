/// 设置窗口对屏幕捕获不可见
///
/// 使用 Windows API SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)
/// 效果：窗口在屏幕共享、截图、录屏时不可见，但用户自己在屏幕上能看到
#[cfg(target_os = "windows")]
pub fn set_invisible_to_capture(hwnd: isize) -> Result<(), String> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE,
    };

    unsafe {
        let hwnd = HWND(hwnd);
        SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE)
            .map_err(|e| format!("设置窗口不可见失败: {}", e))?;
    }

    tracing::info!("窗口已设置为对屏幕捕获不可见");
    Ok(())
}

/// 设置窗口置顶
#[cfg(target_os = "windows")]
pub fn set_always_on_top(hwnd: isize, on_top: bool) -> Result<(), String> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, HWND_NOTOPMOST, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE,
    };

    unsafe {
        let hwnd = HWND(hwnd);
        let insert_after = if on_top { HWND_TOPMOST } else { HWND_NOTOPMOST };
        SetWindowPos(hwnd, insert_after, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE)
            .map_err(|e| format!("设置窗口置顶失败: {}", e))?;
    }

    tracing::info!("窗口置顶: {}", on_top);
    Ok(())
}

/// 设置窗口透明度 (0.0 - 1.0)
#[cfg(target_os = "windows")]
pub fn set_opacity(hwnd: isize, opacity: f64) -> Result<(), String> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetWindowLongW, SetLayeredWindowAttributes, SetWindowLongW, GWL_EXSTYLE,
        LWA_ALPHA, WS_EX_LAYERED,
    };

    unsafe {
        let hwnd = HWND(hwnd);
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        SetWindowLongW(hwnd, GWL_EXSTYLE, ex_style | WS_EX_LAYERED.0 as i32);

        let alpha = (opacity * 255.0) as u8;
        SetLayeredWindowAttributes(
            hwnd,
            windows::Win32::Foundation::COLORREF(0),
            alpha,
            LWA_ALPHA,
        )
        .map_err(|e| format!("设置透明度失败: {}", e))?;
    }

    tracing::info!("窗口透明度: {:.0}%", opacity * 100.0);
    Ok(())
}

// 非 Windows 平台的空实现
#[cfg(not(target_os = "windows"))]
pub fn set_invisible_to_capture(_hwnd: isize) -> Result<(), String> {
    tracing::warn!("当前平台不支持屏幕捕获隐藏");
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn set_always_on_top(_hwnd: isize, _on_top: bool) -> Result<(), String> {
    tracing::warn!("当前平台不支持窗口置顶");
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn set_opacity(_hwnd: isize, _opacity: f64) -> Result<(), String> {
    tracing::warn!("当前平台不支持透明度设置");
    Ok(())
}
