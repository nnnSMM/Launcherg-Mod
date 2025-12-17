
// Bilinear Interpolation Shader

//!MAGPIE EFFECT
//!VERSION 4

//!TEXTURE
Texture2D INPUT;
Texture2D OUTPUT;

//!PASS 1
//!IN INPUT
//!OUT OUTPUT
//!BLOCK_SIZE 16
//!NUM_THREADS 16, 16, 1

//!STYLE PS
//!SAMPLER 
//!FILTER LINEAR
//!ADDRESS CLAMP
SamplerState sam;

float4 Pass1(float2 pos) {
    return INPUT.SampleLevel(sam, pos, 0);
}
