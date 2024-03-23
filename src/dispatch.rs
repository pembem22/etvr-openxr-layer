use std::ffi::c_char;
use std::ffi::CStr;

use crate::layer::INSTANCE;

use openxr_sys::ActionSpaceCreateInfo;
use openxr_sys::ActionStateGetInfo;
use openxr_sys::ActionStatePose;
use openxr_sys::ExtensionProperties;
use openxr_sys::InteractionProfileSuggestedBinding;
use openxr_sys::Session;
use openxr_sys::Space;
use openxr_sys::SpaceLocation;
use openxr_sys::SystemId;
use openxr_sys::SystemProperties;
use openxr_sys::Time;
use openxr_sys::{pfn, Instance, Result};

use openxr_sys::{loader::ApiLayerCreateInfo, InstanceCreateInfo};

pub unsafe extern "system" fn xr_create_api_layer_instance(
    instance_create_info_ptr: *const InstanceCreateInfo,
    api_layer_info_ptr: *const ApiLayerCreateInfo,
    instance: *mut Instance,
) -> Result {
    println!("--> xr_create_api_layer_instance");

    // Call the chain to create the instance.
    let mut chain_instance_create_info = *instance_create_info_ptr;

    // Hide our extension from the list assuming it's in the beginning.
    // This is to avoid an `ERROR_EXTENSION_NOT_PRESENT` error from the runtime.
    chain_instance_create_info.enabled_extension_count -= 1;
    chain_instance_create_info.enabled_extension_names =
        chain_instance_create_info.enabled_extension_names.add(1);

    let api_layer_info = *api_layer_info_ptr;
    let mut chain_api_layer_info = api_layer_info;
    chain_api_layer_info.next_info = (*api_layer_info.next_info).next;
    let result = ((*api_layer_info.next_info).next_create_api_layer_instance)(
        &chain_instance_create_info,
        &chain_api_layer_info,
        instance,
    );

    println!("xr_create_api_layer_instance result: {result:?}");

    if result == Result::SUCCESS {
        // Create our layer.
        INSTANCE.get_instance_proc_addr =
            Some((*api_layer_info.next_info).next_get_instance_proc_addr);
        INSTANCE.instance = Some(*instance);
    }

    println!("<-- xr_create_api_layer_instance");

    result
}

