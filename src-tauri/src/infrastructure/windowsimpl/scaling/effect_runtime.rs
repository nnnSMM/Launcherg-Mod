//! エフェクトランタイム
//! マルチパスエフェクトの実行時リソース管理

use super::effect_desc::*;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use windows::Win32::Graphics::Direct3D::Fxc::{
    D3DCOMPILE_ENABLE_STRICTNESS, D3DCOMPILE_OPTIMIZATION_LEVEL3,
};
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::*;

/// コンパイル済みパス
#[derive(Clone, Serialize, Deserialize)]
pub struct CompiledPass {
    /// シェーダーバイトコード
    pub cso: Vec<u8>,
    /// 入力テクスチャインデックス
    pub inputs: Vec<usize>,
    /// 出力テクスチャインデックス
    pub outputs: Vec<usize>,
    /// ブロックサイズ
    pub block_size: (u32, u32),
    /// スレッド数
    pub num_threads: [u32; 3],
    /// PSスタイルかどうか
    pub is_ps_style: bool,
    /// 説明
    pub desc: String,
}

/// コンパイル済みエフェクト
#[derive(Clone, Serialize, Deserialize)]
pub struct CompiledEffect {
    /// エフェクト名
    pub name: String,
    /// パス一覧
    pub passes: Vec<CompiledPass>,
    /// テクスチャ記述子
    pub textures: Vec<EffectTextureDesc>,
    /// サンプラー記述子
    pub samplers: Vec<EffectSamplerDesc>,
    /// パラメータ
    pub params: Vec<EffectParameterDesc>,
    /// フラグ
    pub flags: u32,
}

/// キャッシュ用データ
#[derive(Serialize, Deserialize)]
pub struct EffectCacheData {
    /// ソースのハッシュ
    pub source_hash: u64,
    /// コンパイラバージョン
    pub compiler_version: u32,
    /// コンパイル済みエフェクト
    pub effect: CompiledEffect,
}

/// テクスチャ記述子（実行時用）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EffectTextureDesc {
    /// 名前
    pub name: String,
    /// フォーマット
    pub format: EffectIntermediateTextureFormat,
    /// 幅の式
    pub width_expr: String,
    /// 高さの式
    pub height_expr: String,
    /// ソース（外部テクスチャの場合）
    pub source: String,
    /// 入力テクスチャかどうか
    pub is_input: bool,
    /// 出力テクスチャかどうか
    pub is_output: bool,
}

impl EffectTextureDesc {
    pub fn input() -> Self {
        Self {
            name: "INPUT".to_string(),
            format: EffectIntermediateTextureFormat::Unknown,
            width_expr: String::new(),
            height_expr: String::new(),
            source: String::new(),
            is_input: true,
            is_output: false,
        }
    }

    pub fn output() -> Self {
        Self {
            name: "OUTPUT".to_string(),
            format: EffectIntermediateTextureFormat::Unknown,
            width_expr: String::new(),
            height_expr: String::new(),
            source: String::new(),
            is_input: false,
            is_output: true,
        }
    }
}

/// ランタイムテクスチャ（GPU上に作成されたテクスチャ）
#[derive(Clone)]
pub struct RuntimeTexture {
    pub texture: ID3D11Texture2D,
    pub srv: Option<ID3D11ShaderResourceView>,
    pub uav: Option<ID3D11UnorderedAccessView>,
    pub width: u32,
    pub height: u32,
    pub format: DXGI_FORMAT,
}

/// エフェクトランタイム
pub struct EffectRuntime {
    /// デバイス
    device: ID3D11Device,
    /// コンテキスト
    context: ID3D11DeviceContext,
    /// コンパイル済みシェーダー
    shaders: Vec<ID3D11ComputeShader>,
    /// ランタイムテクスチャ
    textures: Vec<Option<RuntimeTexture>>,
    /// サンプラーステート
    samplers: Vec<ID3D11SamplerState>,
    /// エフェクト記述
    effect: CompiledEffect,
    /// 入力サイズ
    input_size: (u32, u32),
    /// 出力サイズ
    output_size: (u32, u32),
    /// Blit用（スケーリング）シェーダー
    blit_shader: ID3D11ComputeShader,
    /// Blit用定数バッファ
    blit_buffer: ID3D11Buffer,
}

