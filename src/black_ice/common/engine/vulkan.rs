#![allow(unused)]
#![cfg(feature = "vulkan")]
extern crate raw_window_handle;
use std::sync::*;
use ash::vk::PipelineLayout;
use ash::vk::RenderPass;
use ash::vk::ShaderModule;
use colored::*;

use ash::*;
use ash::extensions::khr::*;
use ash::vk::Handle;

use sdl2::raw_window_handle::*;
use crate::black_ice::common::filesystem::files::ShaderFile;
use crate::black_ice::common::engine::gamesys::Env;
use super::gamesys::*;
use super::pipeline::*;

#[derive(Clone)]
pub enum TextureType {
    RGB,
    DEPTH
}

#[derive(Clone)]
pub struct RenderTexture {
    inner: u32,
    width: i32,
    height: i32,
    texture_type: TextureType
}

#[derive(Default, Clone)]
pub struct DriverValues {
    pub entry: Option<Entry>,
    pub instance: Option<Instance>,
    pub physical_devices: Vec<(u32, vk::PhysicalDevice)>,
    pub logical_devices : Vec<(usize, Option<Device>, Vec<u32>)>,
    pub surface : Option<vk::SurfaceKHR>,
    pub enable_validation_layers: bool,
    pub device_ext: Vec<&'static std::ffi::CStr>,
    pub swap_chain: Option<vk::SwapchainKHR>,
    pub swap_chain_images: Vec<vk::Image>,
    pub swap_chain_image_format: Option<vk::Format>,
    pub swap_chain_extent: Option<vk::Extent2D>,
    pub swap_chain_image_views: Vec<vk::ImageView>,
    pub chosen_device: usize,
    pub debug_util: Option<vk::DebugUtilsMessengerEXT>,

    pub shader_stages: Vec<(String, Vec<u32>, Vec<u32>)>,
}



#[derive(Clone, Default)]
struct QueueFamiltyIdices {
    graphics_family: Option<u32>,
    present_family: Option<u32>,
    transfer_family: Option<u32>,
}


impl QueueFamiltyIdices {
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}


#[derive(Clone, Default)]
struct SwapChainSupportDetals {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}

pub struct PipelineValues {
    pub render_pass: RenderPass, 
    pub graphics_pipeline: vk::Pipeline, 
    pub pipeline_layout: PipelineLayout, 
    pub shader_modules: Vec<ShaderModule>
}

impl PipelineValues {
    pub fn new(render_pass: RenderPass, graphics_pipeline: vk::Pipeline, pipeline_layout: PipelineLayout, shader_modules: Vec<ShaderModule>) -> Self{
        Self { render_pass: render_pass, graphics_pipeline: graphics_pipeline, pipeline_layout: pipeline_layout, shader_modules: shader_modules }
    }
}

