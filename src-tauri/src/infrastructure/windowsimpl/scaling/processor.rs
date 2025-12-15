use crate::infrastructure::windowsimpl::scaling::shader::{MagpieConstants, ShaderManager};
use crate::infrastructure::windowsimpl::scaling::window::WindowManager;
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
    ID3D11Texture2D, ID3D11UnorderedAccessView, D3D11_BIND_CONSTANT_BUFFER, D3D11_BIND_SHADER_RESOURCE,
    D3D11_BIND_UNORDERED_ACCESS, D3D11_BUFFER_DESC, D3D11_CPU_ACCESS_WRITE,
    D3D11_MAPPED_SUBRESOURCE, D3D11_MAP_WRITE_DISCARD, D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT,
    D3D11_USAGE_DYNAMIC,
};
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_ALPHA_MODE_IGNORE, DXGI_ALPHA_MODE_PREMULTIPLIED, DXGI_FORMAT_B8G8R8A8_UNORM,
    DXGI_SAMPLE_DESC,
};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS, DWMWA_NCRENDERING_ENABLED};
use windows::Win32::Graphics::Dxgi::{
    IDXGIDevice, IDXGIFactory2, IDXGISwapChain1, DXGI_SWAP_CHAIN_DESC1,
    DXGI_SWAP_EFFECT_FLIP_DISCARD, DXGI_USAGE_RENDER_TARGET_OUTPUT,
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
    DispatchMessageW, GetMessageW, PeekMessageW, PostQuitMessage, TranslateMessage, MSG, PM_REMOVE,
    QS_ALLINPUT, WM_QUIT, MsgWaitForMultipleObjectsEx, MWMO_INPUTAVAILABLE, GetWindowRect,
};
use windows::Win32::Graphics::Gdi::ScreenToClient;
use windows::Win32::Graphics::Direct2D::Common::D2D_POINT_2F;
use crate::infrastructure::windowsimpl::scaling::src_tracker::SrcTracker;

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
            2,
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

        let frame = match pool.TryGetNextFrame() {
            Ok(f) => f,
            Err(_) => return Ok(FrameSourceState::Waiting),
        };

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
    _shader_manager: Option<ShaderManager>,
    _window_manager: Option<WindowManager>,
    _compute_shader: Option<ID3D11ComputeShader>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl ScalingProcessor {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            thread_id: Arc::new(AtomicU32::new(0)),
            device: None,
            context: None,
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

            let init_result = (|| -> Result<(WindowManager, ID3D11Device, ID3D11DeviceContext, IDXGISwapChain1, windows::Win32::Foundation::HANDLE, ID3D11Texture2D, ID3D11RenderTargetView, ID3D11ComputeShader, ID3D11Buffer, ID3D11Texture2D, ID3D11UnorderedAccessView, crate::infrastructure::windowsimpl::scaling::toolbar::Toolbar, crate::infrastructure::windowsimpl::scaling::cursor::CursorManager, crate::infrastructure::windowsimpl::scaling::cursor_renderer::CursorRenderer, crate::infrastructure::windowsimpl::scaling::overlay::SimpleToolbar, windows::Win32::Graphics::Direct3D11::ID3D11SamplerState)> {
                println!("Init: WindowManager");
                let mut window_manager = WindowManager::new();
                window_manager.prepare_target_window(target_hwnd)?;
                window_manager.create_overlay_window()?;
                let overlay_window = window_manager
                    .get_overlay_window()
                    .ok_or(anyhow!("No overlay window"))?;

                println!("Init: CreateDevice");
                let device = create_d3d_device()?;
                let context = unsafe { device.GetImmediateContext()? };
                unsafe {
                    let feature_level = device.GetFeatureLevel();
                    println!("D3D Feature Level: {:?}", feature_level);
                }

                println!("Init: DXGI Objects");
                let dxgi_device: IDXGIDevice = device.cast()?;
                let adapter = unsafe { dxgi_device.GetAdapter()? };
                let factory: IDXGIFactory2 = unsafe { adapter.GetParent()? };

                let mut rect = windows::Win32::Foundation::RECT::default();
                unsafe { windows::Win32::UI::WindowsAndMessaging::GetClientRect(overlay_window, &mut rect) };
                let width = (rect.right - rect.left) as u32;
                let height = (rect.bottom - rect.top) as u32;
                println!("Overlay Window Size: {}x{}", width, height);

                let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
                    Width: width,
                    Height: height,
                    Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    Stereo: false.into(),
                    SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                    BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                    BufferCount: 2,
                    Scaling: windows::Win32::Graphics::Dxgi::DXGI_SCALING_STRETCH,
                    SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                    AlphaMode: DXGI_ALPHA_MODE_IGNORE,
                    // フレームレイテンシ制御を有効化 (Magpie方式)
                    Flags: windows::Win32::Graphics::Dxgi::DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT.0 as u32,
                };

                println!("Init: SwapChain");
                let swap_chain: IDXGISwapChain1 = unsafe {
                    // NOTE: Sometimes 0x887A0001 happens if window style is weird (e.g. layered + child?)
                    // Overlay is WS_POPUP | WS_VISIBLE | WS_EX_LAYERED | WS_EX_TRANSPARENT.
                    // Layered windows support Flip SwapChain? Yes, starting from Win8.
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
                // Must match SwapChain size and format (roughly)
                // SwapChain is B8G8R8A8, CS usually outputs R8G8B8A8?
                // Magpie Shaders output R8G8B8A8 usually.
                // WE NEED TO BE CAREFUL ABOUT FORMAT.
                // If SwapChain is B8G8R8A8 (standard for Windows Store/Composition),
                // and CS writes RGBA, channels might be swapped.
                // For now, let's use B8G8R8A8 for intermediate too if possible.
                // D3D11 supports UAV on B8G8R8A8? Yes.
                
                let output_texture_desc = D3D11_TEXTURE2D_DESC {
                    Width: width,
                    Height: height,
                    MipLevels: 1,
                    ArraySize: 1,
                    Format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
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
                let output_uav = output_uav_out.unwrap();

                println!("Init: Shaders");
                let mut shader_path_buf = std::env::current_dir()?;
                if !shader_path_buf.ends_with("src-tauri") {
                    shader_path_buf.push("src-tauri");
                }
                shader_path_buf.push("src/infrastructure/windowsimpl/scaling/shaders");

                let shader_manager = ShaderManager::new(device.clone(), &shader_path_buf);
                let shade_file_name = if shader_name.ends_with(".hlsl") {
                    shader_name.clone()
                } else {
                    format!("{}.hlsl", shader_name)
                };
                let full_shader_path = shader_path_buf.join(shade_file_name);
                
                let compute_shader =
                    shader_manager.compile_compute_shader(&full_shader_path)
                        .map_err(|e| anyhow!("Shader compilation failed: {:?}", e))?;

                println!("Init: ConstantBuffer");
                let const_buffer = unsafe {
                    let desc = D3D11_BUFFER_DESC {
                        ByteWidth: std::mem::size_of::<MagpieConstants>() as u32,
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

                println!("Init: Done");
                Ok((window_manager, device, context, swap_chain, frame_latency_handle, cached_backbuffer, cached_rtv, compute_shader, const_buffer, output_texture, output_uav, toolbar, cursor_manager, cursor_renderer, simple_toolbar, sampler))
            })();

            match init_result {
                Ok((window_manager, device, context, swap_chain, frame_latency_handle, cached_backbuffer, cached_rtv, compute_shader, const_buffer, output_texture, output_uav, mut toolbar, mut cursor_manager, mut cursor_renderer, mut simple_toolbar, sampler)) => {
                    println!("Initialization successful");
                    let mut src_tracker = SrcTracker::new(target_hwnd);
                    let source_result = CaptureFrameSource::new(target_hwnd, device.clone());
                    
                    if let Ok(mut source) = source_result {
                        if let Ok(_) = source.start() {
                            println!("Capture started");
                            
                            let mut last_time = std::time::Instant::now();
                            let mut frames = 0;
                            let mut total_processing_time = 0.0;
                            let mut processing_start;

                            while running.load(Ordering::SeqCst) {
                                unsafe {
                                    let mut msg = MSG::default();
                                    while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                                        if msg.message == WM_QUIT {
                                            running.store(false, Ordering::SeqCst);
                                        }
                                        DispatchMessageW(&msg);
                                    }
                                }

                                if !running.load(Ordering::SeqCst) { break; }

                                // ソースウィンドウ監視: 消滅/非表示/最小化で終了
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

                                let mut got_new_frame = false;
                                match source.update() {
                                    Ok(FrameSourceState::NewFrame) => {
                                        if let Ok(input_texture) = source.get_texture() {
                                            unsafe {
                                                let mut input_srv = None;
                                                let _ = device.CreateShaderResourceView(&input_texture, None, Some(&mut input_srv));

                                                if let Some(input_srv) = input_srv {
                                                    let mut desc = D3D11_TEXTURE2D_DESC::default();
                                                    input_texture.GetDesc(&mut desc);
                                                    let input_width = desc.Width;
                                                    let input_height = desc.Height;
                                                    
                                                    // Output Size = SwapChain Size
                                                    let mut output_desc = D3D11_TEXTURE2D_DESC::default();
                                                    output_texture.GetDesc(&mut output_desc);
                                                    let output_width = output_desc.Width;
                                                    let output_height = output_desc.Height;

                                                    // Pseudo-borderless: Calculate borders based on Magpie logic
                                                    let _ = src_tracker.update();
                                                    let source_rect = src_tracker.get_capture_rect();
                                                    let window_kind = src_tracker.get_window_kind();
                                                    
                                                    // Frame Rect for reference (Capture is usually Frame)
                                                    let frame_rect = src_tracker.get_frame_rect();

                                                    // Calculate offsets relative to Frame Rect
                                                    // Source Rect tells us what part of the Screen we want.
                                                    // Texture contains everything in FrameRect (usually).
                                                    // So we map SourceRect relative to FrameRect.
                                                    
                                                    let mut border_left = source_rect.left - frame_rect.left;
                                                    let mut border_top = source_rect.top - frame_rect.top;
                                                    
                                                    // Clamp to 0
                                                    if border_left < 0 { border_left = 0; }
                                                    if border_top < 0 { border_top = 0; }

                                                    let region_w = (source_rect.right - source_rect.left) as f32;
                                                    let region_h = (source_rect.bottom - source_rect.top) as f32;
                                                    
                                                    let cap_w = input_width as f32;
                                                    let cap_h = input_height as f32;

                                                    // UV Offsets & Scales
                                                    let offset_u = if cap_w > 0.0 { border_left as f32 / cap_w } else { 0.0 };
                                                    let offset_v = if cap_h > 0.0 { border_top as f32 / cap_h } else { 0.0 };
                                                    let scale_u = if cap_w > 0.0 { region_w / cap_w } else { 1.0 };
                                                    let scale_v = if cap_h > 0.0 { region_h / cap_h } else { 1.0 };

                                                    let constants = MagpieConstants {
                                                        input_size: [input_width, input_height],
                                                        output_size: [output_width, output_height], 
                                                        input_pt: [1.0 / input_width as f32, 1.0 / input_height as f32],
                                                        output_pt: [1.0 / output_width as f32, 1.0 / output_height as f32],
                                                        scale: [scale_u, scale_v], // ROI / Capture ratio
                                                        src_rect_offset: [offset_u, offset_v], // Offset
                                                    };


                                                    let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
                                                    if context.Map(&const_buffer, 0, D3D11_MAP_WRITE_DISCARD, 0, Some(&mut mapped)).is_ok() {
                                                        let ptr = mapped.pData as *mut MagpieConstants;
                                                        ptr.write(constants);
                                                        context.Unmap(&const_buffer, 0);
                                                    }

                                                    context.CSSetShader(&compute_shader, None);
                                                    let cbs = [Some(const_buffer.clone())];
                                                    context.CSSetConstantBuffers(0, Some(&cbs));
                                                    let srvs = [Some(input_srv)];
                                                    context.CSSetShaderResources(0, Some(&srvs));
                                                    let samplers = [Some(sampler.clone())];
                                                    context.CSSetSamplers(0, Some(&samplers)); // Bind Sampler
                                                    let uavs = [Some(output_uav.clone())];
                                                    context.CSSetUnorderedAccessViews(0, 1, Some(uavs.as_ptr()), None);

                                                    // Dispatch
                                                    context.Dispatch((output_width + 7) / 8, (output_height + 7) / 8, 1);

                                                    let null_uav: [Option<ID3D11UnorderedAccessView>; 1] = [None];
                                                    context.CSSetUnorderedAccessViews(0, 1, Some(null_uav.as_ptr()), None);
                                                    
                                                    // Calculate Timing
                                                    let elapsed = processing_start.elapsed().as_secs_f32() * 1000.0;
                                                    total_processing_time += elapsed;

                                                    got_new_frame = true;
                                                    frames += 1;
                                                }
                                            }
                                        }
                                    }
                                    _ => {
                                        // No new frame
                                    }
                                }

                                // Update FPS & Stats (Every ~0.2s)
                                let elapsed_chk = last_time.elapsed().as_secs_f32();
                                let mut stats_updated = false;
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
                                    stats_updated = true;
                                }

                                // Always Present to consume frame latency signal and keep cursor smooth
                                unsafe {
                                    // カーソル描画前に必ずバックバッファをリセット（残像防止）
                                    // FLIP_DISCARDはバックバッファを破棄するため、常にコピーが必要
                                    context.CopyResource(&cached_backbuffer, &output_texture);
                                    
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
                                    if (dest_rect.right - dest_rect.left) > 0 {
                                        // SrcTrackerからフォーカス状態を取得
                                        let is_src_focused = src_tracker.is_focused();
                                        if let Ok(_) = cursor_manager.update(src_rect, dest_rect, target_hwnd, is_src_focused) {
                                            // ツールバー更新と描画 (カーソルより先に描画)
                                            let mut toolbar_cursor_pos = cursor_manager.draw_pos;
                                            if let Some(hwnd) = window_manager.get_overlay_window() {
                                                let _ = ScreenToClient(hwnd, &mut toolbar_cursor_pos);
                                            }
                                            simple_toolbar.tick();
                                            simple_toolbar.update_visibility(toolbar_cursor_pos, (dest_rect.bottom - dest_rect.top) as i32);
                                            let _ = simple_toolbar.render(&cached_backbuffer);
                                            
                                            // 終了ボタンチェック (左クリック)
                                            if (windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(0x01) as u16 & 0x8000) != 0 {
                                                if simple_toolbar.check_close_button_click(toolbar_cursor_pos) {
                                                    println!("Close button clicked, stopping scaling");
                                                    running.store(false, Ordering::SeqCst);
                                                    break;
                                                }
                                            }

                                            // システムカーソルを描画 (ツールバーの上に)
                                            if cursor_manager.should_draw_cursor {
                                                let mut client_pt = cursor_manager.draw_pos;
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
                                                    let _ = cursor_renderer.draw_cursor(&cached_backbuffer, cursor_info.hCursor, client_pt, 1.0);
                                                }
                                            }
                                            
                                            let _ = (&toolbar, output_desc.Width, &cached_rtv, &frame_latency_handle); // unused warning 回避
                                        }
                                    }

                                    let _ = swap_chain.Present(0, 0);
                                }
                                
                                // Wait for Frame Latency Object (to sync with refresh rate) or New Frame Event
                                unsafe {
                                    let handles = [frame_latency_handle, source.frame_event];
                                    MsgWaitForMultipleObjectsEx(Some(&handles), 1000, QS_ALLINPUT, MWMO_INPUTAVAILABLE);
                                }
                            }
                            let _ = source.stop();
                        } else {
                            println!("Failed to start capture");
                        }
                    } else {
                        println!("Failed to create capture source");
                    }
                    window_manager.restore_target_window();
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
