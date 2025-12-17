// D3D11ベースのカーソル描画
// システムカーソルのビットマップを取得してD3D11テクスチャとして描画

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use windows::core::PCSTR;
use windows::Win32::Foundation::POINT;
use windows::Win32::Graphics::Direct3D::Fxc::{D3DCompile, D3DCOMPILE_OPTIMIZATION_LEVEL3};
use windows::Win32::Graphics::Direct3D::D3D11_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP;
use windows::Win32::Graphics::Direct3D11::{
    ID3D11BlendState, ID3D11Buffer, ID3D11Device, ID3D11DeviceContext, ID3D11InputLayout,
    ID3D11PixelShader, ID3D11SamplerState, ID3D11ShaderResourceView, ID3D11Texture2D,
    ID3D11VertexShader, D3D11_BIND_SHADER_RESOURCE, D3D11_BIND_VERTEX_BUFFER, D3D11_BLEND_DESC,
    D3D11_BLEND_INV_SRC_ALPHA, D3D11_BLEND_ONE, D3D11_BLEND_OP_ADD, D3D11_BUFFER_DESC,
    D3D11_COLOR_WRITE_ENABLE_ALL, D3D11_CPU_ACCESS_WRITE, D3D11_FILTER_MIN_MAG_MIP_POINT,
    D3D11_INPUT_ELEMENT_DESC, D3D11_INPUT_PER_VERTEX_DATA, D3D11_MAPPED_SUBRESOURCE,
    D3D11_MAP_WRITE_DISCARD, D3D11_RENDER_TARGET_BLEND_DESC, D3D11_SAMPLER_DESC,
    D3D11_SUBRESOURCE_DATA, D3D11_TEXTURE2D_DESC, D3D11_TEXTURE_ADDRESS_CLAMP, D3D11_USAGE_DEFAULT,
    D3D11_USAGE_DYNAMIC, D3D11_VIEWPORT,
};
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_R32G32_FLOAT, DXGI_SAMPLE_DESC,
};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, DeleteDC, GetDIBits, GetObjectW, BITMAP, BITMAPINFO, BITMAPINFOHEADER,
    BI_RGB, DIB_RGB_COLORS,
};
use windows::Win32::UI::WindowsAndMessaging::{GetIconInfo, HCURSOR, ICONINFO};

// 頂点シェーダー (HLSL)
const VS_SOURCE: &str = r#"
struct VS_INPUT {
    float2 pos : POSITION;
    float2 tex : TEXCOORD;
};
struct VS_OUTPUT {
    float4 pos : SV_POSITION;
    float2 tex : TEXCOORD;
};
VS_OUTPUT main(VS_INPUT input) {
    VS_OUTPUT output;
    output.pos = float4(input.pos, 0.0, 1.0);
    output.tex = input.tex;
    return output;
}
"#;

// ピクセルシェーダー (テクスチャサンプリング)
const PS_TEXTURE_SOURCE: &str = r#"
Texture2D cursorTex : register(t0);
SamplerState samplerState : register(s0);
struct PS_INPUT {
    float4 pos : SV_POSITION;
    float2 tex : TEXCOORD;
};
float4 main(PS_INPUT input) : SV_TARGET {
    float4 color = cursorTex.Sample(samplerState, input.tex);
    // 予乗済みアルファを元に戻す (Magpie方式)
    // color.rgb は既に予乗済み、color.a は 255 - 元のアルファ
    float alpha = 1.0 - color.a;
    if (alpha < 0.001) discard;
    return float4(color.rgb, alpha);
}
"#;

// 頂点データ構造体
#[repr(C)]
struct Vertex {
    pos: [f32; 2],
    tex: [f32; 2],
}

/// カーソル情報（キャッシュ用）
#[derive(Clone)]
struct CachedCursorInfo {
    texture: ID3D11Texture2D,
    srv: ID3D11ShaderResourceView,
    width: u32,
    height: u32,
    hot_spot_x: i32,
    hot_spot_y: i32,
    is_inverse: bool,
}

/// D3D11ベースのカーソル描画
pub struct CursorRenderer {
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    vertex_shader: Option<ID3D11VertexShader>,
    pixel_shader: Option<ID3D11PixelShader>,
    input_layout: Option<ID3D11InputLayout>,
    vertex_buffer: Option<ID3D11Buffer>,
    blend_state: Option<ID3D11BlendState>,
    inverse_blend_state: Option<ID3D11BlendState>,
    sampler_state: Option<ID3D11SamplerState>,
    cached_cursors: HashMap<isize, CachedCursorInfo>,
    initialized: bool,
    last_valid_h_cursor: Option<windows::Win32::UI::WindowsAndMessaging::HCURSOR>,
}

