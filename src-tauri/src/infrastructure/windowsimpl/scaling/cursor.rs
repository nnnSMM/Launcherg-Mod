use anyhow::Result;
use std::sync::OnceLock;
use windows::core::{s, w};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, POINT, RECT};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::Graphics::Gdi::{
    ClientToScreen, CreateRectRgn, GetWindowRgn, PtInRect, PtInRegion, GDI_REGION_TYPE,
};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows::Win32::System::Threading::Sleep;
use windows::Win32::UI::WindowsAndMessaging::{
    ChildWindowFromPointEx, ClipCursor, EnumWindows, GetAncestor, GetClientRect, GetClipCursor,
    GetCursorPos, GetForegroundWindow, GetWindowLongW, GetWindowRect, IsWindowVisible,
    SetCursorPos, SetWindowLongW, SetWindowPos, SystemParametersInfoW,
    WindowFromPoint as WinApiWindowFromPoint, CWP_SKIPDISABLED, CWP_SKIPINVISIBLE,
    CWP_SKIPTRANSPARENT, GA_ROOT, GWL_EXSTYLE, HWND_NOTOPMOST, HWND_TOP, HWND_TOPMOST,
    SPI_GETMOUSESPEED, SPI_SETMOUSESPEED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, WS_EX_TRANSPARENT,
};

// ShowSystemCursor API の関数ポインタ型
type ShowSystemCursorFn = unsafe extern "system" fn(BOOL);

// 動的に取得したShowSystemCursor APIを格納
static SHOW_SYSTEM_CURSOR: OnceLock<Option<ShowSystemCursorFn>> = OnceLock::new();

fn get_show_system_cursor() -> Option<ShowSystemCursorFn> {
    *SHOW_SYSTEM_CURSOR.get_or_init(|| unsafe {
        let h_user32 = GetModuleHandleW(w!("user32.dll")).ok()?;
        let proc = GetProcAddress(h_user32, s!("ShowSystemCursor"))?;
        Some(std::mem::transmute::<_, ShowSystemCursorFn>(proc))
    })
}

/// ウィンドウが指定座標でマウス入力を受け取るかどうか (Magpie PtInWindow 相当)
fn pt_in_window(hwnd: HWND, pt: POINT) -> bool {
    unsafe {
        // 1. ウィンドウが可視かチェック
        if !IsWindowVisible(hwnd).as_bool() {
            return false;
        }

        // 2. ウィンドウ矩形内に含まれるか
        let mut window_rect = RECT::default();
        if GetWindowRect(hwnd, &mut window_rect).is_err() {
            return false;
        }
        if !PtInRect(&window_rect, pt).as_bool() {
            return false;
        }

        // 3. WS_EX_TRANSPARENT スタイルをチェック
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
        if (ex_style & WS_EX_TRANSPARENT.0) != 0 {
            return false;
        }

        // 4. DWMWA_CLOAKED をチェック
        let mut is_cloaked: u32 = 0;
        if DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED,
            &mut is_cloaked as *mut _ as *mut _,
            std::mem::size_of::<u32>() as u32,
        )
        .is_ok()
            && is_cloaked != 0
        {
            return false;
        }

        // 5. クライアント領域の透明性をチェック
        let mut client_rect = RECT::default();
        if GetClientRect(hwnd, &mut client_rect).is_ok() {
            let mut client_pt = POINT { x: 0, y: 0 };
            let _ = ClientToScreen(hwnd, &mut client_pt);
            let client_screen_rect = RECT {
                left: client_pt.x,
                top: client_pt.y,
                right: client_pt.x + (client_rect.right - client_rect.left),
                bottom: client_pt.y + (client_rect.bottom - client_rect.top),
            };

            if PtInRect(&client_screen_rect, pt).as_bool() {
                let local_pt = POINT {
                    x: pt.x - client_screen_rect.left,
                    y: pt.y - client_screen_rect.top,
                };
                windows::Win32::Foundation::SetLastError(windows::Win32::Foundation::WIN32_ERROR(
                    0,
                ));
                let child = ChildWindowFromPointEx(
                    hwnd,
                    local_pt,
                    CWP_SKIPINVISIBLE | CWP_SKIPDISABLED | CWP_SKIPTRANSPARENT,
                );
                if child == HWND(0 as _) {
                    let err = windows::Win32::Foundation::GetLastError();
                    if err.is_ok() {
                        return false;
                    }
                }
            }
        }

        // 6. カスタムウィンドウリージョンをチェック
        let h_rgn = CreateRectRgn(0, 0, 0, 0);
        let region_type = GetWindowRgn(hwnd, h_rgn);
        if region_type == GDI_REGION_TYPE(2) || region_type == GDI_REGION_TYPE(3) {
            if !PtInRegion(h_rgn, pt.x - window_rect.left, pt.y - window_rect.top).as_bool() {
                return false;
            }
        }

        true
    }
}