impl DriverValues {
    #[cfg(feature = "vulkan")]
    pub unsafe fn init_vulkan(driver:&mut DriverValues, window: &sdl2::video::Window, video: &sdl2::VideoSubsystem)  {
        
        let entry = ash::Entry::load().expect("Failed to get entry!");
        // std::thread::sleep(std::time::Duration::from_secs(1));
        // println!(
        //     "{} - Vulkan Instance {}.{}.{}",
        //     "cock and balls",
        //     vk::api_version_major(entry.try_enumerate_instance_version().unwrap().unwrap()),
        //     vk::api_version_minor(entry.try_enumerate_instance_version().unwrap().unwrap()),
        //     vk::api_version_patch(entry.try_enumerate_instance_version().unwrap().unwrap())
        // );
        

        let mut app_info = vk::ApplicationInfo::default();
        app_info.s_type = vk::StructureType::APPLICATION_INFO;
        let game_name =  Env::get_game_name();
        let gm = std::ffi::CString::new(game_name.as_str()).unwrap();
        app_info.p_application_name = gm.as_ptr();
        app_info.application_version = vk::make_api_version(0, 0, 1, 0);
        let engine_name = std::ffi::CString::new("Black-Ice Engine").unwrap();
        app_info.p_engine_name = engine_name.as_ptr();
        app_info.engine_version = vk::make_api_version(0, 0, 1, 0);
        app_info.api_version = vk::make_api_version(0, 1, 3, 241);
        
        let mut extension_names = Self::get_window_extentions(window);
        extension_names.push(ash::extensions::ext::DebugUtils::name().to_str().unwrap());
        // #[cfg(target_os = "linux")] extension_names.push(ash::extensions::khr::XlibSurface::name().to_str().unwrap());
        for ext in extension_names.to_vec() {
            //println!("{}", ext.blue());
        }
        let layer_names = [std::ffi::CStr::from_bytes_with_nul_unchecked(
            b"VK_LAYER_KHRONOS_validation\0",
        )];
        let layers_names_raw: Vec<*const std::os::raw::c_char> = layer_names
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();
        let ext1 = extension_names.into_iter().map(|s| s.as_ptr().cast::<i8>()).collect::<Vec<*const i8>>();
        let create_flags = vk::InstanceCreateFlags::default();


        let mut create_info = vk::InstanceCreateInfo::builder()
            .enabled_extension_names(ext1.as_slice())
            .enabled_layer_names(&layers_names_raw)
            .application_info(&app_info)
            .flags(create_flags)
            .build();
        
        
        let instance = entry.create_instance(&create_info, None).expect("Failed to create Instance!");
        
        // println!("Created Inst.");

        // let exts = entry.enumerate_instance_extension_properties(None)
        //     .expect("Failed to get Extention data.");
        // for ext in exts {
        //     println!("{}", std::ffi::CStr::from_ptr(ext.extension_name.as_ptr()).to_str().unwrap());
        // }

        // create devices

        let physical_devices = instance.enumerate_physical_devices().expect("Failed to get physical devices");

        driver.entry = Some(entry);
        driver.instance = Some(instance);
        driver.device_ext = vec![
            ash::extensions::khr::Swapchain::name()
        ];

        DriverValues::get_debug_messanger(driver);
        DriverValues::get_surface(driver, window);
        DriverValues::register_physical_devices(driver, physical_devices);
        DriverValues::create_logical_devices(driver);
        DriverValues::choose_best_device(driver);
        DriverValues::create_swap_chain(driver);
        DriverValues::create_image_views(driver);
        //println!("Created Swapcahin!!");
        
        // let mut vt_input = vk::VertexInputBindingDescription::default();
        // vt_input.input_rate = vk::VertexInputRate::VERTEX;
        // vt_input.stride = std::mem::size_of::<[f32; 3]>() as u32;
        // vt_input.binding = 0;
        
    }

    #[cfg(feature = "vulkan")]
    unsafe fn create_shader_module(driver:&Self, code: Vec<u32>) -> vk::ShaderModule {
        let shader_module_info = vk::ShaderModuleCreateInfo::builder()
            .code(code.as_slice())
            .build();

        let logical_device = DriverValues::get_current_logical_device(driver);

        logical_device.create_shader_module(&shader_module_info, None).expect("Failed to create Shader Module!!")

    }
    
    // Create a custom pipeline function and a compute pipeline function!!

    #[cfg(feature = "vulkan")]
    pub unsafe fn create_graphics_pipeline(driver: &Self, stage: usize) -> PipelineValues {

        
        assert!(!driver.shader_stages.is_empty());
        assert!(driver.shader_stages.len() > stage);

        let (shader_name, frag_shader, vert_shader) = driver.shader_stages[stage].clone();

        let frag_module = DriverValues::create_shader_module(driver, frag_shader);
        let vert_module = DriverValues::create_shader_module(driver, vert_shader);

        let mut frag_name = shader_name.clone();
        frag_name.push_str("_fragment");
        let frag_shader_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_module)
            .name(std::ffi::CString::new(frag_name).unwrap().as_c_str())
            .build();