#[repr(C)]
struct BlitConstants {
    dst_offset: [u32; 2],
    dst_size: [u32; 2],
    input_size: [f32; 2],
    param_b: f32,
    param_c: f32,
}

impl EffectRuntime {
    /// 新しいエフェクトランタイムを作成
    pub fn new(
        device: ID3D11Device,
        context: ID3D11DeviceContext,
        effect: CompiledEffect,
    ) -> Result<Self> {
        // シェーダーを作成
        let mut shaders = Vec::new();
        for pass in &effect.passes {
            let shader = Self::create_compute_shader(&device, &pass.cso)?;
            shaders.push(shader);
        }

        // サンプラーを作成
        let mut samplers = Vec::new();
        for sampler_desc in &effect.samplers {
            let sampler = Self::create_sampler(&device, sampler_desc)?;
            samplers.push(sampler);
        }

        // テクスチャのスロットを確保
        let textures = vec![None; effect.textures.len()];

        // Blitリソース作成
        let blit_shader = Self::create_blit_shader(&device)?;
        let blit_buffer = Self::create_blit_buffer(&device)?;

        Ok(Self {
            device,
            context,
            shaders,
            textures,
            samplers,
            effect,
            input_size: (0, 0),
            output_size: (0, 0),
            blit_shader,
            blit_buffer,
        })
    }