/// EnumWindows用のデータ構造
struct EnumData {
    result: HWND,
    scaling_hwnd: HWND,
    renderer_rect: RECT,
    pt: POINT,
    click_through_host: bool,
}

/// カーソル位置にあるウィンドウを検出 (Magpie WindowFromPoint 相当)
fn window_from_point(
    scaling_hwnd: HWND,
    renderer_rect: RECT,
    pt: POINT,
    click_through_host: bool,
) -> HWND {
    let mut data = EnumData {
        result: HWND::default(),
        scaling_hwnd,
        renderer_rect,
        pt,
        click_through_host,
    };

    unsafe {
        let _ = EnumWindows(
            Some(enum_windows_callback),
            LPARAM(&mut data as *mut EnumData as isize),
        );
    }

    data.result
}

unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let data = &mut *(lparam.0 as *mut EnumData);

    // Magpie方式: scaling_hwndの場合は特別処理
    if hwnd == data.scaling_hwnd {
        // Magpie: PtInRect(&data.rendererRect, data.pt) && !data.clickThroughHost
        if PtInRect(&data.renderer_rect, data.pt).as_bool() && !data.click_through_host {
            data.result = hwnd;
            return BOOL::from(false); // 列挙停止
        } else {
            return BOOL::from(true); // 続行
        }
    }

    if pt_in_window(hwnd, data.pt) {
        data.result = hwnd;
        return BOOL::from(false); // 列挙停止
    }

    BOOL::from(true) // 続行
}

pub struct CursorManager {
    pub is_under_capture: bool,
    pub should_draw_cursor: bool,
    pub cursor_pos: POINT, // _cursorPos (Magpie) - 常に拡大座標系
    pub draw_pos: POINT,   // 描画位置 (processor.rsとの互換性)
    scaling_hwnd: HWND,

    is_processing: bool,
    src_rect_cache: RECT,
    dest_rect_cache: RECT,
    is_system_cursor_shown: bool,
    original_mouse_speed: i32,
    last_src_focused: bool,
}

impl CursorManager {
    pub fn new(scaling_hwnd: HWND) -> Self {
        Self {
            is_under_capture: false,
            should_draw_cursor: false,
            cursor_pos: POINT::default(),
            draw_pos: POINT::default(),
            scaling_hwnd,
            is_processing: false,
            src_rect_cache: RECT::default(),
            dest_rect_cache: RECT::default(),
            is_system_cursor_shown: true,
            original_mouse_speed: 0,
            last_src_focused: false,
        }
    }

    /// 更新処理 (Magpie CursorManager::Update 相当)
    pub fn update(
        &mut self,
        src_rect: RECT,
        dest_rect: RECT,
        src_hwnd: HWND,
        is_src_focused: bool,
    ) -> Result<()> {
        // デバッグ: update呼び出し確認（最初の数回のみ）
        static mut UPDATE_DEBUG_COUNT: u32 = 0;
        unsafe {
            if UPDATE_DEBUG_COUNT < 5 {
                println!(
                    "[Cursor] update() called, is_src_focused={}",
                    is_src_focused
                );
                UPDATE_DEBUG_COUNT += 1;
            }
        }

        if self.is_processing {
            return Ok(());
        }
        self.is_processing = true;
        self.src_rect_cache = src_rect;
        self.dest_rect_cache = dest_rect;

        // フォーカス変化時のZ-order制御 (Magpie方式)
        self.handle_focus_change(is_src_focused);

        // 状態更新
        self.update_cursor_state(src_rect, dest_rect, src_hwnd);

        // カーソル位置更新 (Magpie _UpdateCursorPos 相当)
        self.update_cursor_pos(src_rect, dest_rect);

        self.is_processing = false;
        Ok(())
    }

