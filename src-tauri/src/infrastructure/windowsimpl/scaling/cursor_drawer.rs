use anyhow::{anyhow, Result};
use std::collections::HashMap;
use windows::Win32::Foundation::POINT;
use windows::Win32::Graphics::Direct3D11::{
    ID3D11Device, ID3D11DeviceContext, ID3D11ShaderResourceView, ID3D11Texture2D,
    D3D11_BIND_SHADER_RESOURCE, D3D11_BOX, D3D11_SUBRESOURCE_DATA, D3D11_TEXTURE2D_DESC,
    D3D11_USAGE_DEFAULT,
};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC};
use windows::Win32::Graphics::Gdi::{
    DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC, BITMAP, BITMAPINFO, BITMAPINFOHEADER,
    BI_RGB, DIB_RGB_COLORS,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorInfo, GetIconInfo, CURSORINFO, HCURSOR, ICONINFO,
};

/// キャッシュされたカーソル情報
struct CachedCursor {
    texture: ID3D11Texture2D,
    _srv: ID3D11ShaderResourceView,
    hotspot: POINT,
    size: (i32, i32),
}

/// カーソル描画を管理
pub struct CursorDrawer {
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    cursor_cache: HashMap<isize, CachedCursor>,
    last_cursor_handle: isize,
    last_cursor_pos: POINT,
}

impl CursorDrawer {
    pub fn new(device: ID3D11Device, context: ID3D11DeviceContext) -> Self {
        Self {
            device,
            context,
            cursor_cache: HashMap::new(),
            last_cursor_handle: 0,
            last_cursor_pos: POINT {
                x: i32::MAX,
                y: i32::MAX,
            },
        }
    }

    /// 再描画が必要かどうかを判定
    pub fn need_redraw(&self, cursor_handle: isize, cursor_pos: POINT) -> bool {
        if cursor_handle != self.last_cursor_handle {
            return true;
        }
        if cursor_handle == 0 {
            return false;
        }
        cursor_pos.x != self.last_cursor_pos.x || cursor_pos.y != self.last_cursor_pos.y
    }

    /// 現在のカーソル情報を取得
    pub fn get_current_cursor() -> Option<(HCURSOR, POINT)> {
        unsafe {
            let mut ci = CURSORINFO {
                cbSize: std::mem::size_of::<CURSORINFO>() as u32,
                ..Default::default()
            };
            if GetCursorInfo(&mut ci).is_ok() && !ci.hCursor.is_invalid() {
                Some((ci.hCursor, ci.ptScreenPos))
            } else {
                None
            }
        }
    }

    /// カーソルテクスチャを作成
    fn create_cursor_texture(&self, h_cursor: HCURSOR) -> Option<CachedCursor> {
        unsafe {
            let mut icon_info = ICONINFO::default();
            if GetIconInfo(h_cursor, &mut icon_info).is_err() {
                return None;
            }

            let hotspot = POINT {
                x: icon_info.xHotspot as i32,
                y: icon_info.yHotspot as i32,
            };

            // マスクビットマップからサイズを取得
            let h_mask = icon_info.hbmMask;
            let h_color = icon_info.hbmColor;

            let mut bmp = BITMAP::default();
            if GetObjectW(
                h_mask,
                std::mem::size_of::<BITMAP>() as i32,
                Some(&mut bmp as *mut _ as *mut _),
            ) == 0
            {
                return None;
            }

            let width = bmp.bmWidth;
            let height = if h_color.is_invalid() {
                bmp.bmHeight / 2 // 白黒カーソルは高さが2倍
            } else {
                bmp.bmHeight
            };

            // ピクセルデータを取得
            let pixel_count = (width * height) as usize;
            let mut pixels: Vec<u8> = vec![0u8; pixel_count * 4];

            let hdc = GetDC(None);
            if hdc.is_invalid() {
                return None;
            }

            let bi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width,
                    biHeight: -height, // Top-down
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0 as u32,
                    biSizeImage: (pixel_count * 4) as u32,
                    ..Default::default()
                },
                ..Default::default()
            };

            let source_bitmap = if !h_color.is_invalid() {
                h_color
            } else {
                h_mask
            };

            let result = GetDIBits(
                hdc,
                source_bitmap,
                0,
                height as u32,
                Some(pixels.as_mut_ptr() as *mut _),
                &bi as *const _ as *mut _,
                DIB_RGB_COLORS,
            );

            ReleaseDC(None, hdc);

            if result == 0 {
                return None;
            }

