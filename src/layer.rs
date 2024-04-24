use std::{
    collections::HashMap,
    ffi::{c_char, CStr},
    time::{Duration, SystemTime},
};

use openxr_sys::{
    pfn, Action, ActionSpaceCreateInfo, ActionStateGetInfo, ActionStatePose, BaseOutStructure,
    ExtensionProperties, EyeGazeSampleTimeEXT, Instance, InteractionProfileSuggestedBinding, Path,
    Quaternionf, Result, Session, Space, SpaceLocation, SpaceLocationFlags, StructureType,
    SystemEyeGazeInteractionPropertiesEXT, SystemId, SystemProperties, Time, Vector3f,
};

use once_cell::sync::Lazy;

use crate::server::OSCServer;

pub static mut INSTANCE: Lazy<OpenXRLayer> = Lazy::new(OpenXRLayer::new);

struct Extension {
    name: &'static str,
    version: u32,
}

const ADVERTISED_EXTENSIONS: &[Extension] = &[Extension {
    name: "XR_EXT_eye_gaze_interaction",
    version: 1,
}];

pub struct OpenXRLayer {
    pub instance: Option<Instance>,
    pub get_instance_proc_addr: Option<pfn::GetInstanceProcAddr>,
    pub enumerate_instance_extensions_properties: Option<pfn::EnumerateInstanceExtensionProperties>,
    pub get_system_properties: Option<pfn::GetSystemProperties>,
    pub suggest_interaction_profile_bindings: Option<pfn::SuggestInteractionProfileBindings>,
    pub path_to_string: Option<pfn::PathToString>,
    pub create_action_space: Option<pfn::CreateActionSpace>,
    pub get_action_state_pose: Option<pfn::GetActionStatePose>,
    pub locate_space: Option<pfn::LocateSpace>,

    possible_spaces: HashMap<(Action, Path), Space>,

    eye_gaze_action: Option<Action>,
    l_eye_gaze_space: Option<Space>,
    r_eye_gaze_space: Option<Space>,

    start_time: SystemTime,

    server: OSCServer,
}

impl OpenXRLayer {
    pub fn new() -> OpenXRLayer {
        let res = OpenXRLayer {
            instance: None,
            get_instance_proc_addr: None,
            enumerate_instance_extensions_properties: None,
            get_system_properties: None,
            suggest_interaction_profile_bindings: None,
            path_to_string: None,
            eye_gaze_action: None,
            create_action_space: None,
            l_eye_gaze_space: None,
            r_eye_gaze_space: None,
            get_action_state_pose: None,
            locate_space: None,
            possible_spaces: HashMap::new(),
            start_time: SystemTime::now(),
            server: OSCServer::new(),
        };

        res.server.run();

        res
    }

    pub unsafe fn enumerate_instance_extension_properties(
        &self,
        layer_name: *const c_char,
        property_capacity_input: u32,
        property_count_output: *mut u32,
        properties_ptr: *mut ExtensionProperties,
    ) -> Result {
        let mut result = self.enumerate_instance_extensions_properties.unwrap()(
            layer_name,
            property_capacity_input,
            property_count_output,
            properties_ptr,
        );

        let base_offset = *property_count_output as usize;
        *property_count_output += ADVERTISED_EXTENSIONS.len() as u32;
        if property_capacity_input > 0 {
            if property_capacity_input < *property_count_output {
                result = Result::ERROR_SIZE_INSUFFICIENT;
            } else {
                result = Result::SUCCESS;

                let properties = std::slice::from_raw_parts_mut(
                    properties_ptr,
                    (*property_count_output).try_into().unwrap(),
                );

                for i in base_offset..*property_count_output as usize {
                    if properties[i].ty != StructureType::EXTENSION_PROPERTIES {
                        result = Result::ERROR_VALIDATION_FAILURE;
                        break;
                    }

                    let extension = &ADVERTISED_EXTENSIONS[i - base_offset];

                    std::ptr::copy(
                        extension.name.as_ptr(),
                        properties[i].extension_name.as_mut_ptr() as *mut u8,
                        extension.name.len(),
                    );
                    properties[i].extension_version = extension.version;
                }
            }
        }

        result
    }

    pub unsafe fn get_system_properties(
        &self,
        instance: Instance,
        system_id: SystemId,
        properties: *mut SystemProperties,
    ) -> Result {
        println!("--> get_system_properties");

        let mut property_ptr = properties as *mut BaseOutStructure;
        while !property_ptr.is_null() {
            let property = &mut *property_ptr;

            println!("get_system_properties type {:?}", property.ty);

            if property.ty == StructureType::SYSTEM_EYE_GAZE_INTERACTION_PROPERTIES_EXT {
                let property = &mut *(property_ptr as *mut SystemEyeGazeInteractionPropertiesEXT);
                property.supports_eye_gaze_interaction = true.into();
            }

            property_ptr = property.next;
        }

        let result = self.get_system_properties.unwrap()(instance, system_id, properties);
        if result != Result::SUCCESS {
            println!("get_system_properties result: {result:?}");
            return result;
        }

        println!("<-- get_system_properties");
        Result::SUCCESS
    }