    /// フォーカス変化時のZ-order制御
    fn handle_focus_change(&mut self, is_src_focused: bool) {
        // フォーカスが変化した時のみログを出力
        if is_src_focused != self.last_src_focused {
            println!(
                "[Cursor] Focus CHANGED: {} -> {}",
                self.last_src_focused, is_src_focused
            );
            self.last_src_focused = is_src_focused;

            unsafe {
                if is_src_focused {
                    // ソースがフォーカス: 拡大ウィンドウを最前面に
                    println!("[Cursor] Focus gained - setting TOPMOST");
                    let _ = SetWindowPos(
                        self.scaling_hwnd,
                        HWND_TOPMOST,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
                    );
                    let _ = SetWindowPos(
                        self.scaling_hwnd,
                        HWND_TOP,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
                    );
                } else {
                    // ソースがフォーカス外: 拡大ウィンドウの最前面を解除
                    println!("[Cursor] Focus lost - removing TOPMOST");
                    let _ = SetWindowPos(
                        self.scaling_hwnd,
                        HWND_NOTOPMOST,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
                    );

                    // フォーカスを失ったらキャプチャも解除してカーソルを拡大位置に移動
                    if self.is_under_capture {
                        println!("[Cursor] Releasing capture due to focus loss");
                        let mut pos = POINT::default();
                        let _ = GetCursorPos(&mut pos);

                        // stop_capture内でposが拡大座標に変換される
                        self.stop_capture(&mut pos);

                        // カーソルを拡大位置に移動
                        println!(
                            "[Cursor] Moving cursor to scaled position: ({}, {})",
                            pos.x, pos.y
                        );
                        self.reliable_set_cursor_pos(pos);

                        self.set_ex_transparent(false, unsafe {
                            GetWindowLongW(self.scaling_hwnd, GWL_EXSTYLE) as u32
                        });
                        self.should_draw_cursor = false;
                        self.show_system_cursor(true);
                    }
                }
            }
        }
    }

