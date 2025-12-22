use crate::infrastructure::windowsimpl::scaling::shader::ShaderManager;
use crate::infrastructure::windowsimpl::scaling::effect_runtime::{EffectRuntime, MagpieConstants};
use crate::infrastructure::windowsimpl::scaling::window::WindowManager;
// use crate::infrastructure::windowsimpl::scaling::shared_resources::get_shared_d3d_device; // Reverted
use crate::infrastructure::windowsimpl::screenshot::d3d::create_d3d_device;
use anyhow::{anyhow, Result};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;
use windows::core::ComInterface;
use windows::Foundation::TypedEventHandler;
use windows::Graphics::Capture::{
    Direct3D11CaptureFrame, Direct3D11CaptureFramePool, GraphicsCaptureItem, GraphicsCaptureSession,
};
use windows::Graphics::DirectX::DirectXPixelFormat;

use windows::Win32::Graphics::Direct3D11::{
    ID3D11Buffer, ID3D11ComputeShader, ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView,
    ID3D11UnorderedAccessView, D3D11_BIND_CONSTANT_BUFFER, D3D11_BIND_SHADER_RESOURCE,
    D3D11_BIND_UNORDERED_ACCESS, D3D11_BUFFER_DESC, D3D11_CPU_ACCESS_WRITE,
    D3D11_MAPPED_SUBRESOURCE, D3D11_MAP_WRITE_DISCARD, D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT,
    D3D11_USAGE_DYNAMIC, ID3D11ShaderResourceView, ID3D11Texture2D,
};
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_ALPHA_MODE_IGNORE, DXGI_FORMAT_R8G8B8A8_UNORM,
    DXGI_SAMPLE_DESC,
};
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWINDOWATTRIBUTE};
use windows::Win32::Graphics::Dxgi::{
    IDXGIDevice, IDXGIFactory2, IDXGISwapChain1, DXGI_SWAP_CHAIN_DESC1,
    DXGI_SWAP_EFFECT_FLIP_DISCARD, DXGI_USAGE_RENDER_TARGET_OUTPUT, DXGI_USAGE_UNORDERED_ACCESS
};
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
use windows::Win32::System::Threading::{
    CreateEventW, GetCurrentThreadId, SetEvent,
};
use windows::Win32::System::WinRT::Direct3D11::{
    CreateDirect3D11DeviceFromDXGIDevice, IDirect3DDxgiInterfaceAccess,
};
use windows::Win32::System::WinRT::Graphics::Capture::IGraphicsCaptureItemInterop;
use windows::Win32::System::WinRT::{RoInitialize, RO_INIT_SINGLETHREADED};
use windows::Win32::Foundation::{CloseHandle, HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, PeekMessageW, MSG, PM_REMOVE,
    QS_ALLINPUT, WM_QUIT, MsgWaitForMultipleObjectsEx, MWMO_INPUTAVAILABLE, GetWindowRect,
    GetWindowLongPtrW, SetWindowLongPtrW, GWLP_USERDATA,
};
use windows::Win32::Graphics::Gdi::ScreenToClient;
use crate::infrastructure::windowsimpl::scaling::src_tracker::SrcTracker;
use crate::infrastructure::windowsimpl::scaling::window::SharedWindowState;

// DWM constants for corner preference
const DWMWA_WINDOW_CORNER_PREFERENCE: DWMWINDOWATTRIBUTE = DWMWINDOWATTRIBUTE(33);
const DWMWCP_DEFAULT: u32 = 0;
const DWMWCP_DONOTROUND: u32 = 1;

#[derive(Debug, PartialEq, Eq)]
enum FrameSourceState {
    NewFrame,
    Waiting,
}

struct CaptureFrameSource {
    item: GraphicsCaptureItem,
    frame_pool: Option<Direct3D11CaptureFramePool>,
    session: Option<GraphicsCaptureSession>,
    current_frame: Option<Direct3D11CaptureFrame>,
    _last_size: (i32, i32),
    _device: ID3D11Device,
    winrt_device: windows::Graphics::DirectX::Direct3D11::IDirect3DDevice,
    pub frame_event: windows::Win32::Foundation::HANDLE,
}

impl CaptureFrameSource {
    fn new(hwnd: HWND, device: ID3D11Device) -> Result<Self> {
        let interop = windows::core::factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()?;
        let item = unsafe { interop.CreateForWindow(hwnd)? };

        // Create WinRT wrapper for D3D device
        let dxgi_device: windows::Win32::Graphics::Dxgi::IDXGIDevice = device.cast()?;
        let inspectable = unsafe { CreateDirect3D11DeviceFromDXGIDevice(&dxgi_device)? };
        let winrt_device: windows::Graphics::DirectX::Direct3D11::IDirect3DDevice =
            inspectable.cast()?;

        // Create Auto-reset event
        let frame_event = unsafe { CreateEventW(None, false, false, None)? };

        Ok(Self {
            item,
            frame_pool: None,
            session: None,
            current_frame: None,
            _last_size: (0, 0),
            _device: device,
            winrt_device,
            frame_event,
        })
    }