impl CursorRenderer {
    pub fn new(device: ID3D11Device, context: ID3D11DeviceContext) -> Self {
        Self {
            device,
            context,
            vertex_shader: None,
            pixel_shader: None,
            input_layout: None,
            vertex_buffer: None,
            blend_state: None,
            inverse_blend_state: None,
            sampler_state: None,
            cached_cursors: HashMap::new(),
            initialized: false,
            last_valid_h_cursor: None,
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
                    Format: DXGI_FORMAT_R32G32_FLOAT,
                    InputSlot: 0,
                    AlignedByteOffset: 0,
                    InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                    InstanceDataStepRate: 0,
                },
                D3D11_INPUT_ELEMENT_DESC {
                    SemanticName: PCSTR(b"TEXCOORD\0".as_ptr()),
                    SemanticIndex: 0,
                    Format: DXGI_FORMAT_R32G32_FLOAT,
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

            // ピクセルシェーダーをコンパイル (テクスチャサンプリング)
            let mut ps_blob = None;
            D3DCompile(
                PS_TEXTURE_SOURCE.as_ptr() as *const _,
                PS_TEXTURE_SOURCE.len(),
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

            // 頂点バッファ (動的)
            let buffer_desc = D3D11_BUFFER_DESC {
                ByteWidth: (std::mem::size_of::<Vertex>() * 4) as u32,
                Usage: D3D11_USAGE_DYNAMIC,
                BindFlags: D3D11_BIND_VERTEX_BUFFER.0 as u32,
                CPUAccessFlags: D3D11_CPU_ACCESS_WRITE.0 as u32,
                ..Default::default()
            };
            let mut vertex_buffer = None;
            self.device
                .CreateBuffer(&buffer_desc, None, Some(&mut vertex_buffer))?;
            let current_vertex_buffer = self.vertex_buffer.clone();
            self.vertex_buffer = vertex_buffer;

            // ブレンド状態 (予乗済みアルファ)
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

            // ブレンド状態 (反転描画用: Iビームなど)
            // SrcBlend = INV_DEST_COLOR (Src * (1-Dest))
            // DestBlend = INV_SRC_ALPHA (Dest * (1-SrcAlpha))
            use windows::Win32::Graphics::Direct3D11::D3D11_BLEND_INV_DEST_COLOR;
            let inv_blend_desc = D3D11_BLEND_DESC {
                RenderTarget: [
                    D3D11_RENDER_TARGET_BLEND_DESC {
                        BlendEnable: true.into(),
                        SrcBlend: D3D11_BLEND_INV_DEST_COLOR,
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
            let mut inverse_blend_state = None;
            self.device
                .CreateBlendState(&inv_blend_desc, Some(&mut inverse_blend_state))?;
            self.inverse_blend_state = inverse_blend_state;

            // サンプラー状態
            let sampler_desc = D3D11_SAMPLER_DESC {
                Filter: D3D11_FILTER_MIN_MAG_MIP_POINT,
                AddressU: D3D11_TEXTURE_ADDRESS_CLAMP,
                AddressV: D3D11_TEXTURE_ADDRESS_CLAMP,
                AddressW: D3D11_TEXTURE_ADDRESS_CLAMP,
                MaxAnisotropy: 1,
                ComparisonFunc: windows::Win32::Graphics::Direct3D11::D3D11_COMPARISON_NEVER,
                MinLOD: 0.0,
                MaxLOD: f32::MAX,
                ..Default::default()
            };
            let mut sampler_state = None;
            self.device
                .CreateSamplerState(&sampler_desc, Some(&mut sampler_state))?;
            self.sampler_state = sampler_state;
        }

        self.initialized = true;
        Ok(())
    }

    /// カーソルハンドルからテクスチャを取得（キャッシュあり）
    fn get_cursor_texture(&mut self, h_cursor: HCURSOR) -> Option<&CachedCursorInfo> {
        let cursor_key = h_cursor.0 as isize;

        if self.cached_cursors.contains_key(&cursor_key) {
            return self.cached_cursors.get(&cursor_key);
        }

        // カーソル情報を取得
        let mut icon_info = ICONINFO::default();
        unsafe {
            if GetIconInfo(h_cursor, &mut icon_info).is_err() {
                return None;
            }
        }

        // ビットマップ情報を取得
        let mut bmp = BITMAP::default();
        let hbm_to_use = if !icon_info.hbmColor.is_invalid() {
            icon_info.hbmColor
        } else {
            icon_info.hbmMask
        };

        unsafe {
            if GetObjectW(
                hbm_to_use,
                std::mem::size_of::<BITMAP>() as i32,
                Some(&mut bmp as *mut _ as *mut _),
            ) == 0
            {
                return None;
            }
        }

        let width = bmp.bmWidth as u32;
        let height = if icon_info.hbmColor.is_invalid() {
            bmp.bmHeight as u32 / 2 // 単色カーソルはAND/XORマスクで2倍高さ
        } else {
            bmp.bmHeight as u32
        };

        // ビットマップデータを取得
        let bi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: bmp.bmWidth,
                biHeight: -(bmp.bmHeight as i32), // トップダウン
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                biSizeImage: (bmp.bmWidth * bmp.bmHeight * 4) as u32,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut pixels = vec![0u8; (bmp.bmWidth * bmp.bmHeight * 4) as usize];

        unsafe {
            let hdc = CreateCompatibleDC(None);
            if hdc.is_invalid() {
                return None;
            }

            let result = GetDIBits(
                hdc,
                hbm_to_use,
                0,
                bmp.bmHeight as u32,
                Some(pixels.as_mut_ptr() as *mut _),
                &bi as *const _ as *mut _,
                DIB_RGB_COLORS,
            );
            let _ = DeleteDC(hdc);

            if result == 0 {
                return None;
            }
        }

        let mut is_inverse = false;

        // BGRAからRGBAに変換し、アルファ処理
        if icon_info.hbmColor.is_invalid() {
            // モノクロカーソル (AND Mask + XOR Mask)
            let half_height = bmp.bmHeight as usize / 2;
            let mut final_pixels = vec![0u8; (width as usize * height as usize * 4)];

            // スキャンパス: 反転ピクセルがあるかチェック
            for y in 0..height as usize {
                for x in 0..width as usize {
                    let idx = (y * width as usize + x) * 4;
                    let mask_idx = idx;
                    let xor_idx = idx + (half_height * width as usize * 4);

                    let and_val = pixels[mask_idx] != 0;
                    let xor_val = pixels[xor_idx] != 0;

                    if and_val && xor_val {
                        is_inverse = true;
                        break;
                    }
                }
                if is_inverse {
                    break;
                }
            }

            for y in 0..height as usize {
                for x in 0..width as usize {
                    let idx = (y * width as usize + x) * 4;
                    // Top-Down DIB: AND mask is first half, XOR mask is second half
                    let mask_idx = idx;
                    let xor_idx = idx + (half_height * width as usize * 4);

                    // Get values (using Red channel)
                    let and_val = pixels[mask_idx] != 0;
                    let xor_val = pixels[xor_idx] != 0;

                    if is_inverse {
                        if and_val && !xor_val {
                            // Transparent
                            final_pixels[idx] = 0;
                            final_pixels[idx + 1] = 0;
                            final_pixels[idx + 2] = 0;
                            final_pixels[idx + 3] = 255;
                        } else if !and_val && !xor_val {
                            // Black
                            final_pixels[idx] = 0;
                            final_pixels[idx + 1] = 0;
                            final_pixels[idx + 2] = 0;
                            final_pixels[idx + 3] = 0;
                        } else if !and_val && xor_val {
                            // White -> Becomes Inverted
                            final_pixels[idx] = 255;
                            final_pixels[idx + 1] = 255;
                            final_pixels[idx + 2] = 255;
                            final_pixels[idx + 3] = 0;
                        } else {
                            // Inverse -> Becomes Inverted
                            final_pixels[idx] = 255;
                            final_pixels[idx + 1] = 255;
                            final_pixels[idx + 2] = 255;
                            final_pixels[idx + 3] = 0;
                        }
                    } else {
                        // Standard Logic (Alpha Blending)
                        // Shader: alpha = 1.0 - tex.a
                        if and_val && !xor_val {
                            // Transparent -> tex.a=255 -> alpha=0
                            final_pixels[idx] = 0;
                            final_pixels[idx + 1] = 0;
                            final_pixels[idx + 2] = 0;
                            final_pixels[idx + 3] = 255;
                        } else if !and_val && !xor_val {
                            // Black -> tex.a=0 -> alpha=1
                            final_pixels[idx] = 0;
                            final_pixels[idx + 1] = 0;
                            final_pixels[idx + 2] = 0;
                            final_pixels[idx + 3] = 0;
                        } else {
                            // White or Inverse(treated as White) -> tex.a=0 -> alpha=1
                            final_pixels[idx] = 255;
                            final_pixels[idx + 1] = 255;
                            final_pixels[idx + 2] = 255;
                            final_pixels[idx + 3] = 0;
                        }
                    }
                }
            }
            pixels = final_pixels;
        } else {
            // カラーカーソル (Masked Color含む)
            let pixel_count = (width * height) as usize;

            // アルファチャンネルチェック
            let mut has_alpha = false;
            for i in 0..pixel_count {
                if pixels[i * 4 + 3] != 0 {
                    has_alpha = true;
                    break;
                }
            }

            if has_alpha {
                // 通常のカラーカーソル
                for i in 0..pixel_count {
                    let idx = i * 4;
                    // BGRをRGBにスワップ
                    pixels.swap(idx, idx + 2);

                    // アルファチャンネル処理
                    let alpha = pixels[idx + 3] as f32 / 255.0;
                    if alpha > 0.0 {
                        // 予乗済みアルファに変換
                        pixels[idx] = (pixels[idx] as f32 * alpha) as u8;
                        pixels[idx + 1] = (pixels[idx + 1] as f32 * alpha) as u8;
                        pixels[idx + 2] = (pixels[idx + 2] as f32 * alpha) as u8;
                        // Magpie方式: アルファを反転 (255 - alpha)
                        pixels[idx + 3] = 255 - pixels[idx + 3];
                    } else {
                        // 完全透明
                        pixels[idx + 3] = 255;
                    }
                }
            } else {
                // マスク付きカラーカーソル (Alphaなし, Maskあり)
                let mut mask_pixels = vec![0u8; (width * height * 4) as usize];
                unsafe {
                    let hdc = CreateCompatibleDC(None);
                    if !hdc.is_invalid() {
                        let bi_mask = BITMAPINFO {
                            bmiHeader: BITMAPINFOHEADER {
                                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                                biWidth: bmp.bmWidth,
                                biHeight: -(bmp.bmHeight as i32), // Top-down
                                biPlanes: 1,
                                biBitCount: 32,
                                biCompression: BI_RGB.0,
                                biSizeImage: (width * height * 4) as u32,
                                ..Default::default()
                            },
                            ..Default::default()
                        };

                        GetDIBits(
                            hdc,
                            icon_info.hbmMask,
                            0,
                            height,
                            Some(mask_pixels.as_mut_ptr() as *mut _),
                            &bi_mask as *const _ as *mut _,
                            DIB_RGB_COLORS,
                        );
                        let _ = DeleteDC(hdc);
                    }
                }

                for i in 0..pixel_count {
                    let idx = i * 4;
                    // BGRをRGBにスワップ
                    pixels.swap(idx, idx + 2);

                    // マスクを確認 (Mask!=0 -> Transparent, Mask=0 -> Opaque)
                    let is_mask_white = mask_pixels[idx] != 0;

                    if is_mask_white {
                        // Transparent
                        pixels[idx] = 0;
                        pixels[idx + 1] = 0;
                        pixels[idx + 2] = 0;
                        pixels[idx + 3] = 255; // Alpha=0
                    } else {
                        // Opaque (Keep color)
                        pixels[idx + 3] = 0; // Alpha=1
                    }
                }
            }
        }

        // D3D11テクスチャを作成
        let tex_desc = D3D11_TEXTURE2D_DESC {
            Width: width,
            Height: height,
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
            ..Default::default()
        };

        let init_data = D3D11_SUBRESOURCE_DATA {
            pSysMem: pixels.as_ptr() as *const _,
            SysMemPitch: width * 4,
            SysMemSlicePitch: 0,
        };

        let mut texture = None;
        let mut srv = None;

        unsafe {
            if self
                .device
                .CreateTexture2D(&tex_desc, Some(&init_data), Some(&mut texture))
                .is_err()
            {
                return None;
            }

            let texture = texture?;

            if self
                .device
                .CreateShaderResourceView(&texture, None, Some(&mut srv))
                .is_err()
            {
                return None;
            }

            let srv = srv?;

            let cursor_info = CachedCursorInfo {
                texture,
                srv,
                width,
                height,
                hot_spot_x: icon_info.xHotspot as i32,
                hot_spot_y: icon_info.yHotspot as i32,
                is_inverse,
            };

            self.cached_cursors.insert(cursor_key, cursor_info);
        }

        self.cached_cursors.get(&cursor_key)
    }

    /// カーソルを描画
    /// viewport_rect: カーソルをクリップするビューポート矩形（クライアント座標系、Noneの場合はクリップなし）
    pub fn draw_cursor(
        &mut self,
        backbuffer: &ID3D11Texture2D,
        h_cursor: HCURSOR,
        cursor_pos: POINT,
        scale: f32,
        viewport_rect: Option<windows::Win32::Foundation::RECT>,
    ) -> Result<()> {
        if !self.initialized {
            self.initialize()?;
        }

        // カーソルテクスチャを取得
        // get_cursor_textureの戻り値の借用とself.last_valid_h_cursorへの代入が競合するため、クローンして借用を解除する
        let cursor_info_opt = self.get_cursor_texture(h_cursor).cloned();

        let cursor_info = match cursor_info_opt {
            Some(info) => {
                self.last_valid_h_cursor = Some(h_cursor);
                info
            }
            None => {
                // Fallback to last valid cursor
                if let Some(last_h) = self.last_valid_h_cursor {
                    if let Some(info) = self.cached_cursors.get(&(last_h.0 as isize)).cloned() {
                        info
                    } else {
                        return Ok(());
                    }
                } else {
                    return Ok(());
                }
            }
        };

        let cursor_width = (cursor_info.width as f32 * scale) as i32;
        let cursor_height = (cursor_info.height as f32 * scale) as i32;
        let hot_spot_x = (cursor_info.hot_spot_x as f32 * scale) as i32;
        let hot_spot_y = (cursor_info.hot_spot_y as f32 * scale) as i32;

        // SRVをクローン（すでにClone済みだが念のため）
        let srv = cursor_info.srv.clone();

        unsafe {
            // バックバッファのサイズを取得
            let mut bb_desc = D3D11_TEXTURE2D_DESC::default();
            backbuffer.GetDesc(&mut bb_desc);
            let width = bb_desc.Width as f32;
            let height = bb_desc.Height as f32;

            // カーソル描画位置（ホットスポット考慮）
            let draw_x = cursor_pos.x - hot_spot_x;
            let draw_y = cursor_pos.y - hot_spot_y;

            // カーソル矩形を計算
            let cursor_rect = windows::Win32::Foundation::RECT {
                left: draw_x,
                top: draw_y,
                right: draw_x + cursor_width,
                bottom: draw_y + cursor_height,
            };

            // ビューポート判定 (Magpie方式: CursorDrawer.cpp L141-154)
            // ビューポートが指定されている場合、カーソル矩形とビューポートをチェック
            let (clip_left, clip_top, clip_right, clip_bottom) = if let Some(vp) = viewport_rect {
                // カーソル矩形がビューポート完全に外にある場合は描画しない
                if cursor_rect.left >= vp.right
                    || cursor_rect.top >= vp.bottom
                    || cursor_rect.right <= vp.left
                    || cursor_rect.bottom <= vp.top
                {
                    return Ok(());
                }
                // クリップ座標を計算
                (
                    cursor_rect.left.max(vp.left),
                    cursor_rect.top.max(vp.top),
                    cursor_rect.right.min(vp.right),
                    cursor_rect.bottom.min(vp.bottom),
                )
            } else {
                // ビューポートなし = クリップなし
                (
                    cursor_rect.left,
                    cursor_rect.top,
                    cursor_rect.right,
                    cursor_rect.bottom,
                )
            };

            // クリップされたUV座標を計算
            let cursor_w = cursor_rect.right - cursor_rect.left;
            let cursor_h = cursor_rect.bottom - cursor_rect.top;
            let uv_left = (clip_left - cursor_rect.left) as f32 / cursor_w as f32;
            let uv_top = (clip_top - cursor_rect.top) as f32 / cursor_h as f32;
            let uv_right = (clip_right - cursor_rect.left) as f32 / cursor_w as f32;
            let uv_bottom = (clip_bottom - cursor_rect.top) as f32 / cursor_h as f32;

            // NDC座標に変換（クリップ後の座標を使用）
            let left = clip_left as f32 / width * 2.0 - 1.0;
            let right = clip_right as f32 / width * 2.0 - 1.0;
            let top = 1.0 - clip_top as f32 / height * 2.0;
            let bottom = 1.0 - clip_bottom as f32 / height * 2.0;

            // 頂点バッファを更新（クリップされたUV座標を使用）
            let vertices = [
                Vertex {
                    pos: [left, top],
                    tex: [uv_left, uv_top],
                },
                Vertex {
                    pos: [right, top],
                    tex: [uv_right, uv_top],
                },
                Vertex {
                    pos: [left, bottom],
                    tex: [uv_left, uv_bottom],
                },
                Vertex {
                    pos: [right, bottom],
                    tex: [uv_right, uv_bottom],
                },
            ];

            if let Some(vb) = &self.vertex_buffer {
                let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
                self.context
                    .Map(vb, 0, D3D11_MAP_WRITE_DISCARD, 0, Some(&mut mapped))?;
                std::ptr::copy_nonoverlapping(
                    vertices.as_ptr(),
                    mapped.pData as *mut Vertex,
                    vertices.len(),
                );
                self.context.Unmap(vb, 0);
            }

            // レンダーターゲットビューを作成
            let mut rtv = None;
            self.device
                .CreateRenderTargetView(backbuffer, None, Some(&mut rtv))?;
            let rtv = rtv.ok_or_else(|| anyhow!("Failed to create RTV"))?;

            // ビューポート設定
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

            // カーソルタイプに応じたブレンドステートを設定
            if cursor_info.is_inverse {
                if let Some(blend) = &self.inverse_blend_state {
                    self.context.OMSetBlendState(blend, None, 0xffffffff);
                }
            } else {
                if let Some(blend) = &self.blend_state {
                    self.context.OMSetBlendState(blend, None, 0xffffffff);
                }
            }

            self.context
                .IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP);

            if let Some(il) = &self.input_layout {
                self.context.IASetInputLayout(il);
            }

            if let Some(vb) = &self.vertex_buffer {
                let stride = std::mem::size_of::<Vertex>() as u32;
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

            // テクスチャとサンプラーをバインド
            self.context.PSSetShaderResources(0, Some(&[Some(srv)]));
            if let Some(sampler) = &self.sampler_state {
                self.context
                    .PSSetSamplers(0, Some(&[Some(sampler.clone())]));
            }

            // 描画
            self.context.Draw(4, 0);

            // クリーンアップ
            self.context.PSSetShaderResources(0, Some(&[None]));
            self.context.OMSetRenderTargets(None, None);
        }

        Ok(())
    }

    /// フォールバック: 白丸を描画（テクスチャ取得失敗時用）
    pub fn draw_circle(
        &mut self,
        backbuffer: &ID3D11Texture2D,
        cursor_pos: POINT,
        cursor_size: f32,
    ) -> Result<()> {
        // テクスチャなしで描画する場合のフォールバック
        // 今はdraw_cursorを呼び出す代わりにこれを保持
        if !self.initialized {
            self.initialize()?;
        }

        // 現在のカーソルを取得
        let cursor_info = unsafe {
            windows::Win32::UI::WindowsAndMessaging::GetCursorInfo(
                &mut windows::Win32::UI::WindowsAndMessaging::CURSORINFO {
                    cbSize: std::mem::size_of::<windows::Win32::UI::WindowsAndMessaging::CURSORINFO>(
                    ) as u32,
                    ..Default::default()
                },
            )
        };

        if cursor_info.is_ok() {
            let mut ci = windows::Win32::UI::WindowsAndMessaging::CURSORINFO {
                cbSize: std::mem::size_of::<windows::Win32::UI::WindowsAndMessaging::CURSORINFO>()
                    as u32,
                ..Default::default()
            };
            unsafe {
                if windows::Win32::UI::WindowsAndMessaging::GetCursorInfo(&mut ci).is_ok()
                    && !ci.hCursor.is_invalid()
                {
                    return self.draw_cursor(
                        backbuffer,
                        ci.hCursor,
                        cursor_pos,
                        cursor_size / 32.0,
                        None,
                    );
                }
            }
        }

        Ok(())
    }
}