    // Bicubic Blit Shader
    fn create_blit_shader(device: &ID3D11Device) -> Result<ID3D11ComputeShader> {
        let source = r#"
            Texture2D<float4> Input : register(t0);
            RWTexture2D<float4> Output : register(u0);
            SamplerState Linear : register(s0);

            cbuffer CB : register(b0) {
                uint2 dst_offset;
                uint2 dst_size;
                float2 input_size;
                float param_b;
                float param_c;
            }

            float weight(float x) {
// ...
                const float B = param_b;
                const float C = param_c;
                float ax = abs(x);

                if (ax < 1.0) {
                    return (x * x * ((12.0 - 9.0 * B - 6.0 * C) * ax + (-18.0 + 12.0 * B + 6.0 * C)) + (6.0 - 2.0 * B)) / 6.0;
                } else if (ax >= 1.0 && ax < 2.0) {
                    return (x * x * ((-B - 6.0 * C) * ax + (6.0 * B + 30.0 * C)) + (-12.0 * B - 48.0 * C) * ax + (8.0 * B + 24.0 * C)) / 6.0;
                } else {
                    return 0.0;
                }
            }

            float4 weight4(float x) {
                return float4(
                    weight(x - 2.0),
                    weight(x - 1.0),
                    weight(x),
                    weight(x + 1.0)
                );
            }

            [numthreads(8, 8, 1)]
            void main(uint3 id : SV_DispatchThreadID) {
                if (id.x >= dst_size.x || id.y >= dst_size.y) return;
                
                // UV Calculation (Center of pixel)
                float2 uv = (float2(id.xy) + 0.5) / float2(dst_size);
                
                // Map UV to Input Texture space
                // Note: Input is the whole intermediate texture, so UV is 0..1
                
                float2 pos = uv * input_size;
                float2 pos1 = floor(pos - 0.5) + 0.5;
                float2 f = pos - pos1;

                float4 rowtaps = weight4(1.0 - f.x);
                float4 coltaps = weight4(1.0 - f.y);

                // Re-normalize weights
                rowtaps /= rowtaps.r + rowtaps.g + rowtaps.b + rowtaps.a;
                coltaps /= coltaps.r + coltaps.g + coltaps.b + coltaps.a;

                float2 inputPt = 1.0 / input_size;
                float2 uv1 = pos1 * inputPt;
                float2 uv0 = uv1 - inputPt;
                float2 uv2 = uv1 + inputPt;
                float2 uv3 = uv2 + inputPt;

                float u_weight_sum = rowtaps.y + rowtaps.z;
                float u_middle_offset = rowtaps.z * inputPt.x / u_weight_sum;
                float u_middle = uv1.x + u_middle_offset;

                float v_weight_sum = coltaps.y + coltaps.z;
                float v_middle_offset = coltaps.z * inputPt.y / v_weight_sum;
                float v_middle = uv1.y + v_middle_offset;

                // 9-tap Mixed Sampling (Load + SampleLevel)
                // Note: Use clamp to avoid out-of-bounds Load
                int2 max_coord = int2(input_size - 1.0);
                
                int2 coord_top_left = int2(max(uv0 * input_size, 0.5));
                // Clamp coordinates for Load
                coord_top_left = clamp(coord_top_left, int2(0,0), max_coord);
                
                int2 coord_bottom_right = int2(min(uv3 * input_size, input_size - 0.5));
                coord_bottom_right = clamp(coord_bottom_right, int2(0,0), max_coord);

                float3 top = Input.Load(int3(coord_top_left, 0)).rgb * rowtaps.x;
                top += Input.SampleLevel(Linear, float2(u_middle, uv0.y), 0).rgb * u_weight_sum;
                top += Input.Load(int3(coord_bottom_right.x, coord_top_left.y, 0)).rgb * rowtaps.w;
                float3 total = top * coltaps.x;

                float3 middle = Input.SampleLevel(Linear, float2(uv0.x, v_middle), 0).rgb * rowtaps.x;
                middle += Input.SampleLevel(Linear, float2(u_middle, v_middle), 0).rgb * u_weight_sum;
                middle += Input.SampleLevel(Linear, float2(uv3.x, v_middle), 0).rgb * rowtaps.w;
                total += middle * v_weight_sum;

                float3 bottom = Input.Load(int3(coord_top_left.x, coord_bottom_right.y, 0)).rgb * rowtaps.x;
                bottom += Input.SampleLevel(Linear, float2(u_middle, uv3.y), 0).rgb * u_weight_sum;
                bottom += Input.Load(int3(coord_bottom_right, 0)).rgb * rowtaps.w;
                total += bottom * coltaps.w;

                int2 out_pos = int2(id.x + dst_offset.x, id.y + dst_offset.y);
                Output[out_pos] = float4(total, 1.0);
            }
        "#;

        // Compile shader (using D3DCompile from d3dcompiler_47.dll which is linked)
        use windows::core::s;
        use windows::Win32::Graphics::Direct3D::Fxc::D3DCompile;
        use windows::Win32::Graphics::Direct3D::ID3DBlob;

        let flags = D3DCOMPILE_ENABLE_STRICTNESS | D3DCOMPILE_OPTIMIZATION_LEVEL3;
        let mut bytecode: Option<ID3DBlob> = None;
        let mut error_blob: Option<ID3DBlob> = None;

        unsafe {
            let res = D3DCompile(
                source.as_ptr() as *const _,
                source.len(),
                None,
                None,
                None,
                s!("main"),
                s!("cs_5_0"),
                flags,
                0,
                &mut bytecode,
                Some(&mut error_blob),
            );

            if res.is_err() {
                if let Some(blob) = error_blob {
                    let ptr = blob.GetBufferPointer();
                    let size = blob.GetBufferSize();
                    let slice = std::slice::from_raw_parts(ptr as *const u8, size);
                    let msg = String::from_utf8_lossy(slice);
                    return Err(anyhow!("Blit Shader Compile Error: {}", msg));
                }
                return Err(anyhow!("Blit Shader Compile Failed"));
            }
        }

        let blob = bytecode.unwrap();
        let bytes = unsafe {
            std::slice::from_raw_parts(blob.GetBufferPointer() as *const u8, blob.GetBufferSize())
        };
        Self::create_compute_shader(device, bytes)
    }