    fn start(&mut self) -> Result<()> {
        let item_size = self.item.Size()?;

        self.frame_pool = Some(Direct3D11CaptureFramePool::Create(
            &self.winrt_device,
            DirectXPixelFormat::B8G8R8A8UIntNormalized,
            4, // Magpie uses 4 buffers to help with latency at low frame rates
            item_size,
        )?);

        let pool = self.frame_pool.as_ref().unwrap();

        // Define FrameArrived handler
        let event_handle = self.frame_event;
        pool.FrameArrived(&TypedEventHandler::new(move |_, _| {
            unsafe { let _ = SetEvent(event_handle); };
            Ok(())
        }))?;

        self.session = Some(pool.CreateCaptureSession(&self.item)?);

        let session = self.session.as_ref().unwrap();
        // Disable cursor capture if possible
        if let Ok(result) = session.IsCursorCaptureEnabled() {
            if result {
                let _ = session.SetIsCursorCaptureEnabled(false);
            }
        }
        // Disable yellow border (WGC)
        let _ = session.SetIsBorderRequired(false);

        session.StartCapture()?;
        Ok(())
    }

    fn stop(&mut self) {
        if let Some(session) = &self.session {
            let _ = session.Close();
        }
        if let Some(pool) = &self.frame_pool {
            let _ = pool.Close();
        }
        self.session = None;
        self.frame_pool = None;
        if !self.frame_event.is_invalid() {
            unsafe {
                let _ = CloseHandle(self.frame_event);
                self.frame_event = windows::Win32::Foundation::HANDLE::default();
            }
        }
    }

    fn update(&mut self) -> Result<FrameSourceState> {
        let pool = match &self.frame_pool {
            Some(p) => p,
            None => return Ok(FrameSourceState::Waiting), // Should be Error?
        };

        let mut frame = match pool.TryGetNextFrame() {
            Ok(f) => f,
            Err(_) => return Ok(FrameSourceState::Waiting),
        };

        // Magpie Logic: Drain the queue to get the very latest frame
        // This reduces latency significantly if frames are piling up
        while let Ok(next_frame) = pool.TryGetNextFrame() {
            frame = next_frame;
        }

        // Store the frame
        self.current_frame = Some(frame);

        Ok(FrameSourceState::NewFrame)
    }

    fn get_texture(&self) -> Result<ID3D11Texture2D> {
        let frame = self.current_frame.as_ref().ok_or(anyhow!("No frame"))?;
        let surface = frame.Surface()?;
        let access: IDirect3DDxgiInterfaceAccess = surface.cast()?;
        let texture: ID3D11Texture2D = unsafe { access.GetInterface()? };
        Ok(texture)
    }
}

pub struct ScalingProcessor {
    running: Arc<AtomicBool>,
    thread_id: Arc<AtomicU32>,

