use anyhow::Result;
use windows::Win32::Graphics::Direct2D::Common::{D2D1_COLOR_F, *};
use windows::Win32::Graphics::Direct2D::{
    D2D1CreateFactory, ID2D1Factory, ID2D1RenderTarget, ID2D1SolidColorBrush,
    D2D1_DRAW_TEXT_OPTIONS_NONE, D2D1_ELLIPSE, D2D1_FACTORY_OPTIONS,
    D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_FEATURE_LEVEL_DEFAULT, D2D1_RENDER_TARGET_PROPERTIES,
    D2D1_RENDER_TARGET_TYPE_DEFAULT, D2D1_RENDER_TARGET_USAGE_NONE, D2D1_ROUNDED_RECT,
};
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, IDWriteTextFormat, DWRITE_FACTORY_TYPE_SHARED,
    DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_NORMAL,
    DWRITE_MEASURING_MODE_NATURAL, DWRITE_PARAGRAPH_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT_CENTER,
};
use windows::Win32::Graphics::Dxgi::IDXGISurface;

pub struct Toolbar {
    d2d_factory: ID2D1Factory,
    dw_factory: IDWriteFactory,
    text_format: Option<IDWriteTextFormat>,

    // State
    pub fps: u32,
    pub processing_time_ms: f32,
    pub active_algorithm: String,

    // Layout
    toolbar_rect: D2D_RECT_F,
}

impl Toolbar {
    pub fn new() -> Result<Self> {
        let options = D2D1_FACTORY_OPTIONS::default();
        unsafe {
            let d2d_factory: ID2D1Factory =
                D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, Some(&options))?;
            let dw_factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;

            // Create Text Format
            let font_name = windows::core::w!("Segoe UI");
            let format = dw_factory.CreateTextFormat(
                font_name,
                None,
                DWRITE_FONT_WEIGHT_NORMAL,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                14.0,
                windows::core::w!("en-us"),
            )?;

            format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER)?;
            format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER)?;

            Ok(Self {
                d2d_factory,
                dw_factory,
                text_format: Some(format),
                fps: 0,
                processing_time_ms: 0.0,
                active_algorithm: "Unknown".to_string(),
                toolbar_rect: D2D_RECT_F {
                    left: 0.0,
                    top: 0.0,
                    right: 0.0,
                    bottom: 0.0,
                },
            })
        }
    }

    /// 毎フレーム呼び出し: バックバッファに直接描画
    pub fn render_with_cursor(
        &mut self,
        dxgi_surface: &IDXGISurface,
        width: u32,
        _height: u32,
        cursor_pos: Option<(f32, f32)>,
    ) -> Result<()> {
        unsafe {
            // 毎フレーム新しいレンダーターゲットを作成 (Flip SwapChain対応)
            let props = D2D1_RENDER_TARGET_PROPERTIES {
                r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_UNKNOWN,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: 96.0,
                dpiY: 96.0,
                usage: D2D1_RENDER_TARGET_USAGE_NONE,
                minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
            };

            let target = self
                .d2d_factory
                .CreateDxgiSurfaceRenderTarget(dxgi_surface, &props)?;

            // ブラシを作成
            let white_brush = target.CreateSolidColorBrush(
                &D2D1_COLOR_F {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                },
                None,
            )?;

            let bg_brush = target.CreateSolidColorBrush(
                &D2D1_COLOR_F {
                    r: 0.06,
                    g: 0.06,
                    b: 0.06,
                    a: 0.8,
                },
                None,
            )?;

            target.BeginDraw();

            // カーソルを描画 (ツールバーの前に描画)
            if let Some((x, y)) = cursor_pos {
                let point = D2D_POINT_2F { x, y };
                let ellipse = D2D1_ELLIPSE {
                    point,
                    radiusX: 5.0,
                    radiusY: 5.0,
                };
                target.FillEllipse(&ellipse, &white_brush);
            }

            // ツールバー描画
            let toolbar_width = 400.0;
            let toolbar_height = 36.0;
            let x_pos = (width as f32 - toolbar_width) / 2.0;
            let y_pos = 0.0;

            let rect = D2D_RECT_F {
                left: x_pos,
                top: y_pos,
                right: x_pos + toolbar_width,
                bottom: y_pos + toolbar_height,
            };
            self.toolbar_rect = rect;

            let rounded = D2D1_ROUNDED_RECT {
                rect,
                radiusX: 6.0,
                radiusY: 6.0,
            };

            target.FillRoundedRectangle(&rounded, &bg_brush);

            // テキスト描画
            let text = format!(
                "{} | FPS: {} | Time: {:.2}ms",
                self.active_algorithm, self.fps, self.processing_time_ms
            );
            let text_wide: Vec<u16> = text.encode_utf16().collect();

            if let Some(fmt) = &self.text_format {
                target.DrawText(
                    &text_wide,
                    fmt,
                    &rect,
                    &white_brush,
                    D2D1_DRAW_TEXT_OPTIONS_NONE,
                    DWRITE_MEASURING_MODE_NATURAL,
                );
            }

            target.EndDraw(None, None)?;
        }
        Ok(())
    }
}