pub unsafe extern "system" fn xr_get_instance_proc_addr(
    instance: Instance,
    name_ptr: *const c_char,
    function: *mut Option<pfn::VoidFunction>,
) -> Result {
    let api_name = CStr::from_ptr(name_ptr);
    if instance.into_raw() == 0
        && !(api_name
            == CStr::from_bytes_with_nul_unchecked(b"xrEnumerateInstanceExtensionProperties\0")
            || api_name == CStr::from_bytes_with_nul_unchecked(b"xrEnumerateApiLayerProperties\0")
            || api_name == CStr::from_bytes_with_nul_unchecked(b"xrCreateInstance\0"))
    {
        return Result::ERROR_HANDLE_INVALID;
    }

    println!(
        "xr_get_instance_proc_addr {:?} {}",
        instance,
        CStr::from_ptr(name_ptr).to_str().unwrap()
    );

    let result = INSTANCE.get_instance_proc_addr.unwrap()(instance, name_ptr, function);

    if api_name == CStr::from_bytes_with_nul_unchecked(b"xrEnumerateInstanceExtensionProperties\0")
    {
        INSTANCE.enumerate_instance_extensions_properties = Some(std::mem::transmute::<
            pfn::VoidFunction,
            pfn::EnumerateInstanceExtensionProperties,
        >((*function).unwrap()));
        *function = Some(std::mem::transmute::<
            pfn::EnumerateInstanceExtensionProperties,
            pfn::VoidFunction,
        >(xr_enumerate_instance_extension_properties));
    }

    if api_name == CStr::from_bytes_with_nul_unchecked(b"xrGetSystemProperties\0") {
        INSTANCE.get_system_properties = Some(std::mem::transmute::<
            pfn::VoidFunction,
            pfn::GetSystemProperties,
        >((*function).unwrap()));
        *function = Some(std::mem::transmute::<
            pfn::GetSystemProperties,
            pfn::VoidFunction,
        >(xr_get_system_properties));
    }

    if api_name == CStr::from_bytes_with_nul_unchecked(b"xrSuggestInteractionProfileBindings\0") {
        INSTANCE.suggest_interaction_profile_bindings = Some(std::mem::transmute::<
            pfn::VoidFunction,
            pfn::SuggestInteractionProfileBindings,
        >((*function).unwrap()));
        *function = Some(std::mem::transmute::<
            pfn::SuggestInteractionProfileBindings,
            pfn::VoidFunction,
        >(xr_suggest_interaction_profile_bindings));
    }

    if api_name == CStr::from_bytes_with_nul_unchecked(b"xrCreateActionSpace\0") {
        INSTANCE.create_action_space = Some(std::mem::transmute::<
            pfn::VoidFunction,
            pfn::CreateActionSpace,
        >((*function).unwrap()));
        *function = Some(std::mem::transmute::<
            pfn::CreateActionSpace,
            pfn::VoidFunction,
        >(xr_create_action_space));
    }

    if api_name == CStr::from_bytes_with_nul_unchecked(b"xrGetActionStatePose\0") {
        INSTANCE.get_action_state_pose = Some(std::mem::transmute::<
            pfn::VoidFunction,
            pfn::GetActionStatePose,
        >((*function).unwrap()));
        *function = Some(std::mem::transmute::<
            pfn::GetActionStatePose,
            pfn::VoidFunction,
        >(xr_get_action_state_pose));
    }

    if api_name == CStr::from_bytes_with_nul_unchecked(b"xrLocateSpace\0") {
        INSTANCE.locate_space = Some(std::mem::transmute::<pfn::VoidFunction, pfn::LocateSpace>(
            (*function).unwrap(),
        ));
        *function = Some(std::mem::transmute::<pfn::LocateSpace, pfn::VoidFunction>(
            xr_locate_space,
        ));
    }

    if api_name == CStr::from_bytes_with_nul_unchecked(b"xrPathToString\0") {
        INSTANCE.path_to_string = Some(
            std::mem::transmute::<pfn::VoidFunction, pfn::PathToString>((*function).unwrap()),
        );
    }

    result
}

unsafe extern "system" fn xr_enumerate_instance_extension_properties(
    layer_name: *const c_char,
    property_capacity_input: u32,
    property_count_output: *mut u32,
    properties: *mut ExtensionProperties,
) -> Result {
    INSTANCE.enumerate_instance_extension_properties(
        layer_name,
        property_capacity_input,
        property_count_output,
        properties,
    )
}

unsafe extern "system" fn xr_get_system_properties(
    instance: Instance,
    system_id: SystemId,
    properties: *mut SystemProperties,
) -> Result {
    INSTANCE.get_system_properties(instance, system_id, properties)
}

unsafe extern "system" fn xr_suggest_interaction_profile_bindings(
    instance: Instance,
    suggested_bindings: *const InteractionProfileSuggestedBinding,
) -> Result {
    INSTANCE.suggest_interaction_profile_bindings(instance, suggested_bindings)
}

unsafe extern "system" fn xr_create_action_space(
    session: Session,
    create_info: *const ActionSpaceCreateInfo,
    space: *mut Space,
) -> Result {
    INSTANCE.create_action_space(session, create_info, space)
}

unsafe extern "system" fn xr_get_action_state_pose(
    session: Session,
    get_info: *const ActionStateGetInfo,
    state: *mut ActionStatePose,
) -> Result {
    INSTANCE.get_action_state_pose(session, get_info, state)
}

unsafe extern "system" fn xr_locate_space(
    space: Space,
    base_space: Space,
    time: Time,
    location: *mut SpaceLocation,
) -> Result {
    INSTANCE.locate_space(space, base_space, time, location)
}
