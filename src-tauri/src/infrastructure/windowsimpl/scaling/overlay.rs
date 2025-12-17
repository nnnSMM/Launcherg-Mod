// D3D11ベースのシンプルなツールバー
// カーソルが画面上端に近づくと表示され、FPSと終了ボタンを含む
// DirectWriteを使用した高品質テキスト描画

use anyhow::{anyhow, Result};
use std::time::Instant;
use windows::core::{ComInterface, PCSTR, PCWSTR};
use windows::Win32::Foundation::POINT;
use windows::Win32::Graphics::Direct2D::Common::{
    D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_PIXEL_FORMAT,
};
use windows::Win32::Graphics::Direct2D::{
    D2D1CreateFactory, ID2D1Factory, ID2D1RenderTarget, ID2D1SolidColorBrush,
    D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_RENDER_TARGET_PROPERTIES,
};
use windows::Win32::Graphics::Direct3D::Fxc::{D3DCompile, D3DCOMPILE_OPTIMIZATION_LEVEL3};
use windows::Win32::Graphics::Direct3D::D3D11_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP;
use windows::Win32::Graphics::Direct3D11::{
    ID3D11BlendState, ID3D11Buffer, ID3D11Device, ID3D11DeviceContext, ID3D11InputLayout,
    ID3D11PixelShader, ID3D11Texture2D, ID3D11VertexShader, D3D11_BIND_VERTEX_BUFFER,
    D3D11_BLEND_DESC, D3D11_BLEND_INV_SRC_ALPHA, D3D11_BLEND_ONE, D3D11_BLEND_OP_ADD,
    D3D11_BUFFER_DESC, D3D11_COLOR_WRITE_ENABLE_ALL, D3D11_CPU_ACCESS_WRITE,
    D3D11_INPUT_ELEMENT_DESC, D3D11_INPUT_PER_VERTEX_DATA, D3D11_MAPPED_SUBRESOURCE,
    D3D11_MAP_WRITE_DISCARD, D3D11_RENDER_TARGET_BLEND_DESC, D3D11_TEXTURE2D_DESC,
    D3D11_USAGE_DYNAMIC, D3D11_VIEWPORT,
};
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, IDWriteTextFormat, DWRITE_FACTORY_TYPE_SHARED,
    DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_NORMAL,
    DWRITE_PARAGRAPH_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT_CENTER,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R32G32B32A32_FLOAT;
use windows::Win32::Graphics::Dxgi::IDXGISurface;

// 頂点シェーダー
// 頂点シェーダー (Rounded Rect Support)
const VS_SOURCE: &str = r#"
struct VS_INPUT {
    float2 pos : POSITION;
    float2 uv : TEXCOORD; // -1..1 relative to rect center
    float4 color : COLOR;
    float2 size : SIZE;   // width, height (pixels)
    float radius : RADIUS; // corner radius (pixels)
    uint type : TYPE;     // 0=Flat Color, 1=Rounded Rect Background
};
struct VS_OUTPUT {
    float4 pos : SV_POSITION;
    float2 uv : TEXCOORD;
    float4 color : COLOR;
    float2 size : SIZE;
    float radius : RADIUS;
    nointerpolation uint type : TYPE;
};

VS_OUTPUT main(VS_INPUT input) {
    VS_OUTPUT output;
    output.pos = float4(input.pos, 0.0, 1.0);
    output.uv = input.uv; 
    output.color = input.color;
    output.size = input.size;
    output.radius = input.radius;
    output.type = input.type;
    return output;
}
"#;

// ピクセルシェーダー (Rounded Rect SDF)
const PS_SOURCE: &str = r#"
struct PS_INPUT {
    float4 pos : SV_POSITION;
    float2 uv : TEXCOORD;
    float4 color : COLOR;
    float2 size : SIZE;
    float radius : RADIUS;
    nointerpolation uint type : TYPE;
};

float rounded_box_sdf(float2 p, float2 b, float r) {
    float2 q = abs(p) - b + r;
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - r;
}

// Signed distance to a line segment
float line_sdf(float2 p, float2 a, float2 b) {
    float2 pa = p - a;
    float2 ba = b - a;
    float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
}

