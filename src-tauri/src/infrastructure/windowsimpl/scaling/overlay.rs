// D3D11ベースのシンプルなツールバー
// カーソルが画面上端に近づくと表示され、FPSと終了ボタンを含む

use anyhow::{anyhow, Result};
use std::time::Instant;
use windows::core::PCSTR;
use windows::Win32::Foundation::POINT;
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
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R32G32B32A32_FLOAT;

// 頂点シェーダー
const VS_SOURCE: &str = r#"
struct VS_INPUT {
    float2 pos : POSITION;
    float4 color : COLOR;
};
struct VS_OUTPUT {
    float4 pos : SV_POSITION;
    float4 color : COLOR;
};
VS_OUTPUT main(VS_INPUT input) {
    VS_OUTPUT output;
    output.pos = float4(input.pos, 0.0, 1.0);
    output.color = input.color;
    return output;
}
"#;

// ピクセルシェーダー (単色)
const PS_SOURCE: &str = r#"
struct PS_INPUT {
    float4 pos : SV_POSITION;
    float4 color : COLOR;
};
float4 main(PS_INPUT input) : SV_TARGET {
    return input.color;
}
"#;

// 頂点データ構造体
#[repr(C)]
struct ColorVertex {
    pos: [f32; 2],
    color: [f32; 4],
}

/// シンプルなD3D11ツールバー
pub struct SimpleToolbar {
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    vertex_shader: Option<ID3D11VertexShader>,
    pixel_shader: Option<ID3D11PixelShader>,
    input_layout: Option<ID3D11InputLayout>,
    vertex_buffer: Option<ID3D11Buffer>,
    blend_state: Option<ID3D11BlendState>,
    initialized: bool,

    // FPS計測
    fps_update_time: Instant,
    frame_count: u32,
    current_fps: f32,

    // 表示状態
    is_visible: bool,
    visibility_timer: Instant,

    // ボタン領域 (NDC座標ではなくピクセル座標で保持)
    pub close_button_rect: (f32, f32, f32, f32), // (left, top, right, bottom)
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
            fps_update_time: now,
            frame_count: 0,
            current_fps: 0.0,
            is_visible: false,
            visibility_timer: now,
            close_button_rect: (0.0, 0.0, 0.0, 0.0),
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

            // 入力レイアウト
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
                    SemanticName: PCSTR(b"COLOR\0".as_ptr()),
                    SemanticIndex: 0,
                    Format: DXGI_FORMAT_R32G32B32A32_FLOAT,
                    InputSlot: 0,
                    AlignedByteOffset: 8,
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
    /// 画面上端50ピクセル以内でカーソルがあれば表示、離れたら即座に非表示
    pub fn update_visibility(&mut self, cursor_pos: POINT, _screen_height: i32) {
        let near_top = cursor_pos.y < 50;
        self.is_visible = near_top;
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
    pub fn render(&mut self, backbuffer: &ID3D11Texture2D) -> Result<()> {
        if !self.is_visible {
            return Ok(());
        }

        if !self.initialized {
            self.initialize()?;
        }

        unsafe {
            // バックバッファのサイズを取得
            let mut bb_desc = D3D11_TEXTURE2D_DESC::default();
            backbuffer.GetDesc(&mut bb_desc);
            let width = bb_desc.Width as f32;
            let height = bb_desc.Height as f32;

            // ツールバーの寸法 (ピクセル)
            let toolbar_width = 200.0f32;
            let toolbar_height = 40.0f32;
            let toolbar_x = (width - toolbar_width) / 2.0;
            let toolbar_y = 10.0;

            // 終了ボタンの寸法
            let button_size = 30.0f32;
            let button_x = toolbar_x + toolbar_width - button_size - 5.0;
            let button_y = toolbar_y + 5.0;

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

            // ツールバー背景の頂点
            let bg_color = [0.1, 0.1, 0.1, 0.85]; // 暗いグレー、半透明
            let bg_left = to_ndc_x(toolbar_x);
            let bg_right = to_ndc_x(toolbar_x + toolbar_width);
            let bg_top = to_ndc_y(toolbar_y);
            let bg_bottom = to_ndc_y(toolbar_y + toolbar_height);

            // 終了ボタンの頂点
            let btn_color = [0.8, 0.2, 0.2, 1.0]; // 赤
            let btn_left = to_ndc_x(button_x);
            let btn_right = to_ndc_x(button_x + button_size);
            let btn_top = to_ndc_y(button_y);
            let btn_bottom = to_ndc_y(button_y + button_size);

            // 頂点データを収集するVec
            let mut vertices: Vec<ColorVertex> = Vec::new();

            // 背景矩形 (triangle strip: 4頂点)
            vertices.push(ColorVertex {
                pos: [bg_left, bg_top],
                color: bg_color,
            });
            vertices.push(ColorVertex {
                pos: [bg_right, bg_top],
                color: bg_color,
            });
            vertices.push(ColorVertex {
                pos: [bg_left, bg_bottom],
                color: bg_color,
            });
            vertices.push(ColorVertex {
                pos: [bg_right, bg_bottom],
                color: bg_color,
            });

            // 終了ボタン矩形 (triangle strip: 4頂点)
            vertices.push(ColorVertex {
                pos: [btn_left, btn_top],
                color: btn_color,
            });
            vertices.push(ColorVertex {
                pos: [btn_right, btn_top],
                color: btn_color,
            });
            vertices.push(ColorVertex {
                pos: [btn_left, btn_bottom],
                color: btn_color,
            });
            vertices.push(ColorVertex {
                pos: [btn_right, btn_bottom],
                color: btn_color,
            });

            // FPS数字の描画
            let fps_text = format!("{:.0}", self.current_fps);
            let digit_width = 8.0f32;
            let digit_height = 14.0f32;
            let digit_spacing = 10.0f32;
            let fps_start_x = toolbar_x + 10.0;
            let fps_start_y = toolbar_y + (toolbar_height - digit_height) / 2.0;
            let text_color = [1.0, 1.0, 1.0, 1.0]; // 白

            let fps_vertex_start = vertices.len();
            for (i, ch) in fps_text.chars().enumerate() {
                let x = fps_start_x + i as f32 * digit_spacing;
                let y = fps_start_y;
                let segments = Self::get_7segment_pattern(ch);
                Self::add_7segment_vertices(
                    &mut vertices,
                    x,
                    y,
                    digit_width,
                    digit_height,
                    segments,
                    text_color,
                    &to_ndc_x,
                    &to_ndc_y,
                );
            }

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

            // FPS数字を描画
            let fps_vertices_count = vertices.len() - fps_vertex_start;
            for i in (0..fps_vertices_count).step_by(4) {
                self.context.Draw(4, (fps_vertex_start + i) as u32);
            }

            // クリーンアップ
            self.context.OMSetRenderTargets(None, None);
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
        let seg_thickness = 2.0f32;
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
        vertices.push(ColorVertex {
            pos: [left, top],
            color,
        });
        vertices.push(ColorVertex {
            pos: [right, top],
            color,
        });
        vertices.push(ColorVertex {
            pos: [left, bottom],
            color,
        });
        vertices.push(ColorVertex {
            pos: [right, bottom],
            color,
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