    device: Option<ID3D11Device>,
    context: Option<ID3D11DeviceContext>,
    shader_dir: String,
    _shader_manager: Option<ShaderManager>,
    _window_manager: Option<WindowManager>,
    _compute_shader: Option<ID3D11ComputeShader>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl ScalingProcessor {
    pub fn new(shader_dir: String) -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            thread_id: Arc::new(AtomicU32::new(0)),
            device: None,
            context: None,
            shader_dir,
            _shader_manager: None,
            _window_manager: None,
            _compute_shader: None,
            thread_handle: None,
        }
    }

    pub fn start(&mut self, target_hwnd: HWND, shader_name: String) -> Result<()> {
        let running = self.running.clone();
        let thread_id_atomic = self.thread_id.clone();
        let target_hwnd_val = target_hwnd.0 as u64;
        let shader_dir = self.shader_dir.clone();

        self.running.store(true, Ordering::SeqCst);

        let handle = thread::spawn(move || {
            let target_hwnd = HWND(target_hwnd_val as isize);
            println!("Backend thread started. HWND: {:?}", target_hwnd);

            unsafe {
                if let Err(e) = RoInitialize(RO_INIT_SINGLETHREADED) {
                    println!("RoInitialize failed: {:?}", e);
                    return;
                }
                if let Err(e) = CoInitializeEx(None, COINIT_APARTMENTTHREADED) {
                    println!("CoInitializeEx failed: {:?}", e);
                }
            }

            thread_id_atomic.store(unsafe { GetCurrentThreadId() }, Ordering::SeqCst);

            let init_result = (|| -> Result<(WindowManager, ID3D11Device, ID3D11DeviceContext, IDXGISwapChain1, windows::Win32::Foundation::HANDLE, ID3D11Texture2D, ID3D11RenderTargetView, EffectRuntime, ID3D11Buffer, ID3D11Texture2D, ID3D11UnorderedAccessView, crate::infrastructure::windowsimpl::scaling::toolbar::Toolbar, crate::infrastructure::windowsimpl::scaling::cursor::CursorManager, crate::infrastructure::windowsimpl::scaling::cursor_renderer::CursorRenderer, crate::infrastructure::windowsimpl::scaling::overlay::SimpleToolbar, windows::Win32::Graphics::Direct3D11::ID3D11SamplerState, CaptureFrameSource)> {
                // 1. Start Device Creation in separate thread (Async)
                let device_handle = thread::spawn(|| -> Result<ID3D11Device> {
                    Ok(create_d3d_device()?)
                });
                
                // 2. Main Thread: WindowManager & Shaders
                let mut window_manager = WindowManager::new();
                window_manager.prepare_target_window(target_hwnd)?;
                window_manager.create_overlay_window()?;
                let overlay_window = window_manager
                    .get_overlay_window()
                    .ok_or(anyhow!("No overlay window"))?;

                // Setup SharedWindowState for Hit-Testing
                let shared_state = Box::new(SharedWindowState {
                    toolbar_rect: windows::Win32::Foundation::RECT::default(),
                    is_visible: false,
                });
                let state_ptr = Box::into_raw(shared_state);
                unsafe {
                    SetWindowLongPtrW(overlay_window, GWLP_USERDATA, state_ptr as isize);
                }

                let shader_path_buf = std::path::PathBuf::from(shader_dir);

                // ShaderManager no longer needs Device!
                let shader_manager = ShaderManager::new(&shader_path_buf);
                let shade_file_name = if shader_name.ends_with(".hlsl") {
                    shader_name.clone()
                } else {
                    format!("{}.hlsl", shader_name)
                };
                let full_shader_path = shader_path_buf.join(shade_file_name);
                
                let compiled_effect = shader_manager.compile_effect(&full_shader_path)
                        .map_err(|e| anyhow!("Shader compilation failed: {:?}", e))?;

                // 3. Join Device
                let device = device_handle.join().map_err(|_| anyhow!("Device thread panicked"))??;
                
                // Early Capture Start (Optimization)
                // Initialize capture while other resources are being created
                let mut source = CaptureFrameSource::new(target_hwnd, device.clone())?;
                source.start()?;
                
                // 4. Continue with Device Dependent Init
                let context = unsafe { device.GetImmediateContext()? };
                unsafe {
                    let feature_level = device.GetFeatureLevel();
                    println!("D3D Feature Level: {:?}", feature_level);
                }

                let dxgi_device: IDXGIDevice = device.cast()?;
                let adapter = unsafe { dxgi_device.GetAdapter()? };
                let factory: IDXGIFactory2 = unsafe { adapter.GetParent()? };

                let mut rect = windows::Win32::Foundation::RECT::default();
                unsafe { let _ = windows::Win32::UI::WindowsAndMessaging::GetClientRect(overlay_window, &mut rect); };
                let width = (rect.right - rect.left) as u32;
                let height = (rect.bottom - rect.top) as u32;

                let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
                    Width: width,
                    Height: height,
                    Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                    Stereo: false.into(),
                    SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                    BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT | DXGI_USAGE_UNORDERED_ACCESS,
                    BufferCount: 2,
                    Scaling: windows::Win32::Graphics::Dxgi::DXGI_SCALING_STRETCH,
                    SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                    AlphaMode: DXGI_ALPHA_MODE_IGNORE,
                    Flags: windows::Win32::Graphics::Dxgi::DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT.0 as u32,
                };

                let swap_chain: IDXGISwapChain1 = unsafe {
                    factory.CreateSwapChainForHwnd(&device, overlay_window, &swap_chain_desc, None, None)
                        .map_err(|e| anyhow!("CreateSwapChainForHwnd failed: {:?}", e))?
                };

                // フレームレイテンシ制御 (Magpie方式)
                let swap_chain2: windows::Win32::Graphics::Dxgi::IDXGISwapChain2 = swap_chain.cast()?;
                unsafe { swap_chain2.SetMaximumFrameLatency(1)? };
                let frame_latency_handle = unsafe { swap_chain2.GetFrameLatencyWaitableObject() };

                // バックバッファとRTVをキャッシュ (Magpie方式: 毎フレームRTV作成を避ける)
                let cached_backbuffer: ID3D11Texture2D = unsafe { swap_chain.GetBuffer(0)? };
                let mut cached_rtv_out = None;
                unsafe { device.CreateRenderTargetView(&cached_backbuffer, None, Some(&mut cached_rtv_out))? };
                let cached_rtv = cached_rtv_out.ok_or_else(|| anyhow!("Failed to create cached RTV"))?;

                // Create Intermediate Texture (for CS output)
                let output_texture_desc = D3D11_TEXTURE2D_DESC {
                    Width: width,
                    Height: height,
                    MipLevels: 1,
                    ArraySize: 1,
                    Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                    SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                    Usage: D3D11_USAGE_DEFAULT,
                    BindFlags: (D3D11_BIND_SHADER_RESOURCE.0 | D3D11_BIND_UNORDERED_ACCESS.0) as u32,
                    CPUAccessFlags: 0,
                    MiscFlags: 0,
                };
                
                let mut output_texture_out = None;
                unsafe { device.CreateTexture2D(&output_texture_desc, None, Some(&mut output_texture_out))? };
                let output_texture = output_texture_out.unwrap();
                
                let mut output_uav_out = None;
                unsafe { device.CreateUnorderedAccessView(&output_texture, None, Some(&mut output_uav_out))? };
                let _output_uav = output_uav_out.unwrap();

                // Effect Runtime Init (Requires Device)
                let effect_runtime = EffectRuntime::new(device.clone(), context.clone(), compiled_effect)?;

                let const_buffer = unsafe {
                    let desc = D3D11_BUFFER_DESC {
                        ByteWidth: ((std::mem::size_of::<MagpieConstants>() + 15) & !15) as u32,
                        Usage: D3D11_USAGE_DYNAMIC,
                        BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as u32,
                        CPUAccessFlags: D3D11_CPU_ACCESS_WRITE.0 as u32,
                        MiscFlags: 0,
                        StructureByteStride: 0,
                    };
                    let mut buffer = None;
                    device.CreateBuffer(&desc, None, Some(&mut buffer))?;
                    buffer.unwrap()
                };

                // Toolbar Init
                let mut toolbar = crate::infrastructure::windowsimpl::scaling::toolbar::Toolbar::new()?;
                let cursor_manager = crate::infrastructure::windowsimpl::scaling::cursor::CursorManager::new(overlay_window);
                let cursor_renderer = crate::infrastructure::windowsimpl::scaling::cursor_renderer::CursorRenderer::new(device.clone(), context.clone());
                let simple_toolbar = crate::infrastructure::windowsimpl::scaling::overlay::SimpleToolbar::new(device.clone(), context.clone());
                toolbar.active_algorithm = shader_name.clone();

                // Sampler State (Border Black for Aspect Ratio)
                let sampler_desc = windows::Win32::Graphics::Direct3D11::D3D11_SAMPLER_DESC {
                    Filter: windows::Win32::Graphics::Direct3D11::D3D11_FILTER_MIN_MAG_MIP_LINEAR,
                    AddressU: windows::Win32::Graphics::Direct3D11::D3D11_TEXTURE_ADDRESS_BORDER,
                    AddressV: windows::Win32::Graphics::Direct3D11::D3D11_TEXTURE_ADDRESS_BORDER,
                    AddressW: windows::Win32::Graphics::Direct3D11::D3D11_TEXTURE_ADDRESS_BORDER,
                    MipLODBias: 0.0,
                    MaxAnisotropy: 1,
                    ComparisonFunc: windows::Win32::Graphics::Direct3D11::D3D11_COMPARISON_NEVER,
                    BorderColor: [0.0, 0.0, 0.0, 0.0],
                    MinLOD: 0.0,
                    MaxLOD: windows::Win32::Graphics::Direct3D11::D3D11_FLOAT32_MAX,
                };
                let mut sampler_out = None;
                unsafe { device.CreateSamplerState(&sampler_desc, Some(&mut sampler_out))? };
                let sampler = sampler_out.unwrap();

                Ok((window_manager, device, context, swap_chain, frame_latency_handle, cached_backbuffer, cached_rtv, effect_runtime, const_buffer, output_texture, _output_uav, toolbar, cursor_manager, cursor_renderer, simple_toolbar, sampler, source))
            })();

            match init_result {
                Ok((mut window_manager, device, context, swap_chain, frame_latency_handle, _cached_backbuffer_initial, _cached_rtv_initial, mut effect_runtime, const_buffer, mut output_texture, _output_uav, mut toolbar, mut cursor_manager, mut cursor_renderer, mut simple_toolbar, sampler, mut source)) => {
                    println!("Initialization successful");
                    
                    // ソースウィンドウの位置を保存（終了時に復元するため）
                    let saved_src_rect = unsafe {
                        let mut rect = RECT::default();
                        let _ = GetWindowRect(target_hwnd, &mut rect);
                        rect
                    };
                    
                    let mut src_tracker = SrcTracker::new(target_hwnd);
                    let _ = src_tracker.update(); 
                    
                    // Capture already started
                    {
                            println!("Capture started");
                            
                            let mut last_time = std::time::Instant::now();
                            let mut frames = 0;
                            let mut total_processing_time = 0.0;
                            let mut processing_start;

                            // Aspect Ratio & Centering State
                            let mut current_target_w = 0.0f32;
                            let mut current_target_h = 0.0f32;
                            let mut current_offset_x = 0u32;
                            let mut current_offset_y = 0u32;
                            
                            let mut intermediate_texture: Option<ID3D11Texture2D> = None;
                            let mut using_intermediate = false;
                            
                            // Input Crop Texture (for removing window borders/shadows before shader pass)
                            let mut input_crop_texture: Option<ID3D11Texture2D> = None;

                            // 1. Disable Rounded Corners if possible (Windows 11)
                            unsafe {
                                let preference = DWMWCP_DONOTROUND;
                                let _ = DwmSetWindowAttribute(
                                    target_hwnd,
                                    DWMWA_WINDOW_CORNER_PREFERENCE,
                                    &preference as *const _ as *const std::ffi::c_void,
                                    std::mem::size_of::<u32>() as u32,
                                );
                            }
                            
                            let mut first_frame_presented = false;
                            
                            // --- Caching Variables ---
                            let mut cached_backbuffer_rtv: Option<ID3D11RenderTargetView> = None;
                            let mut cached_input_srv: Option<ID3D11ShaderResourceView> = None;
                            let mut last_input_texture_ptr: Option<*mut std::ffi::c_void> = None;

                            loop {
                                unsafe {
                                    let mut msg = MSG::default();
                                    while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                                        if msg.message == WM_QUIT {
                                            running.store(false, Ordering::SeqCst);
                                        }
                                        DispatchMessageW(&msg);
                                    }
                                }

                                // Loop Condition Check
                                if !running.load(Ordering::SeqCst) { break; }




                                // Acquire BackBuffer (Waitable SwapChain)
                                // Note: GetBuffer is cheap, but CreateRTV is not.
                                // We need to check if we can reuse the cached RTV.
                                // For flip model, backbuffer index changes. But simple way is to just get buffer.
                                // IMPORTANT: In FLIP_DISCARD, the content is discarded, but the interface pointer *might* change or swap?
                                // Actually, SwapChain returns different headers for different buffers.
                                // However, simple optimization: Just create RTV *once* if swapchain buffers don't change?
                                // In FLIP model, we have BufferCount=2.
                                // We can cache RTVs for each buffer? Or just recreate RTV if backbuffer pointer changes?
                                // Let's try simple caching: If GetBuffer returns same pointer, reuse RTV.
                                
                                let backbuffer: ID3D11Texture2D = match unsafe { swap_chain.GetBuffer(0) } {
                                    Ok(b) => b,
                                    Err(e) => {
                                        println!("Failed to get backbuffer: {:?}", e);
                                        continue;
                                    }
                                };
                                
                                // RTV Cache Logic
                                // For now, let's keep it simple: Re-create RTV every frame IS standard for FLIP models if you don't manage a pool.
                                // However, keeping 2 RTVs (one for each buffer) is better.
                                // But since we are modifying `processor.rs`, let's try to just reuse if possible or accept this cost for now if it's tricky.
                                // Wait, the plan was to cache. 
                                // D3D11 SwapChain GetBuffer(0) always returns the *current* back buffer.
                                // In SwapEffect::FlipDiscard, the buffer index rotates.
                                // So the Ptr changes every frame (0 -> 1 -> 0 ...).
                                // Optimally we should map BufferPtr -> RTV.
                                // But for this step, let's just create it, but pass it to renderer so renderer doesn't create it AGAIN.
                                // The optimization in cursor_renderer was to avoid creating it *twice* (once here, once in draw_cursor).
                                // So even creating it here once is improvement.
                                
                                // Let's try to implement a simple 2-slot cache? 
                                // A simple hashmap is unpredictable. 
                                // Let's just create it here.
                                
                                let mut rtv_out = None;
                                if let Err(e) = unsafe { device.CreateRenderTargetView(&backbuffer, None, Some(&mut rtv_out)) } {
                                    println!("Failed to create RTV: {:?}", e);
                                    continue;
                                }
                                let rtv = match rtv_out {
                                    Some(r) => r,
                                    None => continue,
                                };
                                    
                                    // 監視: ソースウィンドウ監視
                                unsafe {
                                    use windows::Win32::UI::WindowsAndMessaging::{IsWindow, IsWindowVisible, IsIconic};
                                    
                                    // ウィンドウが存在しない場合は終了
                                    if !IsWindow(target_hwnd).as_bool() {
                                        println!("Source window destroyed, stopping scaling");
                                        running.store(false, Ordering::SeqCst);
                                        break;
                                    }
                                    
                                    // ウィンドウが非表示または最小化された場合は終了
                                    if !IsWindowVisible(target_hwnd).as_bool() || IsIconic(target_hwnd).as_bool() {
                                        println!("Source window hidden or minimized, stopping scaling");
                                        running.store(false, Ordering::SeqCst);
                                        break;
                                    }
                                }

                                processing_start = std::time::Instant::now();

                                // Try to get new frame
                                let frame_state = source.update().unwrap_or(FrameSourceState::Waiting);

                                if frame_state == FrameSourceState::NewFrame {
                                    if let Ok(input_texture_raw) = source.get_texture() {
                                    // Use raw texture by default, but might be replaced by cropped one
                                    let mut input_texture = input_texture_raw.clone();
                                    
                                    unsafe {
                                        // 1. Get raw dimensions
                                        let mut desc = D3D11_TEXTURE2D_DESC::default();
                                        input_texture_raw.GetDesc(&mut desc);
                                        let raw_width = desc.Width;
                                        let raw_height = desc.Height;
                                        
                                        let mut input_width = raw_width;
                                        let mut input_height = raw_height;

                                        // 2. Update Tracker & Calculate Crop
                                        let _ = src_tracker.update();
                                        let source_rect = src_tracker.get_capture_rect();
                                        let frame_rect = src_tracker.get_frame_rect();

                                        let mut crop_x = (source_rect.left - frame_rect.left).max(0) as u32;
                                        let mut crop_y = (source_rect.top - frame_rect.top).max(0) as u32;
                                        let mut crop_w = (source_rect.right - source_rect.left).max(0) as u32;
                                        let mut crop_h = (source_rect.bottom - source_rect.top).max(0) as u32;
                                        
                                        // Safety Clamp
                                        crop_w = crop_w.min(raw_width - crop_x);
                                        crop_h = crop_h.min(raw_height - crop_y);
                                        
                                        // 3. Perform Crop if needed
                                        let mut border_left = 0.0f32;
                                        let mut border_top = 0.0f32;
                                        let mut region_w = crop_w as f32;
                                        let mut region_h = crop_h as f32;

                                        if crop_w > 0 && crop_h > 0 && (crop_w != raw_width || crop_h != raw_height || crop_x > 0 || crop_y > 0) {
                                            // Recreate crop texture if size changed
                                            let mut recreate = true;
                                            if let Some(ref t) = input_crop_texture {
                                                let mut d = D3D11_TEXTURE2D_DESC::default();
                                                t.GetDesc(&mut d);
                                                if d.Width == crop_w && d.Height == crop_h {
                                                    recreate = false;
                                                }
                                            }
                                            
                                            if recreate {
                                                let crop_desc = D3D11_TEXTURE2D_DESC {
                                                    Width: crop_w,
                                                    Height: crop_h,
                                                    MipLevels: 1,
                                                    ArraySize: 1,
                                                    Format: desc.Format,
                                                    SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                                                    Usage: D3D11_USAGE_DEFAULT,
                                                    BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
                                                    ..Default::default()
                                                };
                                                let mut tex = None;
                                                let _ = device.CreateTexture2D(&crop_desc, None, Some(&mut tex));
                                                input_crop_texture = tex;
                                            }
                                            
                                            // Copy Region
                                            if let Some(ref dst_tex) = input_crop_texture {
                                                let src_box = windows::Win32::Graphics::Direct3D11::D3D11_BOX {
                                                    left: crop_x,
                                                    top: crop_y,
                                                    front: 0,
                                                    right: crop_x + crop_w,
                                                    bottom: crop_y + crop_h,
                                                    back: 1,
                                                };
                                                context.CopySubresourceRegion(dst_tex, 0, 0, 0, 0, &input_texture_raw, 0, Some(&src_box));
                                                
                                                // Switch input
                                                input_texture = dst_tex.clone();
                                                input_width = crop_w;
                                                input_height = crop_h;
                                                
                                                // Reset borders for shader constants (since we cropped them out)
                                                // border_left / top are used for src_rect_offset.
                                                // Since we cropped, the new texture starts at (0,0) of content.
                                                border_left = 0.0;
                                                border_top = 0.0;
                                                region_w = input_width as f32;
                                                region_h = input_height as f32;
                                            }
                                        } else {
                                            // No crop needed (or invalid crop), use raw values
                                            // Still need to calculate border_left etc for shader constants if we didn't crop?
                                            // If we didn't crop, it means input IS raw.
                                            // Ideally we SHOULD have cropped if borders exist.
                                            // If crop logic failed (e.g. crop_w=0), we fallback to full texture.
                                            // In that case, border_left should technically be calculated...
                                            // But for safety let's use what we have.
                                            border_left = (source_rect.left - frame_rect.left).max(0) as f32;
                                            border_top = (source_rect.top - frame_rect.top).max(0) as f32;
                                            region_w = (source_rect.right - source_rect.left) as f32;
                                            region_h = (source_rect.bottom - source_rect.top) as f32;
                                        }

                                        // SRV Caching for Input
                                        let mut input_srv = None;
                                        
                                        // TODO: Implement proper caching comparing texture pointers
                                        // For now, just create new SRV (Safety first)
                                        let _ = device.CreateShaderResourceView(&input_texture, None, Some(&mut input_srv));
 
                                        if let Some(_input_srv) = input_srv {
                                            
                                            // Output Size = SwapChain Size (Full Screen)
                                            let mut output_desc = D3D11_TEXTURE2D_DESC::default();
                                            output_texture.GetDesc(&mut output_desc);
                                            let full_output_width = output_desc.Width;
                                            let full_output_height = output_desc.Height;
                                            let cap_w = input_width as f32;
                                            let cap_h = input_height as f32;

                                            // UV Offsets & Scales
                                            let offset_u = if cap_w > 0.0 { border_left as f32 / cap_w } else { 0.0 };
                                            let offset_v = if cap_h > 0.0 { border_top as f32 / cap_h } else { 0.0 };
                                            let scale_u = if cap_w > 0.0 { region_w / cap_w } else { 1.0 };
                                            let scale_v = if cap_h > 0.0 { region_h / cap_h } else { 1.0 };

                                            // Aspect Ratio Preservation Logic
                                            let src_w = (source_rect.right - source_rect.left) as f32;
                                            let src_h = (source_rect.bottom - source_rect.top) as f32;
                                            
                                            let mut target_w = full_output_width as f32;
                                            let mut target_h = full_output_height as f32;

                                            if src_w > 0.0 && src_h > 0.0 {
                                                let scale_x = full_output_width as f32 / src_w;
                                                let scale_y = full_output_height as f32 / src_h;
                                                let scale = scale_x.min(scale_y);

                                                target_w = (src_w * scale).round();
                                                target_h = (src_h * scale).round();
                                            }

                                            // Centering offsets
                                            let offset_x = ((full_output_width as f32 - target_w) / 2.0).round() as u32;
                                            let offset_y = ((full_output_height as f32 - target_h) / 2.0).round() as u32;
                                            
                                            // Update State for Cursor
                                            current_target_w = target_w;
                                            current_target_h = target_h;
                                            current_offset_x = offset_x;
                                            current_offset_y = offset_y;

                                            // Preferred Output Size Calculation
                                            let mut output_width = target_w as u32;
                                            let mut output_height = target_h as u32;
                                            using_intermediate = false;

                                            if let Ok((pref_w, pref_h)) = effect_runtime.get_preferred_output_size((input_width, input_height), (output_width, output_height)) {
                                                if pref_w != output_width || pref_h != output_height {
                                                    using_intermediate = true;
                                                    output_width = pref_w;
                                                    output_height = pref_h;

                                                    // Check if we need to recreate intermediate texture
                                                    let mut recreate = true;
                                                    if let Some(ref tex) = intermediate_texture {
                                                        let mut desc = D3D11_TEXTURE2D_DESC::default();
                                                        unsafe { tex.GetDesc(&mut desc) };
                                                        if desc.Width == output_width && desc.Height == output_height {
                                                            recreate = false;
                                                        }
                                                    }

                                                    if recreate {
                                                        let desc = D3D11_TEXTURE2D_DESC {
                                                            Width: output_width,
                                                            Height: output_height,
                                                            MipLevels: 1,
                                                            ArraySize: 1,
                                                            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                                                            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                                                            Usage: D3D11_USAGE_DEFAULT,
                                                            BindFlags: (D3D11_BIND_SHADER_RESOURCE.0 | D3D11_BIND_UNORDERED_ACCESS.0) as u32, // UAV for CS output, SRV for Blit input
                                                            ..Default::default()
                                                        };
                                                        let mut tex = None;
                                                        unsafe { device.CreateTexture2D(&desc, None, Some(&mut tex)).unwrap() }; // Handle error properly in prod
                                                        intermediate_texture = tex;
                                                    }
                                                }
                                            }

                                            // Effect Runtime Update
                                            // output_width/height now reflects the ACTUAL render resolution (Target or Preferred)
                                            let _ = effect_runtime.update_size((input_width, input_height), (output_width, output_height));
                                            let _ = effect_runtime.set_input_texture(&input_texture);
                                            
                                            if using_intermediate {
                                                if let Some(ref tex) = intermediate_texture {
                                                        let _ = effect_runtime.set_output_texture(tex);
                                                }
                                            } else {
                                                let _ = effect_runtime.set_output_texture(&output_texture);
                                            }

                                            let pass_infos = effect_runtime.get_pass_infos();

                                            // 定数バッファを更新
                                            let constants = MagpieConstants {
                                                input_size: [input_width, input_height],
                                                output_size: [output_width, output_height], 
                                                input_pt: [1.0 / input_width as f32, 1.0 / input_height as f32],
                                                output_pt: [1.0 / output_width as f32, 1.0 / output_height as f32], // Output PT based on Render Size
                                                scale: [scale_u, scale_v],
                                            };

                                            let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
                                            if context.Map(&const_buffer, 0, D3D11_MAP_WRITE_DISCARD, 0, Some(&mut mapped)).is_ok() {
                                                let ptr = mapped.pData as *mut MagpieConstants;
                                                ptr.write(constants);
                                                context.Unmap(&const_buffer, 0);
                                            }

                                            if let Err(e) = effect_runtime.execute(&const_buffer) {
                                                println!("Effect execution failed: {:?}", e);
                                            }
                                            
                                            // effect_runtime already handles resource binding and unbinding
                                            
                                            // Calculate Timing
                                            let elapsed = processing_start.elapsed().as_secs_f32() * 1000.0;
                                            total_processing_time += elapsed;

                                            frames += 1;
                                        }
                                    }
                                }
                                }

                                // Update FPS & Stats (Every ~0.2s)
                                let elapsed_chk = last_time.elapsed().as_secs_f32();
                                let mut _stats_updated = false;
                                if elapsed_chk >= 0.2 {
                                    toolbar.fps = (frames as f32 / elapsed_chk) as u32;
                                    toolbar.processing_time_ms = if frames > 0 {
                                        total_processing_time / frames as f32
                                    } else {
                                        0.0
                                    };
                                    
                                    frames = 0;
                                    total_processing_time = 0.0;
                                    last_time = std::time::Instant::now();
                                    _stats_updated = true;
                                }

                                // Always Present to consume frame latency signal and keep cursor smooth
                                unsafe {
                                    // 1. Clear Backbuffer (Every Frame)
                                    let black_color = [0.0, 0.0, 0.0, 1.0];
                                    context.ClearRenderTargetView(&rtv, &black_color);

                                    // 2. Draw Last Scaled Frame (Every Frame)
                                    // 2. Draw Last Scaled Frame (Every Frame)
                                    if current_target_w > 0.0 && current_target_h > 0.0 {
                                        if using_intermediate {
                                            if let Some(ref tex) = intermediate_texture {
                                                let dst_rect = RECT {
                                                    left: current_offset_x as i32,
                                                    top: current_offset_y as i32,
                                                    right: current_offset_x as i32 + current_target_w as i32,
                                                    bottom: current_offset_y as i32 + current_target_h as i32,
                                                };
                                                if let Err(e) = effect_runtime.execute_blit(tex, &backbuffer, dst_rect, &sampler) {
                                                    println!("Blit failed: {:?}", e);
                                                }
                                            }
                                        } else {
                                            let dst_rect = windows::Win32::Foundation::RECT {
                                                left: current_offset_x as i32,
                                                top: current_offset_y as i32,
                                                right: (current_offset_x + current_target_w as u32) as i32,
                                                bottom: (current_offset_y + current_target_h as u32) as i32,
                                            };
                                            if let Err(e) = effect_runtime.execute_blit(
                                                &output_texture,
                                                &backbuffer,
                                                dst_rect,
                                                &sampler
                                            ) {
                                                println!("Blit failed: {:?}", e);
                                            }
                                        }
                                    }

                                    // Get Output desc again for size (TODO: optimize)
                                    let mut output_desc = D3D11_TEXTURE2D_DESC::default();
                                    output_texture.GetDesc(&mut output_desc);

                                    // Update Cursor (毎フレーム)
                                    let mut src_rect = RECT::default();
                                    let mut dest_rect = RECT::default();
                                    let _ = GetWindowRect(target_hwnd, &mut src_rect);
                                    if let Some(hwnd) = window_manager.get_overlay_window() {
                                        let _ = GetWindowRect(hwnd, &mut dest_rect);
                                    }
                                    
                                    // Ensure dest_rect is valid (sometimes fails on first frame)
                                    // current_target_w/current_target_hも有効でないとdest_rectが正しくない
                                    if (dest_rect.right - dest_rect.left) > 0 && current_target_w > 0.0 && current_target_h > 0.0 {
                                        
                                        // Calculate Valid Dest Rect (Inner Image Area)
                                        let mut valid_dest_rect = dest_rect;
                                        valid_dest_rect.left += current_offset_x as i32;
                                        valid_dest_rect.top += current_offset_y as i32;
                                        valid_dest_rect.right = valid_dest_rect.left + current_target_w as i32;
                                        valid_dest_rect.bottom = valid_dest_rect.top + current_target_h as i32;

                                        // SrcTrackerからフォーカス状態を取得
                                        let is_src_focused = src_tracker.is_focused();
                                        let overlay_hwnd = window_manager.get_overlay_window().unwrap_or(HWND::default());
                                        cursor_manager.update(target_hwnd, src_rect, valid_dest_rect, dest_rect, false, false);
                                        
                                        // ツールバー更新と描画 (カーソルより先に描画)
                                        let mut toolbar_cursor_pos = cursor_manager.draw_pos();
                                        if let Some(hwnd) = window_manager.get_overlay_window() {
                                            let _ = ScreenToClient(hwnd, &mut toolbar_cursor_pos);
                                        }
                                        simple_toolbar.tick();
                                        
                                        // simple_toolbar.update_visibility(toolbar_cursor_pos, (dest_rect.bottom - dest_rect.top) as i32);
                                        // Use valid_dest_rect for content-relative positioning
                                        simple_toolbar.update_visibility(toolbar_cursor_pos, &valid_dest_rect);
                                        let _ = simple_toolbar.render(&backbuffer, &valid_dest_rect);
                                        
                                        // 終了ボタンチェック (左クリック)
                                        if (windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(0x01) as u16 & 0x8000) != 0 {
                                            if simple_toolbar.check_close_button_click(toolbar_cursor_pos) {
                                                println!("Close button clicked, stopping scaling");
                                                running.store(false, Ordering::SeqCst);
                                                break;
                                            }
                                        }

                                        // Update SharedWindowState removed

                                        // システムカーソルを描画 (ツールバーの上に)
                                        if cursor_manager.should_draw_cursor() {
                                            let mut client_pt = cursor_manager.draw_pos();
                                            if let Some(hwnd) = window_manager.get_overlay_window() {
                                                let _ = ScreenToClient(hwnd, &mut client_pt);
                                            }
                                            
                                            // カーソルハンドルを取得して描画
                                            let mut cursor_info = windows::Win32::UI::WindowsAndMessaging::CURSORINFO {
                                                cbSize: std::mem::size_of::<windows::Win32::UI::WindowsAndMessaging::CURSORINFO>() as u32,
                                                ..Default::default()
                                            };
                                            if windows::Win32::UI::WindowsAndMessaging::GetCursorInfo(&mut cursor_info).is_ok() 
                                                && !cursor_info.hCursor.is_invalid() {
                                                // シザーレクトを設定してコンテンツ領域内にカーソルをクリップ
                                                let scissor_rect = windows::Win32::Foundation::RECT {
                                                    left: current_offset_x as i32,
                                                    top: current_offset_y as i32,
                                                    right: current_offset_x as i32 + current_target_w as i32,
                                                    bottom: current_offset_y as i32 + current_target_h as i32,
                                                };
                                                context.RSSetScissorRects(Some(&[scissor_rect]));
                                                
                                                // リサイズカーソルをフィルタリング (Magpie方式)
                                                let filtered_cursor = cursor_manager.get_cursor_handle(cursor_info.hCursor.0 as isize);
                                                let h_cursor = windows::Win32::UI::WindowsAndMessaging::HCURSOR(filtered_cursor as _);
                                                let _ = cursor_renderer.draw_cursor(&backbuffer, &rtv, output_desc.Width, output_desc.Height, h_cursor, client_pt, 1.0, Some(scissor_rect));
                                                
                                                // シザーレクトをリセット（画面全体に戻す）
                                                let full_rect = windows::Win32::Foundation::RECT {
                                                    left: 0,
                                                    top: 0,
                                                    right: output_desc.Width as i32,
                                                    bottom: output_desc.Height as i32,
                                                };
                                                context.RSSetScissorRects(Some(&[full_rect]));
                                            }
                                        }
                                        
                                        let _ = (&toolbar, output_desc.Width, &rtv, &frame_latency_handle, is_src_focused, overlay_hwnd); // unused warning 回避
                                    }

                                    let _ = swap_chain.Present(0, 0);
                                    
                                    if !first_frame_presented {
                                        window_manager.show_overlay();
                                        first_frame_presented = true;
                                    }
                                }
                                
                                // Wait for Frame Latency Object (to sync with refresh rate) or New Frame Event
                                // Wait for Frame Latency Object (to sync with refresh rate) or New Frame Event or Window Message
                                // This replaces the busy loop!
                                unsafe {
                                    let handles = [frame_latency_handle, source.frame_event];
                                    MsgWaitForMultipleObjectsEx(Some(&handles), 1000, QS_ALLINPUT, MWMO_INPUTAVAILABLE);
                                }
                            }
                            
                            // Restore Rounded Corners
                            unsafe {
                                let preference = DWMWCP_DEFAULT;
                                let _ = DwmSetWindowAttribute(
                                    target_hwnd,
                                    DWMWA_WINDOW_CORNER_PREFERENCE,
                                    &preference as *const _ as *const std::ffi::c_void,
                                    std::mem::size_of::<u32>() as u32,
                                );
                            }


                            // Cursor cleanup: カーソルをスクリーン座標に戻す
                            cursor_manager.stop_capture_public();

                            let _ = source.stop();
                    }
                    
                    window_manager.restore_target_window();
                    
                    // オーバーレイウィンドウを明示的に破棄（ソースウィンドウへの影響を制御するため）
                    if let Some(overlay_hwnd) = window_manager.get_overlay_window() {
                        unsafe {
                            use windows::Win32::UI::WindowsAndMessaging::DestroyWindow;
                            let _ = DestroyWindow(overlay_hwnd);

                            // Free SharedWindowState
                            let ptr = GetWindowLongPtrW(overlay_hwnd, GWLP_USERDATA) as *mut SharedWindowState;
                            if !ptr.is_null() {
                                let _ = Box::from_raw(ptr); 
                            }
                        }
                    }
                    
                    // オーバーレイ破棄後にソースウィンドウの位置を復元
                    unsafe {
                        use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, SWP_NOZORDER, SWP_NOACTIVATE};
                        let _ = SetWindowPos(
                            target_hwnd,
                            None,
                            saved_src_rect.left,
                            saved_src_rect.top,
                            saved_src_rect.right - saved_src_rect.left,
                            saved_src_rect.bottom - saved_src_rect.top,
                            SWP_NOZORDER | SWP_NOACTIVATE,
                        );
                    }
                }
                Err(e) => println!("Backend Init Failed: {:?}", e),
            }
            
            println!("Backend thread stopped");
        });

        self.thread_handle = Some(handle);
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.thread_handle.take() {
            handle
                .join()
                .map_err(|_| anyhow!("Failed to join backend thread"))?;
        }
        println!("Scaling Processor Stopped");
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}