float4 main(PS_INPUT input) : SV_TARGET {
    if (input.type == 1) {
        // Rounded Rect with white border
        float2 p = input.uv * (input.size / 2.0);
        float dist = rounded_box_sdf(p, input.size / 2.0, input.radius);
        
        float alpha = 1.0 - smoothstep(-0.5, 0.5, dist);
        
        // Border: detect if we're near the edge
        float border_width = 0.5; // thinner border
        float border_alpha = 1.0 - smoothstep(border_width - 0.5, border_width + 0.5, abs(dist + border_width / 2.0));
        
        // Mix background color with white border
        float3 border_color = float3(0.38, 0.4, 0.43); // 薄い灰青のボーダー
        float3 final_color = lerp(input.color.rgb, border_color, border_alpha);
        
        float finalAlpha = input.color.a * alpha;
        return float4(final_color * finalAlpha, finalAlpha);
    }
    else if (input.type == 2) {
        // X Icon - two diagonal lines
        // UV is -1..1, size is pixel dimensions
        float2 p = input.uv * (input.size / 2.0);
        float half_s = input.size.x / 2.0 - input.radius; // radius used as padding here
        
        // Line 1: top-left to bottom-right
        float d1 = line_sdf(p, float2(-half_s, -half_s), float2(half_s, half_s));
        // Line 2: top-right to bottom-left
        float d2 = line_sdf(p, float2(half_s, -half_s), float2(-half_s, half_s));
        
        float dist = min(d1, d2);
        float thickness = 0.6; // very thin line for X icon
        float alpha = 1.0 - smoothstep(thickness - 0.15, thickness + 0.15, dist);
        
        float finalAlpha = input.color.a * alpha;
        return float4(input.color.rgb * finalAlpha, finalAlpha);
    }
    // Flat color - also pre-multiply
    return float4(input.color.rgb * input.color.a, input.color.a);
}
"#;

// 頂点データ構造体 (Updated)
#[repr(C)]
struct ColorVertex {
    pos: [f32; 2],
    uv: [f32; 2], // New
    color: [f32; 4],
    size: [f32; 2], // New
    radius: f32,    // New
    type_: u32,     // New
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToolbarAction {
    None,
    Stop,
    Close,
}

/// シンプルなD3D11ツールバー (DirectWrite対応)
pub struct SimpleToolbar {
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    vertex_shader: Option<ID3D11VertexShader>,
    pixel_shader: Option<ID3D11PixelShader>,
    input_layout: Option<ID3D11InputLayout>,
    vertex_buffer: Option<ID3D11Buffer>,
    blend_state: Option<ID3D11BlendState>,
    initialized: bool,

    // DirectWrite/D2D1
    d2d_factory: Option<ID2D1Factory>,
    dwrite_factory: Option<IDWriteFactory>,
    text_format: Option<IDWriteTextFormat>,

    // FPS計測
    fps_update_time: Instant,
    frame_count: u32,
    current_fps: f32,

    // 表示状態
    is_visible: bool,
    visibility_timer: Instant,

    // ボタン領域 (ピクセル座標)
    pub close_button_rect: (f32, f32, f32, f32), // (left, top, right, bottom)
    pub stop_button_rect: (f32, f32, f32, f32),

    // State
    pub hovered_button: ToolbarAction,
}

impl SimpleToolbar {
    pub fn new(device: ID3D11Device, context: ID3D11DeviceContext) -> Self {
        let now = Instant::now();
        Self {
            device,
            context,
            vertex_shader: None,
            pixel_shader: None,
            input_layout: None,
            vertex_buffer: None,
            blend_state: None,
            initialized: false,
            d2d_factory: None,
            dwrite_factory: None,
            text_format: None,
            fps_update_time: now,
            frame_count: 0,
            current_fps: 0.0,
            is_visible: false,
            visibility_timer: now,
            close_button_rect: (0.0, 0.0, 0.0, 0.0),
            stop_button_rect: (0.0, 0.0, 0.0, 0.0),
            hovered_button: ToolbarAction::None,
        }
    }