    fn create_blit_buffer(device: &ID3D11Device) -> Result<ID3D11Buffer> {
        let desc = D3D11_BUFFER_DESC {
            ByteWidth: std::mem::size_of::<BlitConstants>() as u32,
            Usage: D3D11_USAGE_DYNAMIC,
            BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as u32,
            CPUAccessFlags: D3D11_CPU_ACCESS_WRITE.0 as u32,
            MiscFlags: 0,
            StructureByteStride: 0,
        };

        let mut buffer = None;
        unsafe {
            device.CreateBuffer(&desc, None, Some(&mut buffer))?;
        }
        buffer.ok_or_else(|| anyhow!("Failed to create blit buffer"))
    }

    pub fn execute_blit(
        &self,
        src: &ID3D11Texture2D,
        dst: &ID3D11Texture2D,
        dst_rect: windows::Win32::Foundation::RECT,
    ) -> Result<()> {
        let width = (dst_rect.right - dst_rect.left).max(0) as u32;
        let height = (dst_rect.bottom - dst_rect.top).max(0) as u32;

        if width == 0 || height == 0 {
            return Ok(());
        }

        let mut src_desc = D3D11_TEXTURE2D_DESC::default();
        unsafe { src.GetDesc(&mut src_desc) };
        let input_w = src_desc.Width as f32;
        let input_h = src_desc.Height as f32;

        let constants = BlitConstants {
            dst_offset: [dst_rect.left as u32, dst_rect.top as u32],
            dst_size: [width, height],
            input_size: [input_w, input_h],
            param_b: 0.33,
            param_c: 0.33,
        };

        unsafe {
            // RTVがバインドされたままだとUAVバインドに失敗するため、明示的にアンバインド
            self.context.OMSetRenderTargets(Some(&[]), None);

            // 定数バッファ更新
            let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
            self.context.Map(
                &self.blit_buffer,
                0,
                D3D11_MAP_WRITE_DISCARD,
                0,
                Some(&mut mapped),
            )?;
            std::ptr::copy_nonoverlapping(&constants, mapped.pData as *mut BlitConstants, 1);
            self.context.Unmap(&self.blit_buffer, 0);

            // リソース作成
            let mut srv_desc = D3D11_SHADER_RESOURCE_VIEW_DESC::default();
            srv_desc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
            srv_desc.ViewDimension = D3D_SRV_DIMENSION_TEXTURE2D;
            srv_desc.Anonymous.Texture2D.MipLevels = 1;

            let mut uav_desc = D3D11_UNORDERED_ACCESS_VIEW_DESC::default();
            uav_desc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
            uav_desc.ViewDimension = D3D11_UAV_DIMENSION_TEXTURE2D;

            // Get Desc to check formats
            let mut src_desc = D3D11_TEXTURE2D_DESC::default();
            src.GetDesc(&mut src_desc);
            srv_desc.Format = src_desc.Format;

            let mut dst_desc = D3D11_TEXTURE2D_DESC::default();
            dst.GetDesc(&mut dst_desc);
            uav_desc.Format = dst_desc.Format;

            let mut srv = None;
            self.device
                .CreateShaderResourceView(src, Some(&srv_desc), Some(&mut srv))?;

            let mut uav = None;
            self.device
                .CreateUnorderedAccessView(dst, Some(&uav_desc), Some(&mut uav))?;

            // Bind
            self.context.CSSetShader(Some(&self.blit_shader), None);

            let buffers = [Some(self.blit_buffer.clone())];
            self.context.CSSetConstantBuffers(0, Some(&buffers));

            let srvs = [srv];
            self.context.CSSetShaderResources(0, Some(&srvs));

            let uavs = [uav];
            self.context
                .CSSetUnorderedAccessViews(0, 1, Some(uavs.as_ptr()), None);

            // Linear Sampler
            if let Some(sampler) = self.samplers.first() {
                let samplers = [Some(sampler.clone())];
                self.context.CSSetSamplers(0, Some(&samplers));
            }

            // Dispatch
            let dispatch_x = (width + 7) / 8;
            let dispatch_y = (height + 7) / 8;
            self.context.Dispatch(dispatch_x, dispatch_y, 1);

            // Unbind
            let null_uav = [None];
            self.context
                .CSSetUnorderedAccessViews(0, 1, Some(null_uav.as_ptr()), None);
            let null_srv = [None];
            self.context.CSSetShaderResources(0, Some(&null_srv));
            self.context.CSSetShader(None, None);
        }

        Ok(())
    }