    /// カーソル状態の更新 (Magpie _UpdateCursorState 相当)
    fn update_cursor_state(&mut self, src_rect: RECT, dest_rect: RECT, src_hwnd: HWND) {
        let style = unsafe { GetWindowLongW(self.scaling_hwnd, GWL_EXSTYLE) as u32 };

        // Magpie: cursorPos を取得
        let mut cursor_pos = POINT::default();
        if unsafe { GetCursorPos(&mut cursor_pos) }.is_err() {
            return;
        }

        let origin_cursor_pos = cursor_pos;

        if self.is_under_capture {
            // ===== キャプチャ中 (Magpie L591-661) =====

            // フォアグラウンドウィンドウがソースウィンドウでない場合はキャプチャを停止
            // これによりスタートメニューなどのシステムUIが表示されたときに
            // カーソルを解放できる
            let foreground = unsafe { GetForegroundWindow() };
            if foreground != src_hwnd && foreground != self.scaling_hwnd {
                // 別のウィンドウがフォアグラウンドにある - キャプチャ停止
                println!("[Cursor] Foreground changed, stopping capture");
                self.should_draw_cursor = false;
                self.set_ex_transparent(false, style);
                self.stop_capture(&mut cursor_pos);
                self.reliable_set_cursor_pos(cursor_pos);
                self.show_system_cursor(true);
                return;
            }

            // 拡大後の座標でWindowFromPointを呼び出し
            let scaled_pos = self.src_to_scaling(cursor_pos);

            // まず、scaled_posがdest_rect内にあるかチェック
            let in_dest_rect = scaled_pos.x >= dest_rect.left
                && scaled_pos.x < dest_rect.right
                && scaled_pos.y >= dest_rect.top
                && scaled_pos.y < dest_rect.bottom;

            if !in_dest_rect {
                // カーソルが拡大ウィンドウ外に出た - キャプチャ停止
                println!("[Cursor] Cursor outside dest_rect");
                self.should_draw_cursor = false;
                self.set_ex_transparent(false, style);
                if self.stop_capture(&mut cursor_pos) {
                    self.reliable_set_cursor_pos(cursor_pos);
                }
            } else {
                // dest_rect内にいる場合、他のウィンドウが上にあるかチェック
                // WS_EX_TRANSPARENTを一時的に解除してチェック
                let has_transparent = (style & WS_EX_TRANSPARENT.0) != 0;
                if has_transparent {
                    unsafe {
                        SetWindowLongW(
                            self.scaling_hwnd,
                            GWL_EXSTYLE,
                            (style & !WS_EX_TRANSPARENT.0) as i32,
                        );
                    }
                }

                let win_api_hwnd = unsafe { WinApiWindowFromPoint(scaled_pos) };
                let win_api_root = unsafe { GetAncestor(win_api_hwnd, GA_ROOT) };
                let is_scaling_visible =
                    win_api_root == self.scaling_hwnd || win_api_hwnd == self.scaling_hwnd;

                // WS_EX_TRANSPARENTを元に戻す
                if has_transparent {
                    unsafe {
                        SetWindowLongW(self.scaling_hwnd, GWL_EXSTYLE, style as i32);
                    }
                }

                self.should_draw_cursor = is_scaling_visible;

                if is_scaling_visible {
                    // 拡大ウィンドウが見えている - ソースウィンドウが隠されているかチェック
                    let win_api_src_hwnd = unsafe { WinApiWindowFromPoint(cursor_pos) };
                    let win_api_src_root = unsafe { GetAncestor(win_api_src_hwnd, GA_ROOT) };
                    let src_hidden = win_api_src_root != src_hwnd && win_api_src_hwnd != src_hwnd;

                    if src_hidden {
                        // ソースが隠されている - キャプチャ停止
                        println!("[Cursor] Stopping - src hidden");
                        self.set_ex_transparent(false, style);
                        self.stop_capture(&mut cursor_pos);
                        self.reliable_set_cursor_pos(cursor_pos);
                    } else {
                        // 正常動作 - 透明を維持
                        self.set_ex_transparent(true, style);
                    }
                } else {
                    // 拡大ウィンドウが他のウィンドウで隠されている - キャプチャ停止
                    println!("[Cursor] Stopping - scaling hidden by {:?}", win_api_root.0);
                    self.set_ex_transparent(false, style);
                    if self.stop_capture(&mut cursor_pos) {
                        self.reliable_set_cursor_pos(cursor_pos);
                    }
                }
            }
        } else {
            // ===== 非キャプチャ中 (Magpie L662-776) =====

            // フォアグラウンドウィンドウをチェック
            let foreground = unsafe { GetForegroundWindow() };
            let is_src_foreground = foreground == src_hwnd || foreground == self.scaling_hwnd;

            // カーソルがdest_rect内にあるかチェック
            let in_dest_rect = cursor_pos.x >= dest_rect.left
                && cursor_pos.x < dest_rect.right
                && cursor_pos.y >= dest_rect.top
                && cursor_pos.y < dest_rect.bottom;

            if is_src_foreground {
                // ソースがフォアグラウンド (Magpie L679-718)
                // Magpie方式: 自前のwindow_from_point(clickThroughHost=false)を使用
                let hwnd_cur = window_from_point(self.scaling_hwnd, dest_rect, cursor_pos, false);
                self.should_draw_cursor = hwnd_cur == self.scaling_hwnd;

                if self.should_draw_cursor {
                    let new_cursor_pos =
                        Self::scaling_to_src_static(cursor_pos, src_rect, dest_rect);

                    // ソース領域内かチェック (Magpie L683)
                    if new_cursor_pos.x >= src_rect.left
                        && new_cursor_pos.x < src_rect.right
                        && new_cursor_pos.y >= src_rect.top
                        && new_cursor_pos.y < src_rect.bottom
                    {
                        // ソースウィンドウが隠されていないかチェック (Magpie L686-691)
                        let hwnd_at_src =
                            window_from_point(self.scaling_hwnd, dest_rect, new_cursor_pos, true);
                        let start_capture = hwnd_at_src == src_hwnd;

                        if start_capture {
                            // キャプチャ開始 (Magpie L708-713)
                            self.set_ex_transparent(true, style);
                            self.start_capture(&mut cursor_pos);
                        } else {
                            // ソースが隠されている - 透明解除 (Magpie L714-717)
                            self.set_ex_transparent(false, style);
                        }
                    } else {
                        // ソース領域外 - 透明解除
                        self.set_ex_transparent(false, style);
                    }
                }
            } else {
                // ソースがフォアグラウンドでない (Magpie L760-774)
                // カスタムカーソルは描画しない
                self.should_draw_cursor = false;

                // destRect内でも透明にするだけでキャプチャは開始しない
                // （キャプチャするとカーソル位置がジャンプしてループする）
                if in_dest_rect {
                    self.set_ex_transparent(true, style);
                } else {
                    self.set_ex_transparent(false, style);
                }
            }
        }

        // システムカーソルの表示制御 (Magpie L785)
        self.show_system_cursor(!self.should_draw_cursor);

        // カーソル位置が変更された場合は移動 (Magpie L789-792)
        if cursor_pos.x != origin_cursor_pos.x || cursor_pos.y != origin_cursor_pos.y {
            self.reliable_set_cursor_pos(cursor_pos);
        }
    }