    /// シェーダーとリソースを初期化
    pub fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        unsafe {
            // 頂点シェーダーをコンパイル
            let mut vs_blob = None;
            let mut _errors = None;
            D3DCompile(
                VS_SOURCE.as_ptr() as *const _,
                VS_SOURCE.len(),
                PCSTR::null(),
                None,
                None,
                PCSTR(b"main\0".as_ptr()),
                PCSTR(b"vs_4_0\0".as_ptr()),
                D3DCOMPILE_OPTIMIZATION_LEVEL3,
                0,
                &mut vs_blob,
                Some(&mut _errors),
            )?;
            let vs_blob = vs_blob.ok_or_else(|| anyhow!("Failed to compile VS"))?;

            let vs_data = std::slice::from_raw_parts(
                vs_blob.GetBufferPointer() as *const u8,
                vs_blob.GetBufferSize(),
            );
            let mut vertex_shader = None;
            self.device
                .CreateVertexShader(vs_data, None, Some(&mut vertex_shader))?;
            self.vertex_shader = vertex_shader;

            // 入力レイアウト (Updated for new VS_INPUT)
            let input_elements = [
                D3D11_INPUT_ELEMENT_DESC {
                    SemanticName: PCSTR(b"POSITION\0".as_ptr()),
                    SemanticIndex: 0,
                    Format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R32G32_FLOAT,
                    InputSlot: 0,
                    AlignedByteOffset: 0,
                    InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                    InstanceDataStepRate: 0,
                },
                D3D11_INPUT_ELEMENT_DESC {
                    SemanticName: PCSTR(b"TEXCOORD\0".as_ptr()), // UV
                    SemanticIndex: 0,
                    Format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R32G32_FLOAT,
                    InputSlot: 0,
                    AlignedByteOffset: 8,
                    InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                    InstanceDataStepRate: 0,
                },
                D3D11_INPUT_ELEMENT_DESC {
                    SemanticName: PCSTR(b"COLOR\0".as_ptr()),
                    SemanticIndex: 0,
                    Format: DXGI_FORMAT_R32G32B32A32_FLOAT,
                    InputSlot: 0,
                    AlignedByteOffset: 16,
                    InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                    InstanceDataStepRate: 0,
                },
                D3D11_INPUT_ELEMENT_DESC {
                    SemanticName: PCSTR(b"SIZE\0".as_ptr()),
                    SemanticIndex: 0,
                    Format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R32G32_FLOAT,
                    InputSlot: 0,
                    AlignedByteOffset: 32,
                    InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                    InstanceDataStepRate: 0,
                },
                D3D11_INPUT_ELEMENT_DESC {
                    SemanticName: PCSTR(b"RADIUS\0".as_ptr()),
                    SemanticIndex: 0,
                    Format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R32_FLOAT,
                    InputSlot: 0,
                    AlignedByteOffset: 40,
                    InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                    InstanceDataStepRate: 0,
                },
                D3D11_INPUT_ELEMENT_DESC {
                    SemanticName: PCSTR(b"TYPE\0".as_ptr()),
                    SemanticIndex: 0,
                    Format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R32_UINT,
                    InputSlot: 0,
                    AlignedByteOffset: 44,
                    InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                    InstanceDataStepRate: 0,
                },
            ];
            let mut input_layout = None;
            self.device
                .CreateInputLayout(&input_elements, vs_data, Some(&mut input_layout))?;
            self.input_layout = input_layout;

            // ピクセルシェーダーをコンパイル
            let mut ps_blob = None;
            D3DCompile(
                PS_SOURCE.as_ptr() as *const _,
                PS_SOURCE.len(),
                PCSTR::null(),
                None,
                None,
                PCSTR(b"main\0".as_ptr()),
                PCSTR(b"ps_4_0\0".as_ptr()),
                D3DCOMPILE_OPTIMIZATION_LEVEL3,
                0,
                &mut ps_blob,
                None,
            )?;
            let ps_blob = ps_blob.ok_or_else(|| anyhow!("Failed to compile PS"))?;

            let ps_data = std::slice::from_raw_parts(
                ps_blob.GetBufferPointer() as *const u8,
                ps_blob.GetBufferSize(),
            );
            let mut pixel_shader = None;
            self.device
                .CreatePixelShader(ps_data, None, Some(&mut pixel_shader))?;
            self.pixel_shader = pixel_shader;

            // 頂点バッファ (動的、矩形複数個分 - FPS数字用に増加)
            let buffer_desc = D3D11_BUFFER_DESC {
                ByteWidth: (std::mem::size_of::<ColorVertex>() * 200) as u32, // 多数の矩形用
                Usage: D3D11_USAGE_DYNAMIC,
                BindFlags: D3D11_BIND_VERTEX_BUFFER.0 as u32,
                CPUAccessFlags: D3D11_CPU_ACCESS_WRITE.0 as u32,
                ..Default::default()
            };
            let mut vertex_buffer = None;
            self.device
                .CreateBuffer(&buffer_desc, None, Some(&mut vertex_buffer))?;
            self.vertex_buffer = vertex_buffer;

            // ブレンド状態
            let blend_desc = D3D11_BLEND_DESC {
                RenderTarget: [
                    D3D11_RENDER_TARGET_BLEND_DESC {
                        BlendEnable: true.into(),
                        SrcBlend: D3D11_BLEND_ONE,
                        DestBlend: D3D11_BLEND_INV_SRC_ALPHA,
                        BlendOp: D3D11_BLEND_OP_ADD,
                        SrcBlendAlpha: D3D11_BLEND_ONE,
                        DestBlendAlpha: D3D11_BLEND_INV_SRC_ALPHA,
                        BlendOpAlpha: D3D11_BLEND_OP_ADD,
                        RenderTargetWriteMask: D3D11_COLOR_WRITE_ENABLE_ALL.0 as u8,
                    },
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    Default::default(),
                ],
                ..Default::default()
            };
            let mut blend_state = None;
            self.device
                .CreateBlendState(&blend_desc, Some(&mut blend_state))?;
            self.blend_state = blend_state;

            // DirectWrite と Direct2D の初期化
            let d2d_factory: ID2D1Factory =
                D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;
            self.d2d_factory = Some(d2d_factory);

            let dwrite_factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;

            // テキストフォーマット作成 (Segoe UI, 14pt)
            let font_name: Vec<u16> = "Segoe UI\0".encode_utf16().collect();
            let locale: Vec<u16> = "en-us\0".encode_utf16().collect();
            let text_format = dwrite_factory.CreateTextFormat(
                PCWSTR(font_name.as_ptr()),
                None,
                DWRITE_FONT_WEIGHT_NORMAL,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                14.0, // フォントサイズを小さく
                PCWSTR(locale.as_ptr()),
            )?;

            // テキストを中央揃えに
            text_format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER)?;
            text_format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER)?;

