//! CursorManager - Magpieの実装をRustで再実装
//!
//! カーソルのキャプチャ、座標変換、状態管理を担当

use std::sync::OnceLock;
use std::time::Instant;
use windows::core::{s, w};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::Graphics::Gdi::{
    ClientToScreen, CreateRectRgn, EnumDisplayMonitors, GetMonitorInfoW, GetWindowRgn,
    MonitorFromPoint, PtInRect, PtInRegion, GDI_REGION_TYPE, HDC, HMONITOR, MONITORINFO,
    MONITOR_DEFAULTTONULL,
};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows::Win32::System::Threading::Sleep;
use windows::Win32::UI::WindowsAndMessaging::{
    ChildWindowFromPointEx, ClipCursor, EnumWindows, GetAncestor, GetClientRect, GetClipCursor,
    GetCursorPos, GetForegroundWindow, GetGUIThreadInfo, GetWindowLongPtrW, GetWindowRect, IsChild,
    IsWindowVisible, IsZoomed, SendMessageTimeoutW, SetWindowLongPtrW, SetWindowPos,
    SystemParametersInfoW, WindowFromPoint as WinApiWindowFromPoint, CWP_SKIPDISABLED,
    CWP_SKIPINVISIBLE, CWP_SKIPTRANSPARENT, GA_ROOT, GUITHREADINFO, GUI_INMENUMODE, GUI_INMOVESIZE,
    GUI_POPUPMENUMODE, GUI_SYSTEMMENUMODE, GWL_EXSTYLE, GWL_STYLE, HTSIZEFIRST, HTSIZELAST,
    HWND_NOTOPMOST, HWND_TOP, HWND_TOPMOST, SMTO_ABORTIFHUNG, SPI_GETMOUSESPEED, SPI_SETMOUSESPEED,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, WM_NCHITTEST,
    WS_CHILD, WS_EX_TRANSPARENT,
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

/// 丸めモード（ScalingToSrc用）
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RoundMethod {
    Round,
    Floor,
    Ceil,
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
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
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
                if child == HWND::default() {
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

    if hwnd == data.scaling_hwnd {
        if PtInRect(&data.renderer_rect, data.pt).as_bool() && !data.click_through_host {
            data.result = hwnd;
            return BOOL::from(false);
        } else {
            return BOOL::from(true);
        }
    }

    if pt_in_window(hwnd, data.pt) {
        data.result = hwnd;
        return BOOL::from(false);
    }

    BOOL::from(true)
}

/// エッジ領域かどうか (Magpie IsEdgeArea)
fn is_edge_area(area: i16) -> bool {
    area >= HTSIZEFIRST as i16 && area <= HTSIZELAST as i16
}

/// HitTest実行 (Magpie AdvancedWindowHitTest簡易版)
fn window_hit_test(hwnd: HWND, pt: POINT, timeout_ms: u32) -> i16 {
    if unsafe { IsZoomed(hwnd).as_bool() } {
        return 0; // HTNOWHERE
    }

    let x = pt.x as i16;
    let y = pt.y as i16;
    let lparam_val = (x as u32 & 0xFFFF) | ((y as u32 & 0xFFFF) << 16);

    let mut result: usize = 0;
    let send_result = unsafe {
        SendMessageTimeoutW(
            hwnd,
            WM_NCHITTEST,
            WPARAM(0),
            LPARAM(lparam_val as isize),
            SMTO_ABORTIFHUNG,
            timeout_ms,
            Some(&mut result),
        )
    };

    if send_result.0 != 0 {
        result as i16
    } else {
        0 // HTNOWHERE
    }
}

/// モニター矩形を取得
fn obtain_monitor_rects() -> Vec<RECT> {
    let mut rects: Vec<RECT> = Vec::new();
    unsafe {
        let _ = EnumDisplayMonitors(
            HDC::default(),
            None,
            Some(enum_monitor_proc),
            LPARAM(&mut rects as *mut Vec<RECT> as isize),
        );
    }
    rects
}

unsafe extern "system" fn enum_monitor_proc(
    _hmonitor: HMONITOR,
    _hdc: HDC,
    lprect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let rects = &mut *(lparam.0 as *mut Vec<RECT>);
    if !lprect.is_null() {
        rects.push(*lprect);
    }
    BOOL::from(true)
}

/// 矩形が重なっているかチェック
fn is_rect_overlap(r1: &RECT, r2: &RECT) -> bool {
    r1.right > r2.left && r1.bottom > r2.top && r1.left < r2.right && r1.top < r2.bottom
}

/// いずれかのモニターと交差するか
fn any_intersected_monitor(monitor_rects: &[RECT], test_rect: &RECT) -> bool {
    monitor_rects.iter().any(|r| is_rect_overlap(r, test_rect))
}

/// CursorManager - カーソル管理の中核
pub struct CursorManager {
    // 基本ハンドル
    scaling_hwnd: HWND,

    // 状態フラグ
    is_under_capture: bool,
    should_draw_cursor: bool,
    is_captured_on_foreground: bool,
    is_on_overlay: bool,
    is_captured_on_overlay: bool,
    is_system_cursor_shown: bool,

    // カーソル情報
    cursor_pos: POINT,
    h_cursor: isize,

    // 描画位置（processor.rs互換用）
    draw_pos: POINT,

    // ウィンドウ移動時の安定化用
    local_cursor_pos_on_moving: POINT,

    // リサイズカーソル遅延用
    size_cursor_start_time: Option<Instant>,

    // ClipCursor最適化用
    last_clip: RECT,
    last_real_clip: RECT,

    // カーソル速度調整
    origin_cursor_speed: i32,

    // HitTest関連
    next_hit_test_id: u32,
    last_completed_hit_test_id: u32,
    last_completed_hit_test_pos: POINT,
    last_completed_hit_test_result: i16,

    // 矩形キャッシュ
    src_rect: RECT,
    dest_rect: RECT,
    renderer_rect: RECT,

    // ソースウィンドウ
    src_hwnd: HWND,
}

impl CursorManager {
    pub fn new(scaling_hwnd: HWND) -> Self {
        Self {
            scaling_hwnd,
            is_under_capture: false,
            should_draw_cursor: false,
            is_captured_on_foreground: false,
            is_on_overlay: false,
            is_captured_on_overlay: false,
            is_system_cursor_shown: true,
            cursor_pos: POINT {
                x: i32::MAX,
                y: i32::MAX,
            },
            h_cursor: 0,

            // 描画位置
            draw_pos: POINT::default(),
            local_cursor_pos_on_moving: POINT {
                x: i32::MAX,
                y: i32::MAX,
            },
            size_cursor_start_time: None,
            last_clip: RECT {
                left: i32::MAX,
                top: i32::MAX,
                right: i32::MAX,
                bottom: i32::MAX,
            },
            last_real_clip: RECT {
                left: i32::MAX,
                top: i32::MAX,
                right: i32::MAX,
                bottom: i32::MAX,
            },
            origin_cursor_speed: 0,
            next_hit_test_id: 0,
            last_completed_hit_test_id: 0,
            last_completed_hit_test_pos: POINT {
                x: i32::MAX,
                y: i32::MAX,
            },
            last_completed_hit_test_result: 0, // HTNOWHERE
            src_rect: RECT::default(),
            dest_rect: RECT::default(),
            renderer_rect: RECT::default(),
            src_hwnd: HWND::default(),
        }
    }

    /// 更新（毎フレーム呼び出し）
    pub fn update(
        &mut self,
        src_hwnd: HWND,
        src_rect: RECT,
        dest_rect: RECT,
        renderer_rect: RECT,
        is_src_moving: bool,
        is_resizing_or_moving: bool,
    ) {
        self.src_hwnd = src_hwnd;
        self.src_rect = src_rect;
        self.dest_rect = dest_rect;
        self.renderer_rect = renderer_rect;

        self.update_cursor_state(is_src_moving, is_resizing_or_moving);
        self.update_cursor_pos(is_src_moving, is_resizing_or_moving);
    }

    /// カーソルハンドルを取得
    pub fn cursor_handle(&self) -> isize {
        self.h_cursor
    }

    /// カーソル位置を取得（スクリーン座標）
    pub fn cursor_pos(&self) -> POINT {
        self.cursor_pos
    }

    /// カーソルを描画すべきか
    pub fn should_draw_cursor(&self) -> bool {
        self.should_draw_cursor
    }

    /// キャプチャ中か
    pub fn is_cursor_captured(&self) -> bool {
        self.is_under_capture
    }

    /// オーバーレイ上にカーソルがあるか設定
    pub fn set_cursor_on_overlay(&mut self, value: bool) {
        if self.is_on_overlay != value {
            self.is_on_overlay = value;
        }
    }

    /// オーバーレイにキャプチャされているか設定
    pub fn set_cursor_captured_on_overlay(&mut self, value: bool) {
        if self.is_captured_on_overlay != value {
            self.is_captured_on_overlay = value;
        }
    }

    /// 描画位置を取得（processor.rs互換用）
    pub fn draw_pos(&self) -> POINT {
        self.draw_pos
    }

    /// カーソルハンドルを取得（リサイズカーソルフィルタリング付き）
    pub fn get_cursor_handle(&self, _current_cursor: isize) -> isize {
        // リサイズカーソルの場合は通常の矢印カーソルに置き換え
        // スケーリング中はソースウィンドウの境界でリサイズカーソルが表示されるのを防ぐ
        if self.is_size_cursor(self.h_cursor) {
            unsafe {
                use windows::Win32::UI::WindowsAndMessaging::{LoadCursorW, IDC_ARROW};
                LoadCursorW(None, IDC_ARROW)
                    .map(|c| c.0 as isize)
                    .unwrap_or(self.h_cursor)
            }
        } else {
            self.h_cursor
        }
    }

    // ========== 座標変換 ==========

    /// ソース座標を拡大座標に変換 (Magpie SrcToScaling)
    pub fn src_to_scaling(&self, pt: POINT, skip_border: bool) -> POINT {
        Self::src_to_scaling_static(
            pt,
            self.src_rect,
            self.dest_rect,
            self.renderer_rect,
            skip_border,
        )
    }

    pub fn src_to_scaling_static(
        pt: POINT,
        src_rect: RECT,
        dest_rect: RECT,
        renderer_rect: RECT,
        skip_border: bool,
    ) -> POINT {
        let mut result = POINT::default();

        // X座標
        if pt.x >= src_rect.right {
            let base = if skip_border {
                renderer_rect.right
            } else {
                dest_rect.right
            };
            result.x = base + pt.x - src_rect.right;
        } else if pt.x < src_rect.left {
            let base = if skip_border {
                renderer_rect.left
            } else {
                dest_rect.left
            };
            result.x = base + pt.x - src_rect.left;
        } else {
            let src_width = src_rect.right - src_rect.left - 1;
            let dest_width = dest_rect.right - dest_rect.left - 1;
            if src_width > 0 {
                let pos = (pt.x - src_rect.left) as f64 / src_width as f64;
                result.x = (pos * dest_width as f64).round() as i32 + dest_rect.left;
            } else {
                result.x = dest_rect.left;
            }
        }

        // Y座標
        if pt.y >= src_rect.bottom {
            let base = if skip_border {
                renderer_rect.bottom
            } else {
                dest_rect.bottom
            };
            result.y = base + pt.y - src_rect.bottom;
        } else if pt.y < src_rect.top {
            let base = if skip_border {
                renderer_rect.top
            } else {
                dest_rect.top
            };
            result.y = base + pt.y - src_rect.top;
        } else {
            let src_height = src_rect.bottom - src_rect.top - 1;
            let dest_height = dest_rect.bottom - dest_rect.top - 1;
            if src_height > 0 {
                let pos = (pt.y - src_rect.top) as f64 / src_height as f64;
                result.y = (pos * dest_height as f64).round() as i32 + dest_rect.top;
            } else {
                result.y = dest_rect.top;
            }
        }

        result
    }

    /// 拡大座標をソース座標に変換 (Magpie ScalingToSrc)
    pub fn scaling_to_src(&self, pt: POINT, round_method: RoundMethod) -> POINT {
        Self::scaling_to_src_static(pt, self.src_rect, self.dest_rect, round_method)
    }

    pub fn scaling_to_src_static(
        pt: POINT,
        src_rect: RECT,
        dest_rect: RECT,
        round_method: RoundMethod,
    ) -> POINT {
        let src_width = src_rect.right - src_rect.left;
        let src_height = src_rect.bottom - src_rect.top;
        let dest_width = dest_rect.right - dest_rect.left;
        let dest_height = dest_rect.bottom - dest_rect.top;

        let mut result = POINT {
            x: src_rect.left,
            y: src_rect.top,
        };

        // X座標
        if pt.x >= dest_rect.right {
            result.x += src_width + pt.x - dest_rect.right;
        } else if pt.x < dest_rect.left {
            result.x += pt.x - dest_rect.left;
        } else if dest_width > 1 {
            let pos = (pt.x - dest_rect.left) as f64 / (dest_width - 1) as f64;
            let delta = pos * (src_width - 1) as f64;
            result.x += match round_method {
                RoundMethod::Round => delta.round() as i32,
                RoundMethod::Floor => delta.floor() as i32,
                RoundMethod::Ceil => delta.ceil() as i32,
            };
        }

        // Y座標
        if pt.y >= dest_rect.bottom {
            result.y += src_height + pt.y - dest_rect.bottom;
        } else if pt.y < dest_rect.top {
            result.y += pt.y - dest_rect.top;
        } else if dest_height > 1 {
            let pos = (pt.y - dest_rect.top) as f64 / (dest_height - 1) as f64;
            let delta = pos * (src_height - 1) as f64;
            result.y += match round_method {
                RoundMethod::Round => delta.round() as i32,
                RoundMethod::Floor => delta.floor() as i32,
                RoundMethod::Ceil => delta.ceil() as i32,
            };
        }

        result
    }

    // ========== 状態更新 ==========

    fn update_cursor_state(&mut self, is_src_moving: bool, is_resizing_or_moving: bool) {
        if is_resizing_or_moving {
            self.restore_clip_cursor();
            return;
        }

        if is_src_moving {
            if self.is_under_capture {
                self.clip_cursor_on_src_moving();
            } else {
                self.restore_clip_cursor();
            }
            return;
        }

        if self.is_captured_on_overlay {
            self.set_clip_cursor(&self.dest_rect.clone());
            return;
        }

        // 前景ウィンドウのキャプチャ状態をチェック
        self.is_captured_on_foreground = false;
        unsafe {
            let mut info = GUITHREADINFO {
                cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
                ..Default::default()
            };
            if GetGUIThreadInfo(0, &mut info).is_ok() {
                if info.hwndCapture != HWND::default()
                    && (info.flags.0
                        & (GUI_INMENUMODE.0 | GUI_POPUPMENUMODE.0 | GUI_SYSTEMMENUMODE.0))
                        == 0
                {
                    self.is_captured_on_foreground = true;

                    if info.hwndCapture == self.src_hwnd
                        && self.local_cursor_pos_on_moving.x == i32::MAX
                    {
                        self.local_cursor_pos_on_moving.x =
                            self.cursor_pos.x - self.renderer_rect.left;
                        self.local_cursor_pos_on_moving.y =
                            self.cursor_pos.y - self.renderer_rect.top;
                    }

                    if self.is_under_capture && (info.flags.0 & GUI_INMOVESIZE.0) == 0 {
                        self.set_clip_cursor(&self.src_rect.clone());
                    } else {
                        self.restore_clip_cursor();
                    }
                    return;
                }

                if info.hwndCapture != self.src_hwnd {
                    self.local_cursor_pos_on_moving.x = i32::MAX;
                }
            }
        }

        let style = unsafe { GetWindowLongPtrW(self.scaling_hwnd, GWL_EXSTYLE) as u32 };

        let mut cursor_pos = POINT::default();
        if unsafe { GetCursorPos(&mut cursor_pos) }.is_err() {
            self.restore_clip_cursor();
            return;
        }

        let origin_cursor_pos = cursor_pos;
        let is_src_focused = unsafe { GetForegroundWindow() } == self.src_hwnd
            || unsafe { GetForegroundWindow() } == self.scaling_hwnd;

        let mut should_clear_hit_test = true;

        if self.is_under_capture {
            // キャプチャ中
            let scaled_pos = self.src_to_scaling(cursor_pos, is_src_focused);
            let hwnd_cur =
                window_from_point(self.scaling_hwnd, self.renderer_rect, scaled_pos, false);
            self.should_draw_cursor = hwnd_cur == self.scaling_hwnd;

            if self.should_draw_cursor {
                let mut stop_capture = self.is_on_overlay;

                if !stop_capture {
                    let hwnd_at_src =
                        window_from_point(self.scaling_hwnd, self.renderer_rect, cursor_pos, true);

                    stop_capture = hwnd_at_src != self.src_hwnd
                        && (!unsafe { IsChild(self.src_hwnd, hwnd_at_src).as_bool() }
                            || (unsafe { GetWindowLongPtrW(hwnd_at_src, GWL_STYLE) as u32 }
                                & WS_CHILD.0)
                                == 0);

                    if !stop_capture {
                        should_clear_hit_test = false;

                        if self.last_completed_hit_test_pos != cursor_pos {
                            // 非同期HitTest（簡易実装：同期で実行）
                            let result = window_hit_test(self.src_hwnd, cursor_pos, 100);
                            self.last_completed_hit_test_id = self.next_hit_test_id;
                            self.next_hit_test_id += 1;
                            self.last_completed_hit_test_pos = cursor_pos;
                            self.last_completed_hit_test_result = result;
                        }

                        stop_capture = is_edge_area(self.last_completed_hit_test_result);
                    }
                }

                if stop_capture {
                    self.set_ex_transparent(false, style);
                    self.stop_capture(&mut cursor_pos);
                } else if self.is_on_overlay {
                    self.set_ex_transparent(false, style);
                } else {
                    self.set_ex_transparent(true, style);
                }
            } else {
                self.set_ex_transparent(false, style);
                if !self.stop_capture(&mut cursor_pos) {
                    self.should_draw_cursor = true;
                }
            }
        } else {
            // 非キャプチャ中
            let hwnd_cur =
                window_from_point(self.scaling_hwnd, self.renderer_rect, cursor_pos, false);
            self.should_draw_cursor = hwnd_cur == self.scaling_hwnd;

            if self.should_draw_cursor {
                let new_cursor_pos = self.scaling_to_src(cursor_pos, RoundMethod::Round);

                if new_cursor_pos.x >= self.src_rect.left
                    && new_cursor_pos.x < self.src_rect.right
                    && new_cursor_pos.y >= self.src_rect.top
                    && new_cursor_pos.y < self.src_rect.bottom
                {
                    let mut start_capture = !self.is_on_overlay;

                    if start_capture {
                        let hwnd_at_src = window_from_point(
                            self.scaling_hwnd,
                            self.renderer_rect,
                            new_cursor_pos,
                            true,
                        );

                        start_capture = hwnd_at_src == self.src_hwnd
                            || (unsafe { IsChild(self.src_hwnd, hwnd_at_src).as_bool() }
                                && (unsafe { GetWindowLongPtrW(hwnd_at_src, GWL_STYLE) as u32 }
                                    & WS_CHILD.0)
                                    != 0);

                        if start_capture {
                            should_clear_hit_test = false;

                            if self.last_completed_hit_test_pos != new_cursor_pos {
                                let result = window_hit_test(self.src_hwnd, new_cursor_pos, 100);
                                self.last_completed_hit_test_id = self.next_hit_test_id;
                                self.next_hit_test_id += 1;
                                self.last_completed_hit_test_pos = new_cursor_pos;
                                self.last_completed_hit_test_result = result;
                            }

                            start_capture = !is_edge_area(self.last_completed_hit_test_result);
                        }
                    }

                    if start_capture {
                        self.set_ex_transparent(true, style);
                        self.start_capture(&mut cursor_pos);
                    } else {
                        self.set_ex_transparent(false, style);
                    }
                } else if is_src_focused {
                    // 黒帯スキップロジック（簡略化）
                    let clamped_pos = POINT {
                        x: cursor_pos
                            .x
                            .clamp(self.dest_rect.left, self.dest_rect.right - 1),
                        y: cursor_pos
                            .y
                            .clamp(self.dest_rect.top, self.dest_rect.bottom - 1),
                    };

                    if window_from_point(self.scaling_hwnd, self.renderer_rect, clamped_pos, false)
                        == self.scaling_hwnd
                    {
                        self.set_ex_transparent(true, style);
                        self.start_capture(&mut cursor_pos);
                    } else {
                        self.set_ex_transparent(false, style);
                    }
                } else if !self.is_on_overlay
                    && cursor_pos.x >= self.dest_rect.left
                    && cursor_pos.x < self.dest_rect.right
                    && cursor_pos.y >= self.dest_rect.top
                    && cursor_pos.y < self.dest_rect.bottom
                {
                    self.set_ex_transparent(true, style);
                    self.start_capture(&mut cursor_pos);
                } else {
                    self.set_ex_transparent(false, style);
                }
            }
        }

        if should_clear_hit_test {
            self.clear_hit_test_result();
        }

        self.show_system_cursor(!self.should_draw_cursor);

        self.clip_cursor_for_monitors(cursor_pos);

        if cursor_pos != origin_cursor_pos {
            self.reliable_set_cursor_pos(cursor_pos);
        }
    }

    fn update_cursor_pos(&mut self, is_src_moving: bool, is_resizing_or_moving: bool) {
        if self.should_draw_cursor {
            unsafe {
                let mut ci = windows::Win32::UI::WindowsAndMessaging::CURSORINFO {
                    cbSize: std::mem::size_of::<windows::Win32::UI::WindowsAndMessaging::CURSORINFO>(
                    ) as u32,
                    ..Default::default()
                };
                if windows::Win32::UI::WindowsAndMessaging::GetCursorInfo(&mut ci).is_err() {
                    self.h_cursor = 0;
                    return;
                }

                if ci.flags.0 == 1 {
                    // CURSOR_SHOWING
                    // リサイズカーソル遅延処理
                    let is_size_cursor = self.is_size_cursor(ci.hCursor.0 as isize);
                    if is_size_cursor {
                        // リサイズカーソルは矢印カーソルに置き換える
                        use windows::Win32::UI::WindowsAndMessaging::{
                            LoadCursorW, SetCursor, IDC_ARROW,
                        };
                        if let Ok(arrow) = LoadCursorW(None, IDC_ARROW) {
                            self.h_cursor = arrow.0 as isize;
                            // システムカーソルも矢印に変更
                            SetCursor(arrow);
                        }
                        self.size_cursor_start_time = None;
                    } else {
                        self.size_cursor_start_time = None;
                        self.h_cursor = ci.hCursor.0 as isize;
                    }
                } else {
                    self.h_cursor = 0;
                }

                self.cursor_pos = ci.ptScreenPos;
            }
        } else {
            self.h_cursor = 0;
            unsafe {
                let _ = GetCursorPos(&mut self.cursor_pos);
            }
        }

        // ドラッグ中のカーソル位置安定化
        let is_scaling_moving = !self.is_under_capture
            && is_resizing_or_moving
            && self.local_cursor_pos_on_moving.x != i32::MAX;

        if is_src_moving || is_scaling_moving {
            self.cursor_pos.x = self.local_cursor_pos_on_moving.x + self.renderer_rect.left;
            self.cursor_pos.y = self.local_cursor_pos_on_moving.y + self.renderer_rect.top;
        } else if self.is_under_capture {
            self.cursor_pos = self.src_to_scaling(self.cursor_pos, false);
        }

        // draw_posを更新
        self.draw_pos = self.cursor_pos;
    }

    fn is_size_cursor(&self, h_cursor: isize) -> bool {
        use windows::Win32::UI::WindowsAndMessaging::{
            LoadCursorW, IDC_SIZENESW, IDC_SIZENS, IDC_SIZENWSE, IDC_SIZEWE,
        };

        unsafe {
            let cursors = [
                LoadCursorW(None, IDC_SIZENWSE)
                    .map(|c| c.0 as isize)
                    .unwrap_or(0),
                LoadCursorW(None, IDC_SIZENESW)
                    .map(|c| c.0 as isize)
                    .unwrap_or(0),
                LoadCursorW(None, IDC_SIZEWE)
                    .map(|c| c.0 as isize)
                    .unwrap_or(0),
                LoadCursorW(None, IDC_SIZENS)
                    .map(|c| c.0 as isize)
                    .unwrap_or(0),
            ];

            cursors.contains(&h_cursor)
        }
    }

    // ========== キャプチャ制御 ==========

    fn start_capture(&mut self, cursor_pos: &mut POINT) {
        if self.is_under_capture {
            return;
        }

        self.adjust_cursor_speed();

        // カーソルをソース領域に移動
        *cursor_pos = self.scaling_to_src(
            POINT {
                x: cursor_pos
                    .x
                    .clamp(self.dest_rect.left, self.dest_rect.right - 1),
                y: cursor_pos
                    .y
                    .clamp(self.dest_rect.top, self.dest_rect.bottom - 1),
            },
            RoundMethod::Round,
        );

        self.is_under_capture = true;
    }

    fn stop_capture(&mut self, cursor_pos: &mut POINT) -> bool {
        if !self.is_under_capture {
            return true;
        }

        let is_src_focused = unsafe { GetForegroundWindow() } == self.src_hwnd
            || unsafe { GetForegroundWindow() } == self.scaling_hwnd;
        let new_cursor_pos = self.src_to_scaling(*cursor_pos, is_src_focused);

        if unsafe { MonitorFromPoint(new_cursor_pos, MONITOR_DEFAULTTONULL) } == HMONITOR::default()
        {
            // 目標位置にモニターがない場合はソース領域に制限
            cursor_pos.x = cursor_pos
                .x
                .clamp(self.src_rect.left, self.src_rect.right - 1);
            cursor_pos.y = cursor_pos
                .y
                .clamp(self.src_rect.top, self.src_rect.bottom - 1);
            return false;
        }

        *cursor_pos = new_cursor_pos;
        self.restore_cursor_speed();
        self.is_under_capture = false;
        true
    }

    pub fn stop_capture_public(&mut self) {
        let mut pos = POINT::default();
        let _ = unsafe { GetCursorPos(&mut pos) };
        self.stop_capture(&mut pos);
        self.reliable_set_cursor_pos(pos);
        self.show_system_cursor(true);
        self.restore_clip_cursor();
    }

    // ========== ClipCursor ==========

    fn set_clip_cursor(&mut self, clip_rect: &RECT) {
        let mut cur_clip = RECT::default();
        unsafe {
            let _ = GetClipCursor(&mut cur_clip);
        }

        if cur_clip == self.last_real_clip && *clip_rect == self.last_clip {
            return;
        }

        unsafe {
            if ClipCursor(Some(clip_rect)).is_ok() {
                self.last_clip = *clip_rect;
                let _ = GetClipCursor(&mut self.last_real_clip);
            }
        }
    }

    fn restore_clip_cursor(&mut self) {
        if self.last_clip.left == i32::MAX {
            return;
        }

        let mut cur_clip = RECT::default();
        unsafe {
            let _ = GetClipCursor(&mut cur_clip);
        }

        if cur_clip != self.last_real_clip {
            return;
        }

        unsafe {
            if ClipCursor(None).is_ok() {
                self.last_clip = RECT {
                    left: i32::MAX,
                    top: i32::MAX,
                    right: i32::MAX,
                    bottom: i32::MAX,
                };
            }
        }
    }

    fn clip_cursor_for_monitors(&mut self, cursor_pos: POINT) {
        if !self.should_draw_cursor {
            self.restore_clip_cursor();
            return;
        }

        let is_src_focused = unsafe { GetForegroundWindow() } == self.src_hwnd
            || unsafe { GetForegroundWindow() } == self.scaling_hwnd;
        let scaled_pos = if self.is_under_capture {
            self.src_to_scaling(cursor_pos, true)
        } else {
            cursor_pos
        };

        let mut clips = RECT {
            left: i32::MIN,
            top: i32::MIN,
            right: i32::MAX,
            bottom: i32::MAX,
        };

        let monitor_rects = obtain_monitor_rects();
        if !monitor_rects.is_empty() {
            // 左
            let rect = RECT {
                left: i32::MIN,
                top: scaled_pos.y,
                right: self.renderer_rect.left,
                bottom: scaled_pos.y + 1,
            };
            if !any_intersected_monitor(&monitor_rects, &rect) {
                if self.is_under_capture {
                    clips.left = self.src_rect.left;
                } else if is_src_focused && self.dest_rect.left != self.renderer_rect.left {
                    clips.left = self.dest_rect.left;
                }
            }

            // 上
            let rect = RECT {
                left: scaled_pos.x,
                top: i32::MIN,
                right: scaled_pos.x + 1,
                bottom: self.renderer_rect.top,
            };
            if !any_intersected_monitor(&monitor_rects, &rect) {
                if self.is_under_capture {
                    clips.top = self.src_rect.top;
                } else if is_src_focused && self.dest_rect.top != self.renderer_rect.top {
                    clips.top = self.dest_rect.top;
                }
            }

            // 右
            let rect = RECT {
                left: self.renderer_rect.right,
                top: scaled_pos.y,
                right: i32::MAX,
                bottom: scaled_pos.y + 1,
            };
            if !any_intersected_monitor(&monitor_rects, &rect) {
                if self.is_under_capture {
                    clips.right = self.src_rect.right;
                } else if is_src_focused && self.dest_rect.right != self.renderer_rect.right {
                    clips.right = self.dest_rect.right;
                }
            }

            // 下
            let rect = RECT {
                left: scaled_pos.x,
                top: self.renderer_rect.bottom,
                right: scaled_pos.x + 1,
                bottom: i32::MAX,
            };
            if !any_intersected_monitor(&monitor_rects, &rect) {
                if self.is_under_capture {
                    clips.bottom = self.src_rect.bottom;
                } else if is_src_focused && self.dest_rect.bottom != self.renderer_rect.bottom {
                    clips.bottom = self.dest_rect.bottom;
                }
            }
        }

        if clips.left == i32::MIN
            && clips.top == i32::MIN
            && clips.right == i32::MAX
            && clips.bottom == i32::MAX
        {
            self.restore_clip_cursor();
        } else {
            self.set_clip_cursor(&clips);
        }
    }

    fn clip_cursor_on_src_moving(&mut self) {
        if self.local_cursor_pos_on_moving.x == i32::MAX {
            return;
        }

        let scaled_pos = POINT {
            x: self.local_cursor_pos_on_moving.x + self.renderer_rect.left,
            y: self.local_cursor_pos_on_moving.y + self.renderer_rect.top,
        };
        let origin_pos = self.scaling_to_src(scaled_pos, RoundMethod::Round);

        let mut clips = RECT {
            left: i32::MIN,
            top: i32::MIN,
            right: i32::MAX,
            bottom: i32::MAX,
        };

        let monitor_rects = obtain_monitor_rects();
        if !monitor_rects.is_empty() {
            // 単一モニターの場合はワークエリアに制限
            if monitor_rects.len() == 1 {
                let h_mon = unsafe { MonitorFromPoint(POINT::default(), MONITOR_DEFAULTTONULL) };
                if !h_mon.is_invalid() {
                    let mut mi = MONITORINFO {
                        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                        ..Default::default()
                    };
                    if unsafe { GetMonitorInfoW(h_mon, &mut mi).as_bool() } {
                        clips = mi.rcWork;
                    }
                }
            }

            // 各方向の制限
            if scaled_pos.x < origin_pos.x {
                let min_left = monitor_rects
                    .iter()
                    .filter(|r| r.top <= scaled_pos.y && r.bottom > scaled_pos.y)
                    .map(|r| r.left)
                    .min()
                    .unwrap_or(i32::MIN);
                if min_left != i32::MIN {
                    clips.left = min_left + (origin_pos.x - scaled_pos.x);
                }
            }

            if scaled_pos.y < origin_pos.y {
                let min_top = monitor_rects
                    .iter()
                    .filter(|r| r.left <= scaled_pos.x && r.right > scaled_pos.x)
                    .map(|r| r.top)
                    .min()
                    .unwrap_or(i32::MIN);
                if min_top != i32::MIN {
                    clips.top = min_top + (origin_pos.y - scaled_pos.y);
                }
            }

            if scaled_pos.x > origin_pos.x {
                let max_right = monitor_rects
                    .iter()
                    .filter(|r| r.top <= scaled_pos.y && r.bottom > scaled_pos.y)
                    .map(|r| r.right)
                    .max()
                    .unwrap_or(i32::MAX);
                if max_right != i32::MAX {
                    clips.right = max_right - (scaled_pos.x - origin_pos.x);
                }
            }

            if scaled_pos.y > origin_pos.y {
                let max_bottom = monitor_rects
                    .iter()
                    .filter(|r| r.left <= scaled_pos.x && r.right > scaled_pos.x)
                    .map(|r| r.bottom)
                    .max()
                    .unwrap_or(i32::MAX);
                if max_bottom != i32::MAX {
                    clips.bottom = max_bottom - (scaled_pos.y - origin_pos.y);
                }
            }
        }

        if clips.left == i32::MIN
            && clips.top == i32::MIN
            && clips.right == i32::MAX
            && clips.bottom == i32::MAX
        {
            self.restore_clip_cursor();
        } else {
            self.set_clip_cursor(&clips);
        }
    }

    // ========== ユーティリティ ==========

    fn set_ex_transparent(&self, transparent: bool, current_style: u32) {
        unsafe {
            let has_transparent = (current_style & WS_EX_TRANSPARENT.0) != 0;
            if transparent && !has_transparent {
                SetWindowLongPtrW(
                    self.scaling_hwnd,
                    GWL_EXSTYLE,
                    (current_style | WS_EX_TRANSPARENT.0) as isize,
                );
            } else if !transparent && has_transparent {
                SetWindowLongPtrW(
                    self.scaling_hwnd,
                    GWL_EXSTYLE,
                    (current_style & !WS_EX_TRANSPARENT.0) as isize,
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

    fn adjust_cursor_speed(&mut self) {
        let src_width = (self.src_rect.right - self.src_rect.left).max(1) as f64;
        let src_height = (self.src_rect.bottom - self.src_rect.top).max(1) as f64;
        let dest_width = (self.dest_rect.right - self.dest_rect.left).max(1) as f64;
        let dest_height = (self.dest_rect.bottom - self.dest_rect.top).max(1) as f64;

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
            self.origin_cursor_speed = current_speed;
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
        if self.origin_cursor_speed == 0 {
            return;
        }
        unsafe {
            let _ = SystemParametersInfoW(
                SPI_SETMOUSESPEED,
                0,
                Some(self.origin_cursor_speed as *mut _),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
            );
        }
        self.origin_cursor_speed = 0;
    }

    fn reliable_set_cursor_pos(&self, pos: POINT) {
        unsafe {
            use windows::Win32::UI::WindowsAndMessaging::SetCursorPos;

            let mut origin_clip = RECT::default();
            let _ = GetClipCursor(&mut origin_clip);

            // 一時的にカーソル位置を制限して移動（Magpie方式）
            let temp_clip = RECT {
                left: pos.x,
                top: pos.y,
                right: pos.x + 1,
                bottom: pos.y + 1,
            };
            let _ = ClipCursor(Some(&temp_clip));

            // SetCursorPosも呼び出して確実に移動
            let _ = SetCursorPos(pos.x, pos.y);

            // OSが入力キューを処理するまで待機
            Sleep(8);

            // クリップを復元
            let _ = ClipCursor(Some(&origin_clip));
        }
    }

    fn clear_hit_test_result(&mut self) {
        self.last_completed_hit_test_id = self.next_hit_test_id;
        self.next_hit_test_id += 1;
        self.last_completed_hit_test_pos.x = i32::MAX;
        self.last_completed_hit_test_result = 0; // HTNOWHERE
    }

    /// スケーリングWindowの位置変更時に呼び出す
    pub fn on_scaling_pos_changed(&mut self) {
        if self.is_under_capture {
            let new_pos = self.scaling_to_src(self.cursor_pos, RoundMethod::Round);
            self.reliable_set_cursor_pos(new_pos);
        }
        self.clear_hit_test_result();
    }

    /// ソースウィンドウの移動開始時
    pub fn on_src_start_move(&mut self) {
        if !self.is_under_capture {
            return;
        }

        if self.local_cursor_pos_on_moving.x == i32::MAX {
            self.local_cursor_pos_on_moving.x = self.cursor_pos.x - self.renderer_rect.left;
            self.local_cursor_pos_on_moving.y = self.cursor_pos.y - self.renderer_rect.top;
        }

        self.restore_cursor_speed();
    }

    /// ソースウィンドウの移動終了時
    pub fn on_src_end_move(&mut self) {
        if !self.is_under_capture {
            return;
        }

        self.local_cursor_pos_on_moving.x = i32::MAX;
        self.adjust_cursor_speed();
    }

    /// スケーリングウィンドウの移動開始時
    pub fn on_start_move(&mut self) {
        if self.is_under_capture {
            return;
        }

        self.local_cursor_pos_on_moving.x = self.cursor_pos.x - self.renderer_rect.left;
        self.local_cursor_pos_on_moving.y = self.cursor_pos.y - self.renderer_rect.top;
    }

    /// リサイズ/移動終了時
    pub fn on_end_resize_move(&mut self) {
        self.local_cursor_pos_on_moving.x = i32::MAX;
    }

    /// ソース矩形変更時
    pub fn on_src_rect_changed(&mut self) {
        self.clear_hit_test_result();
    }
}

impl Drop for CursorManager {
    fn drop(&mut self) {
        self.show_system_cursor(true);
        self.restore_cursor_speed();
        self.restore_clip_cursor();
    }
}