    /// カーソル位置更新 (Magpie _UpdateCursorPos 相当)
    fn update_cursor_pos(&mut self, src_rect: RECT, dest_rect: RECT) {
        let mut cursor_pos = POINT::default();
        if unsafe { GetCursorPos(&mut cursor_pos) }.is_err() {
            return;
        }

        // キャプチャ中はソース座標を拡大座標に変換 (Magpie L1103-1104)
        if self.is_under_capture {
            self.cursor_pos = self.src_to_scaling(cursor_pos);
        } else {
            self.cursor_pos = cursor_pos;
        }

        // draw_pos は常に拡大座標系 (processor.rsで使用)
        self.draw_pos = self.cursor_pos;
    }

    fn set_ex_transparent(&self, transparent: bool, current_style: u32) {
        unsafe {
            let has_transparent = (current_style & WS_EX_TRANSPARENT.0) != 0;
            if transparent && !has_transparent {
                SetWindowLongW(
                    self.scaling_hwnd,
                    GWL_EXSTYLE,
                    (current_style | WS_EX_TRANSPARENT.0) as i32,
                );
            } else if !transparent && has_transparent {
                SetWindowLongW(
                    self.scaling_hwnd,
                    GWL_EXSTYLE,
                    (current_style & !WS_EX_TRANSPARENT.0) as i32,
                );
            }
        }
    }

    fn show_system_cursor(&mut self, show: bool) {
        if self.is_system_cursor_shown == show {
            return;
        }
        if let Some(show_cursor_fn) = get_show_system_cursor() {
            unsafe {
                show_cursor_fn(BOOL::from(show));
            }
            self.is_system_cursor_shown = show;
        }
    }

    /// キャプチャ開始 (Magpie _StartCapture 相当)
    fn start_capture(&mut self, cursor_pos: &mut POINT) {
        if self.is_under_capture {
            return;
        }

        // カーソル速度調整 (Magpie L1126)
        self.adjust_cursor_speed();

        // カーソル位置をソース座標に変換 (Magpie L1129-1132)
        let clamped = POINT {
            x: cursor_pos
                .x
                .clamp(self.dest_rect_cache.left, self.dest_rect_cache.right - 1),
            y: cursor_pos
                .y
                .clamp(self.dest_rect_cache.top, self.dest_rect_cache.bottom - 1),
        };
        *cursor_pos =
            Self::scaling_to_src_static(clamped, self.src_rect_cache, self.dest_rect_cache);

        self.is_under_capture = true;

        // カーソルをソース領域にクリップ
        unsafe {
            let _ = ClipCursor(Some(&self.src_rect_cache));
        }
    }

    /// キャプチャ停止 (Magpie _StopCapture 相当)
    fn stop_capture(&mut self, cursor_pos: &mut POINT) -> bool {
        if !self.is_under_capture {
            return true;
        }

        // カーソル位置を拡大座標に変換 (Magpie L1159-1161)
        let new_cursor_pos = self.src_to_scaling(*cursor_pos);
        *cursor_pos = new_cursor_pos;

        self.restore_cursor_speed();
        self.is_under_capture = false;

        unsafe {
            let _ = ClipCursor(None);
        }

        true
    }

    pub fn stop_capture_public(&mut self) {
        let mut pos = POINT::default();
        let _ = unsafe { GetCursorPos(&mut pos) };
        self.stop_capture(&mut pos);
    }