        let mut vert_name = shader_name.clone();
        vert_name.push_str("_vertex");
        let vert_shader_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_module)
            .name(std::ffi::CString::new(vert_name).unwrap().as_c_str())
            .build();

        let dynamic_states = vec![
            vk::DynamicState::VIEWPORT,
            vk::DynamicState::SCISSOR,
        ];

        let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(dynamic_states.as_slice())
            .build();

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .build();

        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(true)
            .build();

        let viewport = vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(driver.swap_chain_extent.unwrap().width as f32)
            .height(driver.swap_chain_extent.unwrap().height as f32)
            .min_depth(0.0)
            .max_depth(1.0)
            .build();

        let scissor = vk::Rect2D::builder()
            .offset(vk::Offset2D::builder().x(0).y(0).build())
            .extent(driver.swap_chain_extent.unwrap().clone())
            .build();

        let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .viewports(vec![viewport].as_slice())
            .scissor_count(1)
            .scissors(vec![scissor].as_slice())
            .build();

        let raster_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false)
            .depth_bias_constant_factor(0.0)
            .depth_bias_clamp(0.0)
            .depth_bias_slope_factor(0.0)
            .build();
        
        let mutlisample_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .min_sample_shading(1.0)
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false)
            .build();

        let color_blend_attatchment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(
                vk::ColorComponentFlags::R
                | vk::ColorComponentFlags::G
                | vk::ColorComponentFlags::B
                | vk::ColorComponentFlags::A
            )
            .blend_enable(false)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .build();

        let color_blending_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(vec![color_blend_attatchment].as_slice())
            .blend_constants([0.0, 0.0, 0.0, 0.0])
            .build();

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
            .build();

        
        let instance = driver.instance.as_ref().unwrap();
        let logical_device = DriverValues::get_current_logical_device(driver);
        
        let pipeline_layout = logical_device.create_pipeline_layout(&pipeline_layout_info, None).expect("Failed to create pipeline layout!!");

        let color_attatchment = vk::AttachmentDescription::builder()
            .format(driver.swap_chain_image_format.unwrap())
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build();

        let color_attatchment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();

        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(vec![color_attatchment_ref].as_slice())
            .build();

        let render_pass_info = vk::RenderPassCreateInfo::builder()
            .attachments(vec![color_attatchment].as_slice())
            .subpasses(vec![subpass].as_slice())
            .build();

        let render_pass = logical_device.create_render_pass(&render_pass_info, None).expect("Failed to create render pass!!");

        let graphics_pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(vec![frag_shader_stage, vert_shader_stage].as_slice())
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&raster_info)
            .multisample_state(&mutlisample_info)
            .color_blend_state(&color_blending_info)
            .dynamic_state(&dynamic_state)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .build();

        let graphics_pipeline = logical_device.create_graphics_pipelines(std::mem::zeroed(), vec![graphics_pipeline_info].as_slice(), None)
            .expect("Failed to create Graphics Pipeline!!!")[0];

        PipelineValues::new(render_pass, graphics_pipeline, pipeline_layout, vec![frag_module, vert_module])
        
    }

    #[cfg(feature = "vulkan")]
    unsafe fn get_debug_messanger(driver:&mut Self){
        use ash::vk::DebugUtilsMessengerCreateInfoEXT;
        let entry = driver.entry.as_ref().unwrap();
        let instance = driver.instance.as_ref().unwrap();
        let debug_utils_loader = ash::extensions::ext::DebugUtils::new(entry, instance);
        let create_info = DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR 
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::DEVICE_ADDRESS_BINDING
            )
            .pfn_user_callback(Some(vulkan_debug_callback))
            .build();
        driver.debug_util = Some(debug_utils_loader.create_debug_utils_messenger(&create_info, None).expect("Failed to create debug utils!!"));
    }

    #[cfg(feature = "vulkan")]
    unsafe fn get_window_extentions(window: &sdl2::video::Window) -> Vec<&'static str> {
        #[cfg(target_os = "windows")]
        unsafe fn get_window_extentions_a() -> Vec<*const i8> {
            let mut window_info: sdl2::raw_window_handle::SDL_SysWMinfo = std::mem::zeroed();
            let bb  = SDL_GetWindowWMInfo(GAME.window.raw(), &mut window_info);
            
            let display_handle = raw_window_handle::WindowsDisplayHandle::empty();

            let mut window_handle = raw_window_handle::Win32WindowHandle::empty();
            window_handle.hinstance = window_info.info.win.hinstance.cast();
            window_handle.hwnd = window_info.info.win.window.cast();

            

            ash_window::enumerate_required_extensions(raw_window_handle::RawDisplayHandle::Windows(display_handle))
                .expect("Failed to get window extentions").to_vec()
            
        }
        


        #[cfg(target_os = "linux")]
        unsafe fn get_window_extentions_a(window: &sdl2::video::Window) -> Vec<*const i8> {

            let process = std::process::Command::new("sh").arg("-c").arg("echo $XDG_SESSION_TYPE").output().unwrap();
            let windowing = std::str::from_utf8_unchecked(process.stdout.as_slice());
            //println!("{}", windowing);
            match windowing {

                "wayland\n" =>{
                    let mut window_info: sdl2::raw_window_handle::SDL_SysWMinfo = std::mem::zeroed();
                    SDL_GetWindowWMInfo(window.raw(), &mut window_info);

                    let mut display_handle = raw_window_handle::WaylandDisplayHandle::empty();
                    
                    display_handle.display = window_info.info.wl.display.cast();

                    let mut window_handle = raw_window_handle::WaylandWindowHandle::empty();

                    window_handle.surface = window_info.info.wl.surface.cast();

                    ash_window::enumerate_required_extensions(raw_window_handle::RawDisplayHandle::Wayland(display_handle))
                        .expect("Failed to get window extentions").to_vec()
                }
                "x11\n" => {
                    let mut window_info: sdl2::raw_window_handle::SDL_SysWMinfo = std::mem::zeroed();
                    SDL_GetWindowWMInfo(window.raw(), &mut window_info);
    
                    let mut display_handle = raw_window_handle::XlibDisplayHandle::empty();
                    
                    display_handle.display = window_info.info.x11.display.cast();
    
                    let mut window_handle = raw_window_handle::XlibWindowHandle::empty();
    
                    window_handle.window = window_info.info.x11.window;
    
                    ash_window::enumerate_required_extensions(raw_window_handle::RawDisplayHandle::Xlib(display_handle))
                        .expect("Failed to get window extentions").to_vec()
                }
                _=> todo!()
            }
        }

        #[cfg(target_os = "macos")]
        unsafe fn get_window_extentions_a() -> Vec<*const i8> {

            let mut window_info: sdl2::raw_window_handle::SDL_SysWMinfo = std::mem::zeroed();
            SDL_GetWindowWMInfo(GAME.window.raw(), &mut window_info);

            let mut display_handle = raw_window_handle::AppKitDisplayHandle::empty();

            

            let mut window_handle = raw_window_handle::AppKitWindowHandle::empty();

            window_handle.ns_window = window_info.info.cocoa.window.cast();

            ash_window::enumerate_required_extensions(raw_window_handle::RawDisplayHandle::AppKit(display_handle))
                .expect("Failed to get window extentions").to_vec()

        }

        get_window_extentions_a(window).iter().map(|s| std::ffi::CStr::from_ptr(s.clone()).to_str().unwrap()).collect()
    }

    #[cfg(feature = "vulkan")]
    pub fn get_current_logical_device(driver:&Self) -> &Device {
        driver.logical_devices.iter().find(|v| v.0 == driver.chosen_device).expect("Failed to find logical device!!").1.as_ref().unwrap()
    }

    #[cfg(feature = "vulkan")]
    unsafe fn create_image_views(driver:&mut Self){


        for swap_chain_image in &driver.swap_chain_images {
            let create_info = vk::ImageViewCreateInfo::builder()
                .image(*swap_chain_image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .components(
                    vk::ComponentMapping::builder()
                    .r(vk::ComponentSwizzle::IDENTITY)
                    .g(vk::ComponentSwizzle::IDENTITY)
                    .b(vk::ComponentSwizzle::IDENTITY)
                    .a(vk::ComponentSwizzle::IDENTITY)
                    .build()
                )
                .subresource_range(
                    vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build()
                )
                .format(vk::Format::B8G8R8A8_SRGB)
                .build();
            let device = driver.logical_devices.iter().find(|v| v.0 == driver.chosen_device).unwrap().1.as_ref().unwrap();
            let image_view = device.create_image_view(&create_info, None).expect("Failed to create image_view");
            driver.swap_chain_image_views.push(image_view);
        }
    }

    #[cfg(feature = "vulkan")]
    unsafe fn choose_best_device(driver: &mut DriverValues) {
        let mut best = 0;
        let mut cost = 0;
        let mut i = 0;
        let physical_devices = driver.physical_devices.clone();
        
        for physical_device in physical_devices {
            if physical_device.0 > cost {
                best = i.clone();
                cost = physical_device.0.clone();
            }
            i += 1;
        }

        //driver.chosen_device = 0;
    }

    #[cfg(feature = "vulkan")]
    unsafe fn create_swap_chain(driver:&mut DriverValues) {



            let physical_devices = driver.physical_devices.clone();
            let device = physical_devices[driver.chosen_device].1;
            let chosen_device = driver.chosen_device.clone();
            let support_details = DriverValues::queary_swap_chain_support(driver, device);

            let surface_format = DriverValues::choose_swap_surface_format(support_details.formats);
            let present_mode = DriverValues::choose_swap_present_mode(support_details.present_modes);
            let extent = DriverValues::choose_swap_extent(support_details.capabilities);

            let mut image_count = support_details.capabilities.min_image_count + 1;

            if support_details.capabilities.max_image_count > 0 && image_count > support_details.capabilities.max_image_count {
                image_count = support_details.capabilities.max_image_count;
            }
            let logical_device = driver.logical_devices.iter().find(|v| v.0 == chosen_device).unwrap();


            let mut swap_chain_info = vk::SwapchainCreateInfoKHR::builder()
                .surface(driver.surface.unwrap())
                .min_image_count(image_count)
                .image_format(surface_format.format)
                .image_color_space(surface_format.color_space)
                .image_extent(extent)
                .image_array_layers(1)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .clipped(true)
                .build();
            let queue_family_indices = logical_device.2.as_slice();

            if queue_family_indices[0] != queue_family_indices[1] {
                swap_chain_info.image_sharing_mode = vk::SharingMode::CONCURRENT;
                swap_chain_info.queue_family_index_count = 2;
                swap_chain_info.p_queue_family_indices = queue_family_indices.clone().as_ptr();
            }
            else {
                swap_chain_info.image_sharing_mode = vk::SharingMode::EXCLUSIVE;
                // swap_chain_info.queue_family_index_count = 1;
                // swap_chain_info.p_queue_family_indices = vec![queue_family_indices[0].clone()].as_ptr();
            }

            swap_chain_info.pre_transform = if !support_details.capabilities.current_transform.contains(vk::SurfaceTransformFlagsKHR::IDENTITY) 
                {support_details.capabilities.current_transform} 
                else {vk::SurfaceTransformFlagsKHR::IDENTITY};
            swap_chain_info.composite_alpha = vk::CompositeAlphaFlagsKHR::OPAQUE;

            swap_chain_info.present_mode = present_mode;
            if driver.swap_chain.is_some() {
                swap_chain_info.old_swapchain = driver.swap_chain.unwrap();
            }
            let instance = driver.instance.as_ref().unwrap();
            let swap_chain = Swapchain::new(driver.instance.as_ref().unwrap(), logical_device.1.as_ref().unwrap());
            
            
            let test = swap_chain.create_swapchain(&swap_chain_info, None);
            driver.swap_chain = Some( 
                // match test {
                
                // Ok(v) => v,
                // Err(_) => {
                //     //println!("Failed to load some swapchain. Trying again. (Close app if this message persists).");
                //     continue 'run;
                // }
                
                
                // }
                test.expect("Failed to create SwapChain!!")
            );
            
            driver.swap_chain_images = swap_chain.get_swapchain_images(driver.swap_chain.unwrap()).expect("Failed to get swapchain images!!");
            driver.swap_chain_image_format = Some(surface_format.format);
            driver.swap_chain_extent = Some(extent);



    }

    #[cfg(feature = "vulkan")]
    unsafe fn choose_swap_extent(capabilities: vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            return capabilities.current_extent;
        }

        let (height, width) = Env::get_window_size();

        let mut actual_extents = vk::Extent2D::builder()
            .height(height)
            .width(width)
            .build();

        actual_extents.width = actual_extents.width.clamp(capabilities.min_image_extent.width, capabilities.max_image_extent.width);
        actual_extents.height = actual_extents.height.clamp(capabilities.min_image_extent.height, capabilities.max_image_extent.height);

        actual_extents
    }

    #[cfg(feature = "vulkan")]
    unsafe fn choose_swap_present_mode(present_modes: Vec<vk::PresentModeKHR>) -> vk::PresentModeKHR {
        for present_mode in &present_modes {
            if present_mode.eq(&vk::PresentModeKHR::MAILBOX) {
                return *present_mode;
            }
        }
        
        
        vk::PresentModeKHR::FIFO
    }

    #[cfg(feature = "vulkan")]
    unsafe fn choose_swap_surface_format(formats: Vec<vk::SurfaceFormatKHR>) -> vk::SurfaceFormatKHR {
        for format in &formats {
            if format.format == vk::Format::B8G8R8A8_SRGB && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR {
                return *format;
            }
        }
        
        formats[0]
    }

    #[cfg(feature = "vulkan")]
    unsafe fn queary_swap_chain_support(driver: &mut DriverValues, device: vk::PhysicalDevice) -> SwapChainSupportDetals {
        let mut details = SwapChainSupportDetals::default();
        let surface_loader = extensions::khr::Surface::new(driver.entry.as_ref().unwrap(), driver.instance.as_ref().unwrap());
        details.capabilities = surface_loader.get_physical_device_surface_capabilities(device, driver.surface.unwrap())
            .expect("Failed to get surface capabilities.");
        details.formats = surface_loader.get_physical_device_surface_formats(device, driver.surface.unwrap())
            .expect("Failed to get formats.");
        details.present_modes = surface_loader.get_physical_device_surface_present_modes(device, driver.surface.unwrap())
            .expect("Failed to get present modes.");
        details
    }

    #[cfg(feature = "vulkan")]
    unsafe fn create_logical_devices(driver: &mut DriverValues) {
        

        let physical_devices = driver.physical_devices.clone();
        let mut i = 0;
        for physical_device in physical_devices {
            let physical_device_features = driver.instance.as_ref().unwrap().get_physical_device_features(physical_device.1);
            let queue_indices = DriverValues::find_queue_families(driver, physical_device.1);

            let queue_priorities = vec![1.0];

            let mut queue_graphics_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_indices.graphics_family.unwrap())
            .queue_priorities(queue_priorities.as_slice())
            .build();
            queue_graphics_info.queue_count = 1;

            let extension_names = driver.device_ext.iter().map(|v| v.as_ptr()).collect::<Vec<*const i8>>();

            // for ext in extension_names {
            //     println!("{:?}", std::ffi::CStr::from_ptr(*ext));
            // }

            let mut create_infos = vec![queue_graphics_info];

            // if queue_indices.transfer_family.is_some() && queue_indices.transfer_family.unwrap() != queue_indices.graphics_family.unwrap() {
            //     let info = vk::DeviceQueueCreateInfo::builder()
            //         .queue_family_index(queue_indices.transfer_family.unwrap())
            //         .build();
            //     create_infos.push(info);
            // }

            if queue_indices.present_family.unwrap() != queue_indices.graphics_family.unwrap() {
                let mut queue_present_info = vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_indices.present_family.unwrap())
                .queue_priorities(queue_priorities.as_slice())
                .build();
                create_infos.push(queue_present_info);
                queue_present_info.queue_count = 1;
            }

            let logical_device_info = vk::DeviceCreateInfo::builder()
            .enabled_features(&physical_device_features)
            .queue_create_infos(create_infos.as_slice())
            .enabled_extension_names(extension_names.as_slice())
            .build();

            
            let instance = driver.instance.as_ref().unwrap().clone();
            // OBS will except here on it's thread. Don't worry though.
            let logical_device = instance.create_device(physical_device.1.clone(), &logical_device_info, None)
            .expect("Failed to build logical device!!");
            assert!(logical_device.handle().as_raw() != u64::MAX);
            // if queue_indices.graphics_family.is_some() {
            //     let graphics_queue = logical_device.get_device_queue(queue_indices.graphics_family.unwrap(), 0);
            // }

            driver.logical_devices.push((i, Some(logical_device), vec![queue_indices.graphics_family.unwrap(), queue_indices.present_family.unwrap()]));
            i +=1;
        }
    }

    #[cfg(feature = "vulkan")]
    unsafe fn register_physical_devices(driver: &mut DriverValues, physical_devices: Vec<vk::PhysicalDevice>){
        let mut device_candidates = Vec::<(u32, vk::PhysicalDevice)>::new();
        for physical_device in physical_devices {
            device_candidates.push((DriverValues::rate_physical_device(&physical_device, driver.instance.as_ref().unwrap()), physical_device));
        }
        let mut chosen_devices = Vec::<(u32, vk::PhysicalDevice)>::new();
        for device_candidate in device_candidates {
            if device_candidate.0 > 0 {
                
                if DriverValues::check_device_suitability(driver, device_candidate.1) {
                    chosen_devices.push(device_candidate);
                }
            }
        }
        if chosen_devices.len() == 0 {
            panic!("Failed to find suitable device!! Instead of vulkan use OpenGL instead!!");
        }
        driver.physical_devices = chosen_devices;
    }

    #[cfg(feature = "vulkan")]
    unsafe fn check_device_suitability(driver:&mut DriverValues, device: vk::PhysicalDevice) -> bool{
        let indices = DriverValues::find_queue_families(driver, device);

        let extensions_supported = DriverValues::check_device_extensions_supported(driver, device);
        let mut swap_chain_adequate = false;
        if extensions_supported {
            let swap_chain_support_details = DriverValues::queary_swap_chain_support(driver, device);
            swap_chain_adequate = !swap_chain_support_details.formats.is_empty() && !swap_chain_support_details.present_modes.is_empty();
        }

        return indices.is_complete() && extensions_supported && swap_chain_adequate;
    }

    #[cfg(feature = "vulkan")]
    unsafe fn check_device_extensions_supported(driver:&mut DriverValues, device: vk::PhysicalDevice) -> bool{

        let available_ext = driver.instance.as_ref().unwrap().enumerate_device_extension_properties(device)
        .expect("Failed to get device extension properties");
        
        let device_ext = &driver.device_ext;

        let mut required_ext = device_ext.clone();

        for ext in available_ext {
            
            if device_ext.contains(&std::ffi::CStr::from_ptr(ext.extension_name.as_ptr())) {
                required_ext.remove(
                    device_ext.binary_search(&std::ffi::CStr::from_ptr(ext.extension_name.as_ptr())).unwrap()
                );
            }
            //println!("{}", &std::ffi::CStr::from_ptr(ext.extension_name.as_ptr()).to_str().unwrap());
        }

        required_ext.is_empty()
    }

    #[cfg(feature = "vulkan")]
    unsafe fn find_queue_families(driver:&mut DriverValues, device: vk::PhysicalDevice) -> QueueFamiltyIdices{

        let mut indices: QueueFamiltyIdices = QueueFamiltyIdices::default();
        let mut queue_fams = driver.instance.as_ref().unwrap().get_physical_device_queue_family_properties(device);
        let mut surface_loader = extensions::khr::Surface::new(driver.entry.as_ref().unwrap(), driver.instance.as_ref().unwrap());
        

        let mut i: u32 = 0;
        for queue_family in queue_fams {
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                indices.graphics_family = Some(i);
                
            }

            let present_support = surface_loader.get_physical_device_surface_support(device, i, driver.surface.unwrap())
            .expect("Failed to check surface support!!");
            if present_support {
                indices.present_family = Some(i);
            }

            if queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER) {
                indices.transfer_family = Some(i);
            }
            

            if indices.is_complete() {
                break;
            }
            i +=1;
        }

        indices
    }

    #[cfg(feature = "vulkan")]
    unsafe fn rate_physical_device(physical_device: &vk::PhysicalDevice, instance: &Instance) -> u32 {
        let mut score: u32 = 0;
        let physical_device_properties = instance.get_physical_device_properties(*physical_device);
        let physical_device_features = instance.get_physical_device_features(*physical_device);
        
        if(physical_device_properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU) {
            score += 1000000;
        }

        score += physical_device_properties.limits.max_image_dimension2_d;
        //println!("{name}: {score} : {hasGeom}", name=std::ffi::CStr::from_ptr(physical_device_properties.device_name.as_ptr()).to_str().unwrap(), 
        //hasGeom = physical_device_features.geometry_shader);
        if physical_device_features.geometry_shader == 0 {
            return 0;
        }
        
        score
    }

    #[cfg(feature = "vulkan")]
    pub unsafe fn get_surface(driver: &mut DriverValues, window: &sdl2::video::Window){

        

        #[cfg(target_os = "windows")]
        unsafe fn get_surface_a(driver: &mut DriverValues, window: &sdl2::video::Window) -> Option<vk::SurfaceKHR> {
            let mut window_info: sdl2::raw_window_handle::SDL_SysWMinfo = std::mem::zeroed();
            let bb  = SDL_GetWindowWMInfo(window.raw(), &mut window_info);

            let display_handle = raw_window_handle::WindowsDisplayHandle::empty();

            let mut window_handle = raw_window_handle::Win32WindowHandle::empty();
            window_handle.hinstance = window_info.info.win.hinstance.cast();
            window_handle.hwnd = window_info.info.win.window.cast();

            let surface = ash_window::create_surface(driver.entry.as_ref().unwrap(), 
                driver.instance.as_ref().unwrap(), 
                raw_window_handle::RawDisplayHandle::Windows(display_handle), 
                raw_window_handle::RawWindowHandle::Win32(window_handle), None)
                .expect("Failed to create surface!!");

            Some(surface)
        }
        


        #[cfg(target_os = "linux")]
        unsafe fn get_surface_a(driver: &mut DriverValues, window: &sdl2::video::Window) -> Option<vk::SurfaceKHR> {
            // Assume wayland!!
            let mut window_info: sdl2::raw_window_handle::SDL_SysWMinfo = std::mem::zeroed();
            SDL_GetWindowWMInfo(window.raw(), &mut window_info);

            let mut display_handle = raw_window_handle::WaylandDisplayHandle::empty();

            display_handle.display = window_info.info.wl.display.cast();

            let mut window_handle = raw_window_handle::WaylandWindowHandle::empty();

            window_handle.surface = window_info.info.wl.surface.cast();

            let surface = ash_window::create_surface(driver.entry.as_ref().unwrap(), 
                driver.instance.as_ref().unwrap(), 
                raw_window_handle::RawDisplayHandle::Wayland(display_handle), 
                raw_window_handle::RawWindowHandle::Wayland(window_handle), None)
                .expect("Failed to create surface!!");
            Some(surface)
        }

        #[cfg(target_os = "macos")]
        unsafe fn get_surface_a(driver: &mut DriverValues, window: &sdl2::video::Window) -> Option<vk::SurfaceKHR> {

            let mut window_info: sdl2::raw_window_handle::SDL_SysWMinfo = std::mem::zeroed();
            SDL_GetWindowWMInfo(window.raw(), &mut window_info);

            let mut display_handle = raw_window_handle::AppKitDisplayHandle::empty();

            

            let mut window_handle = raw_window_handle::AppKitWindowHandle::empty();

            window_handle.ns_window = window_info.info.cocoa.window.cast();

            let surface = ash_window::create_surface(driver.entry.as_ref().unwrap(), 
                driver.instance.as_ref().unwrap(), 
                raw_window_handle::RawDisplayHandle::AppKit(display_handle), 
                raw_window_handle::RawWindowHandle::AppKit(window_handle), None)
                .expect("Failed to create surface!!");
            Some(surface)

        }

        let handle = driver.instance.as_ref().unwrap().handle().as_raw();

        let surface_khr = window.vulkan_create_surface(handle as usize).expect("failed to create surface");

        driver.surface = Some(vk::SurfaceKHR::from_raw(surface_khr));

    }

    pub unsafe fn destroy_vulkan(driver: &mut Self){

        let instance = driver.instance.as_ref().unwrap();
        let physical_devices = driver.physical_devices.clone();
        let _device = physical_devices[driver.chosen_device].1;
        let chosen_device = driver.chosen_device.clone();
        let logical_device = driver.logical_devices.iter().find(|v| v.0 == chosen_device).unwrap().1.as_ref().unwrap();
        for image_view in &driver.swap_chain_image_views {
            logical_device.destroy_image_view(*image_view, None);
        }
        let swap_chain = Swapchain::new(instance, logical_device);
        swap_chain.destroy_swapchain(driver.swap_chain.unwrap(), None);
        

        for logical_device in &driver.logical_devices {
            // destroy everything to do with logical device!!
            

            logical_device.1.as_ref().unwrap().destroy_device(None);
        }

        instance.destroy_instance(None);
        
        
    }

    pub unsafe fn create_render_texture(this: &mut Self, width: i32, height: i32, texture_type: TextureType) -> RenderTexture
    {
        
    }

}