    /// サイズを更新し、必要に応じてテクスチャを再作成
    pub fn update_size(&mut self, input_size: (u32, u32), output_size: (u32, u32)) -> Result<()> {
        if self.input_size == input_size && self.output_size == output_size {
            return Ok(());
        }

        self.input_size = input_size;
        self.output_size = output_size;

        // 中間テクスチャを再作成
        for (i, tex_desc) in self.effect.textures.iter().enumerate() {
            // INPUT と OUTPUT はスキップ
            if tex_desc.is_input || tex_desc.is_output {
                continue;
            }

            // サイズを計算
            let (width, height) = self.evaluate_size_expr(
                &tex_desc.width_expr,
                &tex_desc.height_expr,
                input_size,
                output_size,
            )?;

            // フォーマットを取得
            let format = tex_desc.format.desc().dxgi_format;

            // テクスチャを作成
            let runtime_tex = self.create_texture(width, height, format)?;
            self.textures[i] = Some(runtime_tex);
        }

        Ok(())
    }

    pub fn get_pass_infos(&self) -> [MagpiePassInfo; 16] {
        let mut infos = [MagpiePassInfo::default(); 16];
        for (i, pass) in self.effect.passes.iter().enumerate() {
            if i >= 16 {
                break;
            }
            if let Some(&output_idx) = pass.outputs.first() {
                if let Some(Some(tex)) = self.textures.get(output_idx) {
                    infos[i].output_size = [tex.width, tex.height];
                    infos[i].output_pt = [1.0 / tex.width as f32, 1.0 / tex.height as f32];
                }
            }
        }
        infos
    }

    /// エフェクトが要求する推奨出力サイズを計算
    pub fn get_preferred_output_size(
        &self,
        input_size: (u32, u32),
        current_target_size: (u32, u32),
    ) -> Result<(u32, u32)> {
        // OUTPUT テクスチャを探す
        let output_tex = self.effect.textures.iter().find(|t| t.is_output);

        if let Some(desc) = output_tex {
            if !desc.width_expr.is_empty() {
                // OUTPUT_WIDTH/HEIGHT は自己参照しないはずなので、ターゲットサイズを渡すか、0を渡す
                // Magpieではこの段階では未定義だが、eval_expr_innerは値を要求する
                // 式には恐らくINPUT_WIDTHしか使われないため、current_target_sizeを渡しても安全なはず
                let w = self.eval_expr_inner(&desc.width_expr, input_size, current_target_size)?;
                let h = self.eval_expr_inner(&desc.height_expr, input_size, current_target_size)?;
                return Ok((w.round() as u32, h.round() as u32));
            }
        }

        Ok(current_target_size)
    }

    /// サイズ式の変数を評価
    fn evaluate_size_expr(
        &self,
        width_expr: &str,
        height_expr: &str,
        input_size: (u32, u32),
        output_size: (u32, u32),
    ) -> Result<(u32, u32)> {
        let w = self.eval_expr_inner(width_expr, input_size, output_size)?;
        let h = self.eval_expr_inner(height_expr, input_size, output_size)?;
        Ok((w.round() as u32, h.round() as u32))
    }

    fn eval_expr_inner(
        &self,
        expr: &str,
        input_size: (u32, u32),
        output_size: (u32, u32),
    ) -> Result<f32> {
        let expr = expr.trim();
        if expr.is_empty() {
            return Ok(0.0);
        }

        // 変数置換
        let expr_replaced = expr
            .replace("INPUT_WIDTH", &input_size.0.to_string())
            .replace("INPUT_HEIGHT", &input_size.1.to_string())
            .replace("OUTPUT_WIDTH", &output_size.0.to_string())
            .replace("OUTPUT_HEIGHT", &output_size.1.to_string());

        // 簡易評価 ( +, -, *, /, %, max, min )
        // 本来はちゃんとしたパーサーが必要だが、ここでは簡易的な実装にする
        // Magpieのエフェクトで使用される式の範囲をカバーする

        // カッコの処理は省略（必要なら実装）
        // 演算子の優先順位: *, /, % > +, -

        // トークン分割 (スペースで区切られていると仮定しない)
        // 再帰的評価を行う

        self.parse_and_eval(&expr_replaced)
    }