            // カラーカーソルの場合、アルファチャネルを確認
            if !h_color.is_invalid() {
                // BGRAからRGBAに変換し、アルファを設定
                let has_alpha = pixels.iter().skip(3).step_by(4).any(|&a| a != 0);
                if !has_alpha {
                    // アルファがない場合、不透明に設定
                    for i in (3..pixels.len()).step_by(4) {
                        pixels[i] = 255;
                    }
                }
            } else {
                // 白黒カーソル: 簡易処理（白を白、黒を黒、透明を透明に）
                for i in (0..pixels.len()).step_by(4) {
                    let is_white = pixels[i] > 128;
                    if is_white {
                        pixels[i] = 255;
                        pixels[i + 1] = 255;
                        pixels[i + 2] = 255;
                        pixels[i + 3] = 255;
                    } else {
                        pixels[i] = 0;
                        pixels[i + 1] = 0;
                        pixels[i + 2] = 0;
                        pixels[i + 3] = 255;
                    }
                }
            }

            // ビットマップを削除
            if !h_color.is_invalid() {
                let _ = DeleteObject(h_color);
            }
            let _ = DeleteObject(h_mask);

            // D3D11テクスチャを作成
            let desc = D3D11_TEXTURE2D_DESC {
                Width: width as u32,
                Height: height as u32,
                MipLevels: 1,
                ArraySize: 1,
                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                Usage: D3D11_USAGE_DEFAULT,
                BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
                CPUAccessFlags: 0,
                MiscFlags: 0,
            };

            let init_data = D3D11_SUBRESOURCE_DATA {
                pSysMem: pixels.as_ptr() as *const _,
                SysMemPitch: (width * 4) as u32,
                SysMemSlicePitch: 0,
            };

            let mut texture = None;
            if self
                .device
                .CreateTexture2D(&desc, Some(&init_data), Some(&mut texture))
                .is_err()
            {
                return None;
            }
            let texture = texture?;

            let mut srv = None;
            if self
                .device
                .CreateShaderResourceView(&texture, None, Some(&mut srv))
                .is_err()
            {
                return None;
            }
            let srv = srv?;

            Some(CachedCursor {
                texture,
                _srv: srv,
                hotspot,
                size: (width, height),
            })
        }
    }

    /// カーソルをバックバッファに描画（簡易版: CopySubresourceRegion使用）
    pub fn draw(
        &mut self,
        backbuffer: &ID3D11Texture2D,
        cursor_pos: POINT,
        h_cursor: HCURSOR,
        cursor_scaling: f32,
    ) -> Result<()> {
        // まずキャッシュにテクスチャを作成/取得
        let key = h_cursor.0 as isize;
        if !self.cursor_cache.contains_key(&key) {
            if let Some(cached) = self.create_cursor_texture(h_cursor) {
                self.cursor_cache.insert(key, cached);
            } else {
                return Err(anyhow!("Failed to create cursor texture"));
            }
        }

        // キャッシュから必要な情報をクローン
        let (texture, hotspot, size) = {
            let cached = self.cursor_cache.get(&key).unwrap();
            (cached.texture.clone(), cached.hotspot, cached.size)
        };

        // 描画位置を計算（ホットスポットを考慮）
        let draw_x = cursor_pos.x - (hotspot.x as f32 * cursor_scaling) as i32;
        let draw_y = cursor_pos.y - (hotspot.y as f32 * cursor_scaling) as i32;

        // バックバッファのサイズを取得
        let mut bb_desc = D3D11_TEXTURE2D_DESC::default();
        unsafe {
            backbuffer.GetDesc(&mut bb_desc);
        }

        // 描画範囲をクリップ
        let cursor_width = (size.0 as f32 * cursor_scaling) as i32;
        let cursor_height = (size.1 as f32 * cursor_scaling) as i32;

        // 画面外チェック
        if draw_x + cursor_width < 0
            || draw_y + cursor_height < 0
            || draw_x >= bb_desc.Width as i32
            || draw_y >= bb_desc.Height as i32
        {
            return Ok(()); // 画面外
        }

        // スケーリングなしで直接コピー（シンプル版）
        if (cursor_scaling - 1.0).abs() < 0.01 {
            let src_box = D3D11_BOX {
                left: 0,
                top: 0,
                front: 0,
                right: size.0.min((bb_desc.Width as i32 - draw_x).max(0)) as u32,
                bottom: size.1.min((bb_desc.Height as i32 - draw_y).max(0)) as u32,
                back: 1,
            };

            let dest_x = draw_x.max(0) as u32;
            let dest_y = draw_y.max(0) as u32;

            unsafe {
                self.context.CopySubresourceRegion(
                    backbuffer,
                    0,
                    dest_x,
                    dest_y,
                    0,
                    &texture,
                    0,
                    Some(&src_box),
                );
            }
        }

        // 状態を更新
        self.last_cursor_handle = h_cursor.0 as isize;
        self.last_cursor_pos = cursor_pos;

        Ok(())
    }
}