            self.text_format = Some(text_format);
            self.dwrite_factory = Some(dwrite_factory);
        }

        self.initialized = true;
        Ok(())
    }

    /// FPSを更新
    pub fn tick(&mut self) {
        self.frame_count += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.fps_update_time).as_secs_f32();
        if elapsed >= 1.0 {
            self.current_fps = self.frame_count as f32 / elapsed;
            self.frame_count = 0;
            self.fps_update_time = now;
        }
    }

    /// カーソル位置に基づいて表示/非表示を更新
    /// コンテンツ領域の上端50ピクセル以内でカーソルがあれば表示
    pub fn update_visibility(
        &mut self,
        cursor_pos: POINT,
        content_rect: &windows::Win32::Foundation::RECT,
    ) {
        let rel_y = cursor_pos.y - content_rect.top;
        let near_top = rel_y >= -10 && rel_y < 50;
        self.is_visible = near_top;

        // ホバー状態をチェック
        if self.is_visible {
            let cx = cursor_pos.x as f32;
            let cy = cursor_pos.y as f32;
            let (cl, ct, cr, cb) = self.close_button_rect;
            if cx >= cl && cx <= cr && cy >= ct && cy <= cb {
                self.hovered_button = ToolbarAction::Close;
            } else {
                self.hovered_button = ToolbarAction::None;
            }
        } else {
            self.hovered_button = ToolbarAction::None;
        }
    }

    /// 終了ボタンがクリックされたかチェック (ピクセル座標で)
    pub fn check_close_button_click(&self, cursor_pos: POINT) -> bool {
        if !self.is_visible {
            return false;
        }

        let (left, top, right, bottom) = self.close_button_rect;
        cursor_pos.x as f32 >= left
            && cursor_pos.x as f32 <= right
            && cursor_pos.y as f32 >= top
            && cursor_pos.y as f32 <= bottom
    }

    /// ツールバーを描画
    pub fn render(
        &mut self,
        backbuffer: &ID3D11Texture2D,
        content_rect: &windows::Win32::Foundation::RECT,
    ) -> Result<()> {
        if !self.is_visible {
            return Ok(());
        }

        if !self.initialized {
            self.initialize()?;
        }

        unsafe {
            // バックバッファのサイズを取得 (画面全体)
            let mut bb_desc = D3D11_TEXTURE2D_DESC::default();
            backbuffer.GetDesc(&mut bb_desc);
            let width = bb_desc.Width as f32;
            let height = bb_desc.Height as f32;

            // コンテンツ領域の寸法
            let content_x = content_rect.left as f32;
            let content_y = content_rect.top as f32;
            let content_w = (content_rect.right - content_rect.left) as f32;

            // ツールバーの寸法 (ピクセル)
            // Magpieっぽく少し丸みを帯びたデザインにするために上部を少しはみ出させる場合の調整はここで行う
            let toolbar_width = 360.0f32; // Magpie reference uses 360 * scale
                                          // TODO: DPI scale support if needed, currently fixed

            let toolbar_height = 40.0f32;

            // コンテンツ領域の中央に配置
            let toolbar_x = content_x + (content_w - toolbar_width) / 2.0;

            // コンテンツ領域の上端に配置
            // Magpie-style: 上部を少し画面外に出して、上の角丸を隠す
            let corner_rounding = 6.0f32;
            let toolbar_y = content_y - corner_rounding;

            // 終了ボタンの寸法
            let button_size = 22.0f32; // 小さく
                                       // 右寄せ: toolbar_x + toolbar_width - button_size - margin
            let button_x = toolbar_x + toolbar_width - button_size - 10.0;
            // Y位置は角丸オフセット分下げる（可視部分の中央に配置）
            let button_y = toolbar_y
                + corner_rounding
                + (toolbar_height - corner_rounding - button_size) / 2.0;

            // ボタン領域を保存
            self.close_button_rect = (
                button_x,
                button_y,
                button_x + button_size,
                button_y + button_size,
            );

            // NDC座標に変換する関数
            let to_ndc_x = |px: f32| -> f32 { px / width * 2.0 - 1.0 };
            let to_ndc_y = |py: f32| -> f32 { 1.0 - py / height * 2.0 };

            // ツールバー背景の頂点 (Launcherg UIカラー #2d333b)
            let bg_color = [0.176, 0.2, 0.231, 0.95];
            let bg_left = to_ndc_x(toolbar_x);
            let bg_right = to_ndc_x(toolbar_x + toolbar_width);
            let bg_top = to_ndc_y(toolbar_y);
            let bg_bottom = to_ndc_y(toolbar_y + toolbar_height);

            // 終了ボタンの頂点
            let btn_color = if self.hovered_button == ToolbarAction::Close {
                [0.8, 0.2, 0.2, 1.0] // ホバー時: 赤
            } else {
                [0.0, 0.0, 0.0, 0.0] // 通常時: 透明
            };
            let btn_left = to_ndc_x(button_x);
            let btn_right = to_ndc_x(button_x + button_size);
            let btn_top = to_ndc_y(button_y);
            let btn_bottom = to_ndc_y(button_y + button_size);

            // 頂点データを収集するVec
            let mut vertices: Vec<ColorVertex> = Vec::new();

            // 背景矩形 (rounded rect, type_=1)
            let bg_size = [toolbar_width, toolbar_height];
            vertices.push(ColorVertex {
                pos: [bg_left, bg_top],
                uv: [-1.0, 1.0],
                color: bg_color,
                size: bg_size,
                radius: 6.0,
                type_: 1,
            });
            vertices.push(ColorVertex {
                pos: [bg_right, bg_top],
                uv: [1.0, 1.0],
                color: bg_color,
                size: bg_size,
                radius: 6.0,
                type_: 1,
            });
            vertices.push(ColorVertex {
                pos: [bg_left, bg_bottom],
                uv: [-1.0, -1.0],
                color: bg_color,
                size: bg_size,
                radius: 6.0,
                type_: 1,
            });
            vertices.push(ColorVertex {
                pos: [bg_right, bg_bottom],
                uv: [1.0, -1.0],
                color: bg_color,
                size: bg_size,
                radius: 6.0,
                type_: 1,
            });

            // 終了ボタン矩形 (type_=0 for flat color or type_=1 with small radius)
            let btn_sz = [button_size, button_size];
            vertices.push(ColorVertex {
                pos: [btn_left, btn_top],
                uv: [-1.0, 1.0],
                color: btn_color,
                size: btn_sz,
                radius: 4.0,
                type_: 1,
            });
            vertices.push(ColorVertex {
                pos: [btn_right, btn_top],
                uv: [1.0, 1.0],
                color: btn_color,
                size: btn_sz,
                radius: 4.0,
                type_: 1,
            });
            vertices.push(ColorVertex {
                pos: [btn_left, btn_bottom],
                uv: [-1.0, -1.0],
                color: btn_color,
                size: btn_sz,
                radius: 4.0,
                type_: 1,
            });
            vertices.push(ColorVertex {
                pos: [btn_right, btn_bottom],
                uv: [1.0, -1.0],
                color: btn_color,
                size: btn_sz,
                radius: 4.0,
                type_: 1,
            });

            // 終了ボタンにXアイコンを追加（斜め線2本、type_=2）
            let x_color = [1.0, 1.0, 1.0, 1.0]; // 白
            let x_padding = 5.0f32;
            // X アイコン用の矩形（シェーダーで対角線を描画）
            let x_left = to_ndc_x(button_x);
            let x_right = to_ndc_x(button_x + button_size);
            let x_top = to_ndc_y(button_y);
            let x_bottom = to_ndc_y(button_y + button_size);
            let x_sz = [button_size, button_size];

            vertices.push(ColorVertex {
                pos: [x_left, x_top],
                uv: [-1.0, 1.0],
                color: x_color,
                size: x_sz,
                radius: x_padding, // radius is used as padding in type_=2
                type_: 2,
            });
            vertices.push(ColorVertex {
                pos: [x_right, x_top],
                uv: [1.0, 1.0],
                color: x_color,
                size: x_sz,
                radius: x_padding,
                type_: 2,
            });
            vertices.push(ColorVertex {
                pos: [x_left, x_bottom],
                uv: [-1.0, -1.0],
                color: x_color,
                size: x_sz,
                radius: x_padding,
                type_: 2,
            });
            vertices.push(ColorVertex {
                pos: [x_right, x_bottom],
                uv: [1.0, -1.0],
                color: x_color,
                size: x_sz,
                radius: x_padding,
                type_: 2,
            });

            // 頂点バッファを更新
            if let Some(vb) = &self.vertex_buffer {
                let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
                self.context
                    .Map(vb, 0, D3D11_MAP_WRITE_DISCARD, 0, Some(&mut mapped))?;
                std::ptr::copy_nonoverlapping(
                    vertices.as_ptr(),
                    mapped.pData as *mut ColorVertex,
                    vertices.len(),
                );
                self.context.Unmap(vb, 0);
            }

            // RTV作成
            let mut rtv = None;
            self.device
                .CreateRenderTargetView(backbuffer, None, Some(&mut rtv))?;
            let rtv = rtv.ok_or_else(|| anyhow!("Failed to create RTV"))?;

            // ビューポート
            let viewport = D3D11_VIEWPORT {
                TopLeftX: 0.0,
                TopLeftY: 0.0,
                Width: width,
                Height: height,
                MinDepth: 0.0,
                MaxDepth: 1.0,
            };

            // パイプライン設定
            self.context.OMSetRenderTargets(Some(&[Some(rtv)]), None);
            self.context.RSSetViewports(Some(&[viewport]));

            if let Some(blend) = &self.blend_state {
                self.context.OMSetBlendState(blend, None, 0xffffffff);
            }

            self.context
                .IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP);

            if let Some(il) = &self.input_layout {
                self.context.IASetInputLayout(il);
            }

            if let Some(vb) = &self.vertex_buffer {
                let stride = std::mem::size_of::<ColorVertex>() as u32;
                let offset = 0u32;
                self.context.IASetVertexBuffers(
                    0,
                    1,
                    Some(&Some(vb.clone())),
                    Some(&stride),
                    Some(&offset),
                );
            }

            if let Some(vs) = &self.vertex_shader {
                self.context.VSSetShader(vs, None);
            }

            if let Some(ps) = &self.pixel_shader {
                self.context.PSSetShader(ps, None);
            }

            // 背景を描画
            self.context.Draw(4, 0);
            // 終了ボタンを描画
            self.context.Draw(4, 4);
            // Xアイコンを描画
            self.context.Draw(4, 8);

            // クリーンアップ (D3D11レンダリング終了)
            self.context.OMSetRenderTargets(None, None);

            // DirectWrite でFPS描画 (D3D11描画の後に実行)
            if let (Some(d2d_factory), Some(text_format)) = (&self.d2d_factory, &self.text_format) {
                let dxgi_surface: IDXGISurface = backbuffer.cast()?;

                let render_target_props = D2D1_RENDER_TARGET_PROPERTIES {
                    r#type: windows::Win32::Graphics::Direct2D::D2D1_RENDER_TARGET_TYPE_DEFAULT,
                    pixelFormat: D2D1_PIXEL_FORMAT {
                        format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_UNKNOWN,
                        alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                    },
                    dpiX: 0.0,
                    dpiY: 0.0,
                    usage: windows::Win32::Graphics::Direct2D::D2D1_RENDER_TARGET_USAGE_NONE,
                    minLevel: windows::Win32::Graphics::Direct2D::D2D1_FEATURE_LEVEL_DEFAULT,
                };

                let d2d_rt = d2d_factory
                    .CreateDxgiSurfaceRenderTarget(&dxgi_surface, &render_target_props)?;

                let white_color = D2D1_COLOR_F {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                };
                let white_brush = d2d_rt.CreateSolidColorBrush(&white_color, None)?;

                let fps_text = format!("{:.0} FPS", self.current_fps);
                let fps_text_wide: Vec<u16> =
                    fps_text.encode_utf16().chain(std::iter::once(0)).collect();

                let text_rect = windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F {
                    left: toolbar_x,
                    top: toolbar_y + corner_rounding,
                    right: toolbar_x + toolbar_width,
                    bottom: toolbar_y + toolbar_height,
                };

                d2d_rt.BeginDraw();
                d2d_rt.DrawText(
                    &fps_text_wide[..fps_text_wide.len() - 1],
                    text_format,
                    &text_rect,
                    &white_brush,
                    windows::Win32::Graphics::Direct2D::D2D1_DRAW_TEXT_OPTIONS_NONE,
                    windows::Win32::Graphics::DirectWrite::DWRITE_MEASURING_MODE_NATURAL,
                );
                d2d_rt.EndDraw(None, None)?;
            }
        }

        Ok(())
    }

    /// 7セグメント数字パターンを取得 (各ビットが1セグメント)
    /// セグメント配置:  _0_
    ///                 |5 |1|
    ///                  -6-
    ///                 |4 |2|
    ///                  -3-
    fn get_7segment_pattern(ch: char) -> u8 {
        match ch {
            '0' => 0b0111111, // 0,1,2,3,4,5
            '1' => 0b0000110, // 1,2
            '2' => 0b1011011, // 0,1,3,4,6
            '3' => 0b1001111, // 0,1,2,3,6
            '4' => 0b1100110, // 1,2,5,6
            '5' => 0b1101101, // 0,2,3,5,6
            '6' => 0b1111101, // 0,2,3,4,5,6
            '7' => 0b0000111, // 0,1,2
            '8' => 0b1111111, // all
            '9' => 0b1101111, // 0,1,2,3,5,6
            _ => 0,
        }
    }

    /// 7セグメント数字の頂点を追加
    fn add_7segment_vertices<F1, F2>(
        vertices: &mut Vec<ColorVertex>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        segments: u8,
        color: [f32; 4],
        to_ndc_x: &F1,
        to_ndc_y: &F2,
    ) where
        F1: Fn(f32) -> f32,
        F2: Fn(f32) -> f32,
    {
        let seg_thickness = 2.5f32; // 太くして視認性向上
        let half_h = h / 2.0;

        // セグメント0 (上横): y, x to x+w
        if segments & (1 << 0) != 0 {
            Self::add_rect(vertices, x, y, w, seg_thickness, color, to_ndc_x, to_ndc_y);
        }
        // セグメント1 (右上縦): x+w-t, y to y+h/2
        if segments & (1 << 1) != 0 {
            Self::add_rect(
                vertices,
                x + w - seg_thickness,
                y,
                seg_thickness,
                half_h,
                color,
                to_ndc_x,
                to_ndc_y,
            );
        }
        // セグメント2 (右下縦): x+w-t, y+h/2 to y+h
        if segments & (1 << 2) != 0 {
            Self::add_rect(
                vertices,
                x + w - seg_thickness,
                y + half_h,
                seg_thickness,
                half_h,
                color,
                to_ndc_x,
                to_ndc_y,
            );
        }
        // セグメント3 (下横): y+h-t, x to x+w
        if segments & (1 << 3) != 0 {
            Self::add_rect(
                vertices,
                x,
                y + h - seg_thickness,
                w,
                seg_thickness,
                color,
                to_ndc_x,
                to_ndc_y,
            );
        }
        // セグメント4 (左下縦): x, y+h/2 to y+h
        if segments & (1 << 4) != 0 {
            Self::add_rect(
                vertices,
                x,
                y + half_h,
                seg_thickness,
                half_h,
                color,
                to_ndc_x,
                to_ndc_y,
            );
        }
        // セグメント5 (左上縦): x, y to y+h/2
        if segments & (1 << 5) != 0 {
            Self::add_rect(
                vertices,
                x,
                y,
                seg_thickness,
                half_h,
                color,
                to_ndc_x,
                to_ndc_y,
            );
        }
        // セグメント6 (中横): y+h/2-t/2, x to x+w
        if segments & (1 << 6) != 0 {
            Self::add_rect(
                vertices,
                x,
                y + half_h - seg_thickness / 2.0,
                w,
                seg_thickness,
                color,
                to_ndc_x,
                to_ndc_y,
            );
        }
    }

    /// 矩形の頂点を追加 (triangle strip用)
    fn add_rect<F1, F2>(
        vertices: &mut Vec<ColorVertex>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: [f32; 4],
        to_ndc_x: &F1,
        to_ndc_y: &F2,
    ) where
        F1: Fn(f32) -> f32,
        F2: Fn(f32) -> f32,
    {
        let left = to_ndc_x(x);
        let right = to_ndc_x(x + w);
        let top = to_ndc_y(y);
        let bottom = to_ndc_y(y + h);
        let size = [w, h];
        // Type 0 = flat color (no SDF)
        vertices.push(ColorVertex {
            pos: [left, top],
            uv: [-1.0, 1.0],
            color,
            size,
            radius: 0.0,
            type_: 0,
        });
        vertices.push(ColorVertex {
            pos: [right, top],
            uv: [1.0, 1.0],
            color,
            size,
            radius: 0.0,
            type_: 0,
        });
        vertices.push(ColorVertex {
            pos: [left, bottom],
            uv: [-1.0, -1.0],
            color,
            size,
            radius: 0.0,
            type_: 0,
        });
        vertices.push(ColorVertex {
            pos: [right, bottom],
            uv: [1.0, -1.0],
            color,
            size,
            radius: 0.0,
            type_: 0,
        });
    }

    /// 表示中かどうか
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    /// 現在のFPS
    pub fn fps(&self) -> f32 {
        self.current_fps
    }
}