    fn parse_and_eval(&self, expr: &str) -> Result<f32> {
        // +, - で分割
        // (注: カッコ内の +, - は無視する必要があるが、ここではカッコなしと仮定するか、簡易パース)
        // 実装を簡単にするため、スペースなしで結合されている可能性を考慮せず、単純な eval を実装する
        // ここでは非常に簡易な実装にとどめる。必要に応じて強化する。

        let terms: Vec<&str> = expr.split('+').collect();
        if terms.len() > 1 {
            let mut sum = 0.0;
            for term in terms {
                sum += self.parse_and_eval(term)?;
            }
            return Ok(sum);
        }

        let terms: Vec<&str> = expr.split('-').collect();
        if terms.len() > 1 {
            // First term positive, others negative?
            // Handle "a - b - c" -> (a) - (b) - (c)
            // Left associative
            // Split by '-' is tricky with negative numbers. Assuming positive inputs mostly.
            let mut total = self.parse_and_eval(terms[0])?;
            for term in &terms[1..] {
                total -= self.parse_and_eval(term)?;
            }
            return Ok(total);
        }

        // *, /
        let factors: Vec<&str> = expr.split('*').collect();
        if factors.len() > 1 {
            let mut product = 1.0;
            for factor in factors {
                product *= self.parse_and_eval(factor)?;
            }
            return Ok(product);
        }

        let factors: Vec<&str> = expr.split('/').collect();
        if factors.len() > 1 {
            let mut result = self.parse_and_eval(factors[0])?;
            for factor in &factors[1..] {
                result /= self.parse_and_eval(factor)?;
            }
            return Ok(result);
        }

        // 関数 (max, min, floor, ceil)
        let expr = expr.trim();
        if expr.starts_with("max(") && expr.ends_with(")") {
            let content = &expr[4..expr.len() - 1];
            let args: Vec<&str> = content.split(',').collect();
            if args.len() == 2 {
                let v1 = self.parse_and_eval(args[0])?;
                let v2 = self.parse_and_eval(args[1])?;
                return Ok(v1.max(v2));
            }
        }
        if expr.starts_with("floor(") && expr.ends_with(")") {
            let content = &expr[6..expr.len() - 1];
            return Ok(self.parse_and_eval(content)?.floor());
        }

        // 数値
        expr.trim()
            .parse::<f32>()
            .map_err(|_| anyhow!("Invalid number/expr: {}", expr))
    }

    /// 入力テクスチャを設定
    pub fn set_input_texture(&mut self, texture: &ID3D11Texture2D) -> Result<()> {
        // INPUT のインデックスを見つける
        let input_idx = self
            .effect
            .textures
            .iter()
            .position(|t| t.is_input)
            .ok_or_else(|| anyhow!("INPUT texture not found"))?;

        // SRVを作成
        let srv = self.create_srv(texture)?;

        // テクスチャサイズを取得
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        unsafe { texture.GetDesc(&mut desc) };

        self.textures[input_idx] = Some(RuntimeTexture {
            texture: texture.clone(),
            srv: Some(srv),
            uav: None,
            width: desc.Width,
            height: desc.Height,
            format: desc.Format,
        });

        Ok(())
    }

