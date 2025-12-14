use std::mem::size_of;
use windows::Win32::Foundation::{BOOL, HWND, POINT, RECT};
use windows::Win32::Graphics::Dwm::{
    DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS, DWMWA_NCRENDERING_ENABLED,
};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetForegroundWindow, GetWindowLongPtrW, GetWindowRect, GWL_STYLE, WS_CAPTION,
    WS_THICKFRAME,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SrcWindowKind {
    Native,
    NoTitleBar,
    NoBorder,
    NoDecoration,
    OnlyThickFrame,
}

pub struct SrcTracker {
    hwnd: HWND,
    window_kind: SrcWindowKind,
    window_rect: RECT,
    window_frame_rect: RECT,
    client_rect: RECT, // In Screen Coordinates
}

impl SrcTracker {
    pub fn new(hwnd: HWND) -> Self {
        Self {
            hwnd,
            window_kind: SrcWindowKind::Native,
            window_rect: RECT::default(),
            window_frame_rect: RECT::default(),
            client_rect: RECT::default(),
        }
    }

    pub fn update(&mut self) -> anyhow::Result<()> {
        unsafe {
            // 1. Get Window Rect
            GetWindowRect(self.hwnd, &mut self.window_rect)?;

            // 2. Get Extended Frame Bounds (Shadow included usually, or excluded? DWM Frame)
            // Magpie uses DWMWA_EXTENDED_FRAME_BOUNDS
            let mut frame_rect = RECT::default();
            let hr = DwmGetWindowAttribute(
                self.hwnd,
                DWMWA_EXTENDED_FRAME_BOUNDS,
                &mut frame_rect as *mut _ as *mut _,
                size_of::<RECT>() as u32,
            );
            if hr.is_ok() {
                self.window_frame_rect = frame_rect;
            } else {
                // Fallback to WindowRect if DWM fails
                self.window_frame_rect = self.window_rect;
            }

            // 3. Get Client Rect in Screen Coordinates
            let mut client_rect = RECT::default();
            GetClientRect(self.hwnd, &mut client_rect)?;
            let mut pt = POINT { x: 0, y: 0 };
            ClientToScreen(self.hwnd, &mut pt); // Convert (0,0) to screen
                                                // Reconstruct client rect in screen coords
            let width = client_rect.right - client_rect.left;
            let height = client_rect.bottom - client_rect.top;
            self.client_rect = RECT {
                left: pt.x,
                top: pt.y,
                right: pt.x + width,
                bottom: pt.y + height,
            };

            // 4. Determine Window Kind
            let mut has_border = BOOL::from(true);
            let hr = DwmGetWindowAttribute(
                self.hwnd,
                DWMWA_NCRENDERING_ENABLED,
                &mut has_border as *mut _ as *mut _,
                size_of::<BOOL>() as u32,
            );

            if hr.is_err() || !has_border.as_bool() {
                self.window_kind = SrcWindowKind::NoDecoration;
            } else {
                // Native check logic from Magpie
                // "if (_windowRect.top == clientRect.top)"
                if self.window_rect.top == self.client_rect.top {
                    if self.window_rect.left != self.client_rect.left
                        && self.window_rect.right != self.client_rect.right
                        && self.window_rect.bottom != self.client_rect.bottom
                    {
                        // Win10 fake top border or Win11 client area expansion
                        self.window_kind = SrcWindowKind::NoTitleBar;
                    } else {
                        // At least one side matches, so likely NoBorder
                        self.window_kind = SrcWindowKind::NoBorder;
                    }
                } else {
                    // Check Styles
                    let style = GetWindowLongPtrW(self.hwnd, GWL_STYLE) as u32;
                    if (style & WS_CAPTION.0) == 0 && (style & WS_THICKFRAME.0) != 0 {
                        self.window_kind = SrcWindowKind::OnlyThickFrame;
                    } else {
                        self.window_kind = SrcWindowKind::Native;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn get_capture_rect(&self) -> RECT {
        // Based on Magpie's _CalcSrcRect
        // For simplicity, we assume we want to capture the "Client" area logic mostly,
        // but adjusted for the specifics of the WindowKind.

        match self.window_kind {
            SrcWindowKind::NoDecoration => {
                // Return full window rect (which is likely same as client rect for these)
                // Magpie says: "_srcRect = _windowRect;"
                // Note: Magpie's _windowRect is GetWindowRect.
                self.window_rect
            }
            SrcWindowKind::NoTitleBar => {
                // Magpie logic:
                // if NoTitleBar && !CaptureTitleBar && GetClientRectOfUWP... (Skip UWP complexity for now or treat as standard)
                // Magpie uses _windowFrameRect as base.
                // "srcRect.left = _windowFrameRect.left + borderThickness..."
                // Wait, for NoTitleBar, Magpie usually still crops frame.

                // Ideally we just want the Client Rect for content, but if it has a border, we might want to include/exclude it.
                // Re-reading Magpie C++:
                // "else { _srcRect.left = _windowFrameRect.left + borderThickness ... }"

                // If we want "Pseudo-Borderless", we generally want the Client Area visible,
                // but we capture the whole Frame and then crop.
                // However, Windows Graphics Capture captures the Visual (roughly FrameBounds).

                // If we return the rect relative to the screen, the caller will calculate UVs.
                // Let's return the "ROI" (Region of Interest) in Screen Coordinates.

                // For Native/NoTitleBar/OnlyThickFrame, we usually want the Client Rect.
                // But Magpie does Frame + BorderAdjustment.
                // Let's stick to using ClientRect as the safest "No Border" representation for now,
                // effectively behaving like "Crop to Client".

                self.client_rect
            }
            SrcWindowKind::Native => {
                // Crop to Client
                self.client_rect
            }
            SrcWindowKind::OnlyThickFrame => {
                // Crop to Client
                self.client_rect
            }
            SrcWindowKind::NoBorder => {
                // Typically client rect is what we want.
                self.client_rect
            }
        }
    }

    pub fn get_window_kind(&self) -> SrcWindowKind {
        self.window_kind
    }

    pub fn get_window_rect(&self) -> RECT {
        self.window_rect
    }

    pub fn get_client_rect(&self) -> RECT {
        self.client_rect
    }

    pub fn get_frame_rect(&self) -> RECT {
        self.window_frame_rect
    }

    /// ソースウィンドウがフォアグラウンド(アクティブ)かどうかを返す
    /// Magpieの SrcTracker::IsFocused() に相当
    pub fn is_focused(&self) -> bool {
        unsafe { GetForegroundWindow() == self.hwnd }
    }

    /// ソースウィンドウのハンドルを返す
    pub fn handle(&self) -> HWND {
        self.hwnd
    }
}