pub trait VulkanRender {
    fn init(th: Arc<Mutex<Self>>) -> i32;
    fn render(th: Arc<Mutex<Self>>, p_window: Arc<Mutex<sdl2::video::Window>>, p_video: Arc<Mutex<sdl2::VideoSubsystem>>) -> i32;
    fn cleanup(th:Arc<Mutex<Self>>);
}

impl VulkanRender for Pipeline {
    fn init(th: Arc<Mutex<Self>>) -> i32 {
        0
    }

    fn render(th: Arc<Mutex<Self>>, p_window: Arc<Mutex<sdl2::video::Window>>, p_video: Arc<Mutex<sdl2::VideoSubsystem>>) -> i32 {
        todo!()
    }

    fn cleanup(th:Arc<Mutex<Self>>) {
        todo!()
    }
}


extern "C" {
    fn SDL_GetWindowWMInfo(window: *mut sdl2::sys::SDL_Window, info: *mut sdl2::raw_window_handle::SDL_SysWMinfo) -> sdl2::sys::SDL_bool;
}

pub unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number = callback_data.message_id_number;
    
    let message_id_name = if callback_data.p_message_id_name.is_null() {
        String::from("")
    } else {
        String::from(std::ffi::CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy())
    };

    let message = if callback_data.p_message.is_null() {
        String::from("")
    } else {
        String::from(std::ffi::CStr::from_ptr(callback_data.p_message).to_string_lossy())
    };

    
    let output = format!(
        "{message_severity:?}: {message_type:?} [{message_id_name} ({message_id_number})] : {message}",
    );

    // println!{"{}", output.color(match message_severity {
    //     vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => Color::BrightRed,
    //     vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => Color::Yellow,
    //     vk::DebugUtilsMessageSeverityFlagsEXT::INFO => Color::Blue,
    //     _ => Color::White,
    // })}

    vk::FALSE
}