    /// 出力テクスチャを設定
    pub fn set_output_texture(&mut self, texture: &ID3D11Texture2D) -> Result<()> {
        // OUTPUT のインデックスを見つける
        let output_idx = self
            .effect
            .textures
            .iter()
            .position(|t| t.is_output)
            .ok_or_else(|| anyhow!("OUTPUT texture not found"))?;

        // UAVを作成
        let uav = self.create_uav(texture)?;

        // テクスチャサイズを取得
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        unsafe { texture.GetDesc(&mut desc) };

        self.textures[output_idx] = Some(RuntimeTexture {
            texture: texture.clone(),
            srv: None,
            uav: Some(uav),
            width: desc.Width,
            height: desc.Height,
            format: desc.Format,
        });

        Ok(())
    }

    /// すべてのパスを実行
    pub fn execute(&self, constants_buffer: &ID3D11Buffer) -> Result<()> {
        unsafe {
            // 定数バッファをバインド
            self.context
                .CSSetConstantBuffers(0, Some(&[Some(constants_buffer.clone())]));

            // サンプラーをバインド
            let sampler_refs: Vec<Option<ID3D11SamplerState>> =
                self.samplers.iter().map(|s| Some(s.clone())).collect();
            self.context.CSSetSamplers(0, Some(&sampler_refs));

            // 各パスを実行
            for (pass_idx, pass) in self.effect.passes.iter().enumerate() {
                self.execute_pass(pass_idx, pass)?;
            }

            // リソースをアンバインド
            self.context.CSSetShader(None, None);
            let null_srvs = [None, None, None, None, None, None, None, None];
            self.context.CSSetShaderResources(0, Some(&null_srvs));
            let null_uavs = [None, None, None, None, None, None, None, None];
            self.context
                .CSSetUnorderedAccessViews(0, 8, Some(null_uavs.as_ptr()), None);
        }

        Ok(())
    }

    /// 単一パスを実行
    fn execute_pass(&self, pass_idx: usize, pass: &CompiledPass) -> Result<()> {
        unsafe {
            // シェーダーをバインド
            self.context
                .CSSetShader(Some(&self.shaders[pass_idx]), None);

            // 入力テクスチャ（SRV）をバインド
            let mut srvs: Vec<Option<ID3D11ShaderResourceView>> = Vec::new();
            for &input_idx in &pass.inputs {
                if let Some(ref tex) = self.textures[input_idx] {
                    srvs.push(tex.srv.clone());
                } else {
                    srvs.push(None);
                }
            }
            if !srvs.is_empty() {
                self.context.CSSetShaderResources(0, Some(&srvs));
            }

            // 出力テクスチャ（UAV）をバインド
            let mut uavs: Vec<Option<ID3D11UnorderedAccessView>> = Vec::new();
            for &output_idx in &pass.outputs {
                if let Some(ref tex) = self.textures[output_idx] {
                    uavs.push(tex.uav.clone());
                } else {
                    uavs.push(None);
                }
            }
            if !uavs.is_empty() {
                self.context.CSSetUnorderedAccessViews(
                    0,
                    uavs.len() as u32,
                    Some(uavs.as_ptr()),
                    None,
                );
            }

            // ディスパッチサイズを計算
            let output_size = if !pass.outputs.is_empty() {
                if let Some(ref tex) = self.textures[pass.outputs[0]] {
                    (tex.width, tex.height)
                } else {
                    self.output_size
                }
            } else {
                self.output_size
            };

            let dispatch_x = (output_size.0 + pass.block_size.0 - 1) / pass.block_size.0;
            let dispatch_y = (output_size.1 + pass.block_size.1 - 1) / pass.block_size.1;

            self.context.Dispatch(dispatch_x, dispatch_y, 1);

            // リソースハザードを防ぐためにUAVとSRVをアンバインド
            let null_uavs = vec![None; uavs.len()];
            if !null_uavs.is_empty() {
                self.context.CSSetUnorderedAccessViews(
                    0,
                    null_uavs.len() as u32,
                    Some(null_uavs.as_ptr()),
                    None,
                );
            }

            let null_srvs = vec![None; srvs.len()];
            if !null_srvs.is_empty() {
                self.context.CSSetShaderResources(0, Some(&null_srvs));
            }
        }

        Ok(())
    }

    // ヘルパー関数