    pub unsafe fn suggest_interaction_profile_bindings(
        &mut self,
        instance: Instance,
        suggested_bindings: *const InteractionProfileSuggestedBinding,
    ) -> Result {
        let suggested_bindings = &*suggested_bindings;

        let interaction_profile =
            self.path_to_string(instance, suggested_bindings.interaction_profile);

        println!(
            "suggest_interaction_profile_bindings {:?} {}",
            suggested_bindings, interaction_profile
        );

        if interaction_profile != "/interaction_profiles/ext/eye_gaze_interaction" {
            return self.suggest_interaction_profile_bindings.unwrap()(
                instance,
                suggested_bindings,
            );
        }

        let suggested_bindings = std::slice::from_raw_parts(
            suggested_bindings.suggested_bindings,
            suggested_bindings
                .count_suggested_bindings
                .try_into()
                .unwrap(),
        );

        for suggested_binding in suggested_bindings {
            let binding = self.path_to_string(instance, suggested_binding.binding);
            println!("suggest_interaction_profile_bindings binding path {binding}");
            if binding == "/user/eyes_ext/input/gaze_ext/pose" {
                self.eye_gaze_action = Some(suggested_binding.action);
                println!(
                    "suggest_interaction_profile_bindings saved eye gaze action {:?}",
                    suggested_binding.action
                );

                if let Some(l_eye_gaze_space) = self
                    .possible_spaces
                    // TODO: Don't hardcode "/user/hand/left" as Path(1)
                    .get(&(suggested_binding.action, Path::from_raw(1)))
                {
                    self.l_eye_gaze_space = Some(*l_eye_gaze_space);
                    println!("L eye gaze space found: {:?}", l_eye_gaze_space);
                }
                if let Some(r_eye_gaze_space) = self
                    .possible_spaces
                    // TODO: Don't hardcode "/user/hand/right" as Path(2)
                    .get(&(suggested_binding.action, Path::from_raw(2)))
                {
                    self.r_eye_gaze_space = Some(*r_eye_gaze_space);
                    println!("R eye gaze space found: {:?}", r_eye_gaze_space);
                }

                self.possible_spaces.clear();

                println!(
                    "test {:?} {:?}",
                    self.path_to_string(instance, Path::from_raw(1)), // "/user/hand/left"
                    self.path_to_string(instance, Path::from_raw(2)), // "/user/hand/right"
                );
            }
        }

        Result::SUCCESS
    }

    pub unsafe fn create_action_space(
        &mut self,
        session: Session,
        create_info: *const ActionSpaceCreateInfo,
        space: *mut Space,
    ) -> Result {
        println!("--> create_action_space {:?}", *create_info);
        let result = self.create_action_space.unwrap()(session, create_info, space);
        if result != Result::SUCCESS {
            return result;
        }

        // Spaced are created before actions, so save them all and choose later when the action is known.
        let create_info = &*create_info;
        self.possible_spaces
            .insert((create_info.action, create_info.subaction_path), *space);

        println!("<-- create_action_space");
        Result::SUCCESS
    }

    pub unsafe fn get_action_state_pose(
        &self,
        session: Session,
        get_info: *const ActionStateGetInfo,
        state: *mut ActionStatePose,
    ) -> Result {
        if !self
            .eye_gaze_action
            .is_some_and(|a| a == (*get_info).action)
        {
            return self.get_action_state_pose.unwrap()(session, get_info, state);
        }

        // println!("--> get_action_state_pose {:?}", (*get_info).subaction_path);

        let eye_gaze_data = self.server.eye_gaze_data.lock().unwrap();
        let state = &mut *state;

        // Report tracking as disabled if there is no data incoming.
        state.is_active =
            (eye_gaze_data.time.elapsed().unwrap() < Duration::from_millis(50)).into();

        // println!("<-- get_action_state_pose");
        Result::SUCCESS
    }

    pub unsafe fn locate_space(
        &self,
        space: Space,
        base_space: Space,
        time: Time,
        location: *mut SpaceLocation,
    ) -> Result {
        // println!("--> locate_space {:?} {:?} {:?}", space, base_space, time);

        let is_left = self.l_eye_gaze_space.is_some_and(|s| s == space);
        let is_right = self.r_eye_gaze_space.is_some_and(|s| s == space);

        if !is_left && !is_right {
            return self.locate_space.unwrap()(space, base_space, time, location);
        }

        // println!("locate_space {:?} {:?}", space, base_space);

        let location = &mut *location;

        location.location_flags |= SpaceLocationFlags::POSITION_TRACKED;
        location.location_flags |= SpaceLocationFlags::ORIENTATION_TRACKED;

        let eye_gaze_data = self.server.eye_gaze_data.lock().unwrap();

        let (pitch, yaw) = if is_left {
            (eye_gaze_data.l_pitch, eye_gaze_data.l_yaw)
        } else {
            (eye_gaze_data.r_pitch, eye_gaze_data.r_yaw)
        };

        use quaternion_core as quat;
        let q = quat::from_euler_angles(
            quat::RotationType::Extrinsic,
            quat::RotationSequence::XYZ,
            [pitch, yaw, 0.0],
        );

        // TODO: Figure out if this is correct position.
        // If eyeball position is required, can use `xrLocateView` to query camera position.
        location.pose.position = Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        location.pose.orientation = Quaternionf {
            w: q.0,
            x: q.1[0],
            y: q.1[1],
            z: q.1[2],
        };

        // println!("locate_space {:?}", location);

        if !location.next.is_null() {
            let eye_gaze_sample_time = &mut *(location.next as *mut EyeGazeSampleTimeEXT);
            eye_gaze_sample_time.time = Time::from_nanos(0);
            // println!("locate_space {:?}", eye_gaze_sample_time);
        }

        Result::SUCCESS
    }

    pub unsafe fn path_to_string(&self, instance: Instance, path: Path) -> String {
        let mut buffer = vec![0u8; 128];
        let mut out_size = 0u32;
        self.path_to_string.unwrap()(
            instance,
            path,
            buffer.len().try_into().unwrap(),
            &mut out_size as *mut u32,
            buffer.as_mut_ptr() as *mut i8,
        );

        CStr::from_bytes_until_nul(&buffer[..out_size as usize])
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }
}
