//! エフェクト記述子の型定義
//! Magpie の EffectDesc.h を Rust で再実装

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use windows::Win32::Graphics::Dxgi::Common::*;

/// テクスチャの中間フォーマット
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum EffectIntermediateTextureFormat {
    R32G32B32A32Float,
    R16G16B16A16Float,
    R16G16B16A16Unorm,
    R16G16B16A16Snorm,
    R32G32Float,
    R10G10B10A2Unorm,
    R11G11B10Float,
    #[default]
    R8G8B8A8Unorm,
    R8G8B8A8Snorm,
    R16G16Float,
    R16G16Unorm,
    R16G16Snorm,
    R32Float,
    R8G8Unorm,
    R8G8Snorm,
    R16Float,
    R16Unorm,
    R16Snorm,
    R8Unorm,
    R8Snorm,
    Unknown,
}

/// フォーマット記述子
pub struct FormatDesc {
    pub name: &'static str,
    pub dxgi_format: DXGI_FORMAT,
    pub n_channel: u32,
    pub srv_texel_type: &'static str,
    pub uav_texel_type: &'static str,
}

/// フォーマット記述子テーブル (EffectHelper::FORMAT_DESCS相当)
pub const FORMAT_DESCS: &[FormatDesc] = &[
    FormatDesc {
        name: "R32G32B32A32_FLOAT",
        dxgi_format: DXGI_FORMAT(2), // DXGI_FORMAT_R32G32B32A32_FLOAT
        n_channel: 4,
        srv_texel_type: "float4",
        uav_texel_type: "float4",
    },
    FormatDesc {
        name: "R16G16B16A16_FLOAT",
        dxgi_format: DXGI_FORMAT(10), // DXGI_FORMAT_R16G16B16A16_FLOAT
        n_channel: 4,
        srv_texel_type: "MF4",
        uav_texel_type: "MF4",
    },
    FormatDesc {
        name: "R16G16B16A16_UNORM",
        dxgi_format: DXGI_FORMAT(11), // DXGI_FORMAT_R16G16B16A16_UNORM
        n_channel: 4,
        srv_texel_type: "MF4",
        uav_texel_type: "unorm MF4",
    },
    FormatDesc {
        name: "R16G16B16A16_SNORM",
        dxgi_format: DXGI_FORMAT(13), // DXGI_FORMAT_R16G16B16A16_SNORM
        n_channel: 4,
        srv_texel_type: "MF4",
        uav_texel_type: "snorm MF4",
    },
    FormatDesc {
        name: "R32G32_FLOAT",
        dxgi_format: DXGI_FORMAT(16), // DXGI_FORMAT_R32G32_FLOAT
        n_channel: 2,
        srv_texel_type: "float2",
        uav_texel_type: "float2",
    },
    FormatDesc {
        name: "R10G10B10A2_UNORM",
        dxgi_format: DXGI_FORMAT(24), // DXGI_FORMAT_R10G10B10A2_UNORM
        n_channel: 4,
        srv_texel_type: "MF4",
        uav_texel_type: "unorm MF4",
    },
    FormatDesc {
        name: "R11G11B10_FLOAT",
        dxgi_format: DXGI_FORMAT(26), // DXGI_FORMAT_R11G11B10_FLOAT
        n_channel: 3,
        srv_texel_type: "MF3",
        uav_texel_type: "MF3",
    },
    FormatDesc {
        name: "R8G8B8A8_UNORM",
        dxgi_format: DXGI_FORMAT(28), // DXGI_FORMAT_R8G8B8A8_UNORM
        n_channel: 4,
        srv_texel_type: "MF4",
        uav_texel_type: "unorm MF4",
    },
    FormatDesc {
        name: "R8G8B8A8_SNORM",
        dxgi_format: DXGI_FORMAT(31), // DXGI_FORMAT_R8G8B8A8_SNORM
        n_channel: 4,
        srv_texel_type: "MF4",
        uav_texel_type: "snorm MF4",
    },
    FormatDesc {
        name: "R16G16_FLOAT",
        dxgi_format: DXGI_FORMAT(34), // DXGI_FORMAT_R16G16_FLOAT
        n_channel: 2,
        srv_texel_type: "MF2",
        uav_texel_type: "MF2",
    },
    FormatDesc {
        name: "R16G16_UNORM",
        dxgi_format: DXGI_FORMAT(35), // DXGI_FORMAT_R16G16_UNORM
        n_channel: 2,
        srv_texel_type: "MF2",
        uav_texel_type: "unorm MF2",
    },
    FormatDesc {
        name: "R16G16_SNORM",
        dxgi_format: DXGI_FORMAT(37), // DXGI_FORMAT_R16G16_SNORM
        n_channel: 2,
        srv_texel_type: "MF2",
        uav_texel_type: "snorm MF2",
    },
    FormatDesc {
        name: "R32_FLOAT",
        dxgi_format: DXGI_FORMAT(41), // DXGI_FORMAT_R32_FLOAT
        n_channel: 1,
        srv_texel_type: "float",
        uav_texel_type: "float",
    },
    FormatDesc {
        name: "R8G8_UNORM",
        dxgi_format: DXGI_FORMAT(49), // DXGI_FORMAT_R8G8_UNORM
        n_channel: 2,
        srv_texel_type: "MF2",
        uav_texel_type: "unorm MF2",
    },
    FormatDesc {
        name: "R8G8_SNORM",
        dxgi_format: DXGI_FORMAT(51), // DXGI_FORMAT_R8G8_SNORM
        n_channel: 2,
        srv_texel_type: "MF2",
        uav_texel_type: "snorm MF2",
    },
    FormatDesc {
        name: "R16_FLOAT",
        dxgi_format: DXGI_FORMAT(54), // DXGI_FORMAT_R16_FLOAT
        n_channel: 1,
        srv_texel_type: "MF",
        uav_texel_type: "MF",
    },
    FormatDesc {
        name: "R16_UNORM",
        dxgi_format: DXGI_FORMAT(56), // DXGI_FORMAT_R16_UNORM
        n_channel: 1,
        srv_texel_type: "MF",
        uav_texel_type: "unorm MF",
    },
    FormatDesc {
        name: "R16_SNORM",
        dxgi_format: DXGI_FORMAT(58), // DXGI_FORMAT_R16_SNORM
        n_channel: 1,
        srv_texel_type: "MF",
        uav_texel_type: "snorm MF",
    },
    FormatDesc {
        name: "R8_UNORM",
        dxgi_format: DXGI_FORMAT(61), // DXGI_FORMAT_R8_UNORM
        n_channel: 1,
        srv_texel_type: "MF",
        uav_texel_type: "unorm MF",
    },
    FormatDesc {
        name: "R8_SNORM",
        dxgi_format: DXGI_FORMAT(63), // DXGI_FORMAT_R8_SNORM
        n_channel: 1,
        srv_texel_type: "MF",
        uav_texel_type: "snorm MF",
    },
    FormatDesc {
        name: "UNKNOWN",
        dxgi_format: DXGI_FORMAT(0), // DXGI_FORMAT_UNKNOWN
        n_channel: 4,
        srv_texel_type: "float4",
        uav_texel_type: "float4",
    },
];

