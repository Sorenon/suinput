use openxr::{
    sys::SessionCreateInfo, ApplicationInfo, Entry, ExtensionSet, FormFactor, SessionCreateFlags, Version, Instance,
};

#[test]
fn test() {
    // OpenXRDriver::new_headless();
}

struct OpenXRDriver {
    entry: Entry,

    instance: openxr::Instance,
    session: openxr::sys::Session,
}

impl OpenXRDriver {
    pub fn new(entry: Entry, instance: Instance, session: openxr::sys::Session) {
        


    }

    fn create_actions(instance: Instance, session: openxr::sys::Session) {
        
    }

    // pub fn new_headless() -> Self {
    //     let entry = Entry::load().unwrap();
    //     let mut exts = ExtensionSet::default();
    //     exts.mnd_headless = true;

    //     let instance = entry
    //         .create_instance(
    //             &ApplicationInfo {
    //                 application_name: "SuInput Headless",
    //                 application_version: 1,
    //                 engine_name: "",
    //                 engine_version: 0,
    //             },
    //             &exts,
    //             &[],
    //         )
    //         .unwrap();

    //     let system = instance.system(FormFactor::HEAD_MOUNTED_DISPLAY).unwrap();

    //     let session = unsafe {
    //         let mut session = openxr::sys::Session::NULL;

    //         let x = (instance.fp().create_session)(
    //             instance.as_raw(),
    //             &SessionCreateInfo {
    //                 ty: SessionCreateInfo::TYPE,
    //                 next: std::ptr::null(),
    //                 create_flags: SessionCreateFlags::EMPTY,
    //                 system_id: system,
    //             },
    //             &mut session,
    //         );

    //         if x.into_raw() >= 0 {
    //             Ok(session)
    //         } else {
    //             Err(x)
    //         }
    //     }
    //     .unwrap();

    //     Self {
    //         entry,
    //         instance,
    //         session,
    //     }
    // }
}