    fn create_compute_shader(
        device: &ID3D11Device,
        bytecode: &[u8],
    ) -> Result<ID3D11ComputeShader> {
        let mut shader = None;
        unsafe {
            device.CreateComputeShader(
                bytecode,
                Option::<&ID3D11ClassLinkage>::None,
                Some(&mut shader),
            )?;
        }
        shader.ok_or_else(|| anyhow!("Failed to create compute shader"))
    }

    fn create_sampler(
        device: &ID3D11Device,
        desc: &EffectSamplerDesc,
    ) -> Result<ID3D11SamplerState> {
        let filter = match desc.filter_type {
            EffectSamplerFilterType::Point => D3D11_FILTER_MIN_MAG_MIP_POINT,
            EffectSamplerFilterType::Linear => D3D11_FILTER_MIN_MAG_MIP_LINEAR,
        };

        let address = match desc.address_type {
            EffectSamplerAddressType::Clamp => D3D11_TEXTURE_ADDRESS_CLAMP,
            EffectSamplerAddressType::Wrap => D3D11_TEXTURE_ADDRESS_WRAP,
        };

        let sampler_desc = D3D11_SAMPLER_DESC {
            Filter: filter,
            AddressU: address,
            AddressV: address,
            AddressW: address,
            MipLODBias: 0.0,
            MaxAnisotropy: 1,
            ComparisonFunc: D3D11_COMPARISON_NEVER,
            BorderColor: [0.0; 4],
            MinLOD: 0.0,
            MaxLOD: f32::MAX,
        };

        let mut sampler = None;
        unsafe {
            device.CreateSamplerState(&sampler_desc, Some(&mut sampler))?;
        }
        sampler.ok_or_else(|| anyhow!("Failed to create sampler state"))
    }

    fn create_texture(
        &self,
        width: u32,
        height: u32,
        format: DXGI_FORMAT,
    ) -> Result<RuntimeTexture> {
        let desc = D3D11_TEXTURE2D_DESC {
            Width: width,
            Height: height,
            MipLevels: 1,
            ArraySize: 1,
            Format: format,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: (D3D11_BIND_SHADER_RESOURCE | D3D11_BIND_UNORDERED_ACCESS).0 as u32,
            CPUAccessFlags: D3D11_CPU_ACCESS_FLAG(0).0 as u32,
            MiscFlags: D3D11_RESOURCE_MISC_FLAG(0).0 as u32,
        };

        let mut texture = None;
        unsafe {
            self.device
                .CreateTexture2D(&desc, None, Some(&mut texture))?;
        }
        let texture = texture.ok_or_else(|| anyhow!("Failed to create texture"))?;

        let srv = self.create_srv(&texture)?;
        let uav = self.create_uav(&texture)?;

        Ok(RuntimeTexture {
            texture,
            srv: Some(srv),
            uav: Some(uav),
            width,
            height,
            format,
        })
    }

    fn create_srv(&self, texture: &ID3D11Texture2D) -> Result<ID3D11ShaderResourceView> {
        let mut srv = None;
        unsafe {
            self.device
                .CreateShaderResourceView(texture, None, Some(&mut srv))?;
        }
        srv.ok_or_else(|| anyhow!("Failed to create SRV"))
    }

    fn create_uav(&self, texture: &ID3D11Texture2D) -> Result<ID3D11UnorderedAccessView> {
        let mut uav = None;
        unsafe {
            self.device
                .CreateUnorderedAccessView(texture, None, Some(&mut uav))?;
        }
        uav.ok_or_else(|| anyhow!("Failed to create UAV"))
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MagpieConstants {
    pub input_size: [u32; 2],
    pub output_size: [u32; 2],
    pub input_pt: [f32; 2],
    pub output_pt: [f32; 2],
    pub scale: [f32; 2],
    pub src_rect_offset: [f32; 2], // Offset for pseudo-borderless (x, y)
    pub passes: [MagpiePassInfo; 16], // Support up to 16 passes
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct MagpiePassInfo {
    pub output_size: [u32; 2],
    pub output_pt: [f32; 2],
}