    fn adjust_cursor_speed(&mut self) {
        let src_width = (self.src_rect_cache.right - self.src_rect_cache.left).max(1) as f64;
        let src_height = (self.src_rect_cache.bottom - self.src_rect_cache.top).max(1) as f64;
        let dest_width = (self.dest_rect_cache.right - self.dest_rect_cache.left).max(1) as f64;
        let dest_height = (self.dest_rect_cache.bottom - self.dest_rect_cache.top).max(1) as f64;

        let scale = ((dest_width / src_width) + (dest_height / src_height)) / 2.0;
        if scale <= 1.0 {
            return;
        }

        unsafe {
            let mut current_speed: i32 = 0;
            if SystemParametersInfoW(
                SPI_GETMOUSESPEED,
                0,
                Some(&mut current_speed as *mut _ as *mut _),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
            )
            .is_err()
            {
                return;
            }
            self.original_mouse_speed = current_speed;
            let new_speed = ((current_speed as f64) / scale).round() as i32;
            let new_speed = new_speed.clamp(1, 20);
            let _ = SystemParametersInfoW(
                SPI_SETMOUSESPEED,
                0,
                Some(new_speed as *mut _),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
            );
        }
    }

    fn restore_cursor_speed(&mut self) {
        if self.original_mouse_speed == 0 {
            return;
        }
        unsafe {
            let _ = SystemParametersInfoW(
                SPI_SETMOUSESPEED,
                0,
                Some(self.original_mouse_speed as *mut _),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
            );
        }
        self.original_mouse_speed = 0;
    }

    fn reliable_set_cursor_pos(&self, pos: POINT) {
        unsafe {
            let mut origin_clip = RECT::default();
            let _ = GetClipCursor(&mut origin_clip);

            let temp_clip = RECT {
                left: pos.x,
                top: pos.y,
                right: pos.x + 1,
                bottom: pos.y + 1,
            };
            let _ = ClipCursor(Some(&temp_clip));
            Sleep(8);
            let _ = ClipCursor(Some(&origin_clip));
        }
    }

    /// ソース座標を拡大座標に変換 (Magpie SrcToScaling)
    fn src_to_scaling(&self, pt: POINT) -> POINT {
        Self::src_to_scaling_static(pt, self.src_rect_cache, self.dest_rect_cache)
    }

    pub fn src_to_scaling_static(pt: POINT, src_rect: RECT, dest_rect: RECT) -> POINT {
        let src_width = (src_rect.right - src_rect.left).max(1);
        let src_height = (src_rect.bottom - src_rect.top).max(1);
        let dest_width = (dest_rect.right - dest_rect.left).max(1);
        let dest_height = (dest_rect.bottom - dest_rect.top).max(1);

        let x_ratio = (pt.x - src_rect.left) as f64 / src_width as f64;
        let y_ratio = (pt.y - src_rect.top) as f64 / src_height as f64;

        POINT {
            x: dest_rect.left + (x_ratio * dest_width as f64).round() as i32,
            y: dest_rect.top + (y_ratio * dest_height as f64).round() as i32,
        }
    }

    /// 拡大座標をソース座標に変換 (Magpie ScalingToSrc)
    pub fn scaling_to_src_static(pt: POINT, src_rect: RECT, dest_rect: RECT) -> POINT {
        let src_width = (src_rect.right - src_rect.left).max(1);
        let src_height = (src_rect.bottom - src_rect.top).max(1);
        let dest_width = (dest_rect.right - dest_rect.left).max(1);
        let dest_height = (dest_rect.bottom - dest_rect.top).max(1);

        let x_ratio = (pt.x - dest_rect.left) as f64 / dest_width as f64;
        let y_ratio = (pt.y - dest_rect.top) as f64 / dest_height as f64;

        POINT {
            x: src_rect.left + (x_ratio * src_width as f64).round() as i32,
            y: src_rect.top + (y_ratio * src_height as f64).round() as i32,
        }
    }
}

impl Drop for CursorManager {
    fn drop(&mut self) {
        self.show_system_cursor(true);
        self.restore_cursor_speed();
        unsafe {
            let _ = ClipCursor(None);
        }
    }
}
