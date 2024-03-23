use openxr_sys::{
    loader,
    loader::{XrNegotiateApiLayerRequest, XrNegotiateLoaderInfo},
    Result,
};

use std::os::raw::c_char;

mod dispatch;
mod layer;

#[no_mangle]
pub unsafe extern "C" fn xrNegotiateLoaderApiLayerInterface(
    loader_info_ptr: *mut XrNegotiateLoaderInfo,
    _api_layer_name: *mut c_char,
    api_layer_request_ptr: *mut XrNegotiateApiLayerRequest,
) -> Result {
    println!("--> xrNegotiateLoaderApiLayerInterface");

    // if (apiLayerName && std::string_view(apiLayerName) != LAYER_NAME) {
    //     ErrorLog(fmt::format("Invalid apiLayerName \"{}\"\n", apiLayerName));
    //     return XR_ERROR_INITIALIZATION_FAILED;
    // }

    // let loader_info = loader_info

    assert!(!loader_info_ptr.is_null());
    assert!(!api_layer_request_ptr.is_null());

    // if loader_info_ptr.is_null() || api_layer_request_ptr.is_null() {
    //     println!("xrNegotiateLoaderApiLayerInterface validation failed");
    //     return Result::ERROR_INITIALIZATION_FAILED;
    // }

    let loader_info = &mut *loader_info_ptr;
    let api_layer_request = &mut *api_layer_request_ptr;

    assert!(loader_info.ty == XrNegotiateLoaderInfo::TYPE);
    assert!(loader_info.struct_version == XrNegotiateLoaderInfo::VERSION);
    assert!(loader_info.struct_size == std::mem::size_of::<XrNegotiateLoaderInfo>());
    assert!(api_layer_request.ty == XrNegotiateApiLayerRequest::TYPE);
    assert!(api_layer_request.struct_version == XrNegotiateApiLayerRequest::VERSION);
    assert!(api_layer_request.struct_size == std::mem::size_of::<XrNegotiateApiLayerRequest>());
    assert!(loader_info.min_interface_version <= loader::CURRENT_LOADER_API_LAYER_VERSION);
    assert!(loader_info.max_interface_version >= loader::CURRENT_LOADER_API_LAYER_VERSION);
    assert!(loader_info.max_interface_version <= loader::CURRENT_LOADER_API_LAYER_VERSION);
    assert!(loader_info.max_api_version >= openxr_sys::CURRENT_API_VERSION);
    assert!(loader_info.min_api_version <= openxr_sys::CURRENT_API_VERSION);

    // if loader_info.ty != XrNegotiateLoaderInfo::TYPE
    //     || loader_info.struct_version != XrNegotiateLoaderInfo::VERSION
    //     || loader_info.struct_size != std::mem::size_of::<XrNegotiateLoaderInfo>()
    //     || api_layer_request.ty != XrNegotiateApiLayerRequest::TYPE
    //     || api_layer_request.struct_version != XrNegotiateApiLayerRequest::VERSION
    //     || api_layer_request.struct_size != std::mem::size_of::<XrNegotiateApiLayerRequest>()
    //     || loader_info.min_interface_version > loader::CURRENT_LOADER_API_LAYER_VERSION
    //     || loader_info.max_interface_version < loader::CURRENT_LOADER_API_LAYER_VERSION
    //     || loader_info.max_interface_version > loader::CURRENT_LOADER_API_LAYER_VERSION
    //     || loader_info.max_api_version < openxr_sys::CURRENT_API_VERSION
    //     || loader_info.min_api_version > openxr_sys::CURRENT_API_VERSION
    // {
    //     println!("xrNegotiateLoaderApiLayerInterface validation failed");
    //     return Result::ERROR_INITIALIZATION_FAILED;
    // }

    // Setup our layer to intercept OpenXR calls.
    api_layer_request.layer_interface_version = loader::CURRENT_LOADER_API_LAYER_VERSION;
    api_layer_request.layer_api_version = openxr_sys::CURRENT_API_VERSION;
    api_layer_request.get_instance_proc_addr = Some(dispatch::xr_get_instance_proc_addr);
    api_layer_request.create_api_layer_instance = Some(dispatch::xr_create_api_layer_instance);
    // apiLayerRequest->getInstanceProcAddr = reinterpret_cast<PFN_xrGetInstanceProcAddr>(xrGetInstanceProcAddr);
    // apiLayerRequest->createApiLayerInstance = reinterpret_cast<PFN_xrCreateApiLayerInstance>(xrCreateApiLayerInstance);

    println!("<-- xrNegotiateLoaderApiLayerInterface");

    Result::SUCCESS
}
