use anyhow::{anyhow, Result};
use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::HBRUSH;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetSystemMetrics, RegisterClassW, ShowWindow, SM_CXSCREEN,
    SM_CYSCREEN, SW_SHOW, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP,
    WS_POPUP,
};

use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowPos, HWND_TOP, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW,
};

pub struct WindowManager {
    overlay_hwnd: Option<HWND>,
    target_hwnd: Option<HWND>,
}

use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, LoadCursorW, SetCursor, GWLP_USERDATA, HTCLIENT, HTTRANSPARENT, IDC_ARROW,
    MA_NOACTIVATE, WM_MOUSEACTIVATE, WM_NCHITTEST, WM_SETCURSOR,
};

#[repr(C)]
pub struct SharedWindowState {
    pub toolbar_rect: windows::Win32::Foundation::RECT,
    pub is_visible: bool,
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_NCHITTEST {
        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SharedWindowState;
        if !ptr.is_null() && (*ptr).is_visible {
            let state = &*ptr;
            // param is screen coordinates
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

            let mut pt = windows::Win32::Foundation::POINT { x, y };
            windows::Win32::Graphics::Gdi::ScreenToClient(hwnd, &mut pt);

            if pt.x >= state.toolbar_rect.left
                && pt.x <= state.toolbar_rect.right
                && pt.y >= state.toolbar_rect.top
                && pt.y <= state.toolbar_rect.bottom
            {
                return LRESULT(HTCLIENT as isize);
            }
        }
        return LRESULT(HTTRANSPARENT as isize);
    }

    if msg == WM_MOUSEACTIVATE {
        return LRESULT(MA_NOACTIVATE as isize);
    }

    // リサイズカーソルが表示されないように、常に矢印カーソルを設定
    if msg == WM_SETCURSOR {
        if let Ok(arrow) = LoadCursorW(None, IDC_ARROW) {
            SetCursor(arrow);
            return LRESULT(1); // カーソル設定済みを示す
        }
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            overlay_hwnd: None,
            target_hwnd: None,
        }
    }

    /// Magpie方式: ソースウィンドウを変更しない（ボーダーレス化しない）
    pub fn prepare_target_window(&mut self, hwnd: HWND) -> Result<()> {
        self.target_hwnd = Some(hwnd);
        // Magpie方式ではソースウィンドウのスタイルは変更しない
        // スケーリングウィンドウでカバーするだけ
        Ok(())
    }

    /// Magpie方式のオーバーレイウィンドウ作成
    /// - ソースウィンドウを親として設定（Z-order連動）
    /// - WS_EX_TOPMOSTは使わない
    /// - WS_EX_NOACTIVATEでアクティベーション防止
    pub fn create_overlay_window(&mut self) -> Result<()> {
        let target = self
            .target_hwnd
            .ok_or_else(|| anyhow!("Target window not set"))?;

        unsafe {
            let instance = windows::Win32::System::LibraryLoader::GetModuleHandleW(None)?;
            let class_name = w!("MagpieScalingOverlay");

            let wnd_class = WNDCLASSW {
                lpfnWndProc: Some(window_proc),
                hInstance: instance.into(),
                lpszClassName: class_name,
                hbrBackground: HBRUSH(0), // No background
                ..Default::default()
            };

            RegisterClassW(&wnd_class);

            let screen_width = GetSystemMetrics(SM_CXSCREEN);
            let screen_height = GetSystemMetrics(SM_CYSCREEN);

            // Magpie方式:
            // - WS_EX_LAYERED: レイヤードウィンドウ
            // - WS_EX_NOACTIVATE: アクティベーション防止
            // - WS_EX_NOREDIRECTIONBITMAP: DWMリダイレクトなし
            // - 親ウィンドウとしてソースを設定（Z-order連動）
            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_NOACTIVATE | WS_EX_NOREDIRECTIONBITMAP,
                class_name,
                w!("Magpie Overlay"),
                WS_POPUP, // WS_VISIBLE removed to prevent flash
                0,
                0,
                screen_width,
                screen_height,
                target, // ソースウィンドウを親として設定
                None,
                instance,
                None,
            );

            if hwnd.0 == 0 {
                return Err(anyhow!("Failed to create overlay window"));
            }

            // ShowWindow(hwnd, SW_SHOW); // Removed, call show_overlay manually later
            self.overlay_hwnd = Some(hwnd);

            Ok(())
        }
    }

    pub fn show_overlay(&self) {
        if let Some(hwnd) = self.overlay_hwnd {
            unsafe {
                let _ = SetWindowPos(
                    hwnd,
                    HWND_TOP,
                    0,
                    0,
                    0,
                    0,
                    SWP_SHOWWINDOW | SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
                );
            }
        }
    }

    pub fn get_overlay_window(&self) -> Option<HWND> {
        self.overlay_hwnd
    }

    /// Magpie方式: ターゲットウィンドウを変更していないので復元不要
    pub fn restore_target_window(&self) {
        // ターゲットウィンドウのスタイルは変更していないので何もしない
    }
}