impl EffectIntermediateTextureFormat {
    /// 文字列からフォーマットを解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "R32G32B32A32_FLOAT" => Some(Self::R32G32B32A32Float),
            "R16G16B16A16_FLOAT" => Some(Self::R16G16B16A16Float),
            "R16G16B16A16_UNORM" => Some(Self::R16G16B16A16Unorm),
            "R16G16B16A16_SNORM" => Some(Self::R16G16B16A16Snorm),
            "R32G32_FLOAT" => Some(Self::R32G32Float),
            "R10G10B10A2_UNORM" => Some(Self::R10G10B10A2Unorm),
            "R11G11B10_FLOAT" => Some(Self::R11G11B10Float),
            "R8G8B8A8_UNORM" => Some(Self::R8G8B8A8Unorm),
            "R8G8B8A8_SNORM" => Some(Self::R8G8B8A8Snorm),
            "R16G16_FLOAT" => Some(Self::R16G16Float),
            "R16G16_UNORM" => Some(Self::R16G16Unorm),
            "R16G16_SNORM" => Some(Self::R16G16Snorm),
            "R32_FLOAT" => Some(Self::R32Float),
            "R8G8_UNORM" => Some(Self::R8G8Unorm),
            "R8G8_SNORM" => Some(Self::R8G8Snorm),
            "R16_FLOAT" => Some(Self::R16Float),
            "R16_UNORM" => Some(Self::R16Unorm),
            "R16_SNORM" => Some(Self::R16Snorm),
            "R8_UNORM" => Some(Self::R8Unorm),
            "R8_SNORM" => Some(Self::R8Snorm),
            _ => None,
        }
    }

    /// フォーマットのインデックスを取得
    pub fn as_index(&self) -> usize {
        match self {
            Self::R32G32B32A32Float => 0,
            Self::R16G16B16A16Float => 1,
            Self::R16G16B16A16Unorm => 2,
            Self::R16G16B16A16Snorm => 3,
            Self::R32G32Float => 4,
            Self::R10G10B10A2Unorm => 5,
            Self::R11G11B10Float => 6,
            Self::R8G8B8A8Unorm => 7,
            Self::R8G8B8A8Snorm => 8,
            Self::R16G16Float => 9,
            Self::R16G16Unorm => 10,
            Self::R16G16Snorm => 11,
            Self::R32Float => 12,
            Self::R8G8Unorm => 13,
            Self::R8G8Snorm => 14,
            Self::R16Float => 15,
            Self::R16Unorm => 16,
            Self::R16Snorm => 17,
            Self::R8Unorm => 18,
            Self::R8Snorm => 19,
            Self::Unknown => 20,
        }
    }

    /// フォーマット記述子を取得
    pub fn desc(&self) -> &'static FormatDesc {
        &FORMAT_DESCS[self.as_index()]
    }
}

/// サンプラーのフィルタータイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum EffectSamplerFilterType {
    #[default]
    Linear,
    Point,
}

/// サンプラーのアドレスタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum EffectSamplerAddressType {
    #[default]
    Clamp,
    Wrap,
}

/// サンプラー記述子
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EffectSamplerDesc {
    pub name: String,
    pub filter_type: EffectSamplerFilterType,
    pub address_type: EffectSamplerAddressType,
}

/// テクスチャ記述子
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EffectIntermediateTextureDesc {
    pub name: String,
    pub format: EffectIntermediateTextureFormat,
    pub size_expr: (String, String),
    pub source: String,
}

/// パラメータ定数 (float または int)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectConstant {
    Float {
        default_value: f32,
        min_value: f32,
        max_value: f32,
        step: f32,
    },
    Int {
        default_value: i32,
        min_value: i32,
        max_value: i32,
        step: i32,
    },
}

impl Default for EffectConstant {
    fn default() -> Self {
        Self::Float {
            default_value: 0.0,
            min_value: 0.0,
            max_value: 1.0,
            step: 0.01,
        }
    }
}

/// パラメータ記述子
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EffectParameterDesc {
    pub name: String,
    pub label: String,
    pub constant: EffectConstant,
}

/// パスフラグ
pub mod effect_pass_flags {
    pub const PS_STYLE: u32 = 1;
}

/// パス記述子
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EffectPassDesc {
    pub inputs: Vec<usize>,
    pub outputs: Vec<usize>,
    pub num_threads: [u32; 3],
    pub block_size: (u32, u32),
    pub desc: String,
    pub flags: u32,
    pub cso: Option<Vec<u8>>,
}

impl EffectPassDesc {
    pub fn is_ps_style(&self) -> bool {
        (self.flags & effect_pass_flags::PS_STYLE) != 0
    }
}

/// エフェクトフラグ
pub mod effect_flags {
    pub const USE_DYNAMIC: u32 = 1;
    pub const USE_MUL_ADD: u32 = 1 << 1;
    pub const SUPPORT_FP16: u32 = 1 << 2;
    pub const INLINE_PARAMS: u32 = 1 << 16;
    pub const FP16: u32 = 1 << 17;
}

/// エフェクト記述子
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EffectDesc {
    pub name: String,
    pub sort_name: String,
    pub params: Vec<EffectParameterDesc>,
    pub textures: Vec<EffectIntermediateTextureDesc>,
    pub samplers: Vec<EffectSamplerDesc>,
    pub passes: Vec<EffectPassDesc>,
    pub flags: u32,
    pub common_code: String,
}

impl EffectDesc {
    /// 新しいエフェクト記述子を作成
    pub fn new(name: impl Into<String>) -> Self {
        let mut desc = Self {
            name: name.into(),
            ..Default::default()
        };

        // INPUT テクスチャを追加 (インデックス 0)
        desc.textures.push(EffectIntermediateTextureDesc {
            name: "INPUT".to_string(),
            format: EffectIntermediateTextureFormat::R8G8B8A8Unorm,
            size_expr: ("INPUT_WIDTH".to_string(), "INPUT_HEIGHT".to_string()),
            ..Default::default()
        });

        // OUTPUT テクスチャを追加 (インデックス 1)
        desc.textures.push(EffectIntermediateTextureDesc {
            name: "OUTPUT".to_string(),
            format: EffectIntermediateTextureFormat::R8G8B8A8Unorm,
            ..Default::default()
        });

        desc
    }

    /// OUTPUT のサイズ式を取得
    pub fn get_output_size_expr(&self) -> &(String, String) {
        &self.textures[1].size_expr
    }
}
