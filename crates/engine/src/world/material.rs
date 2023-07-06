use std::rc::Rc;

pub type MaterialHandle = Rc<Material>;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
    pub struct MaterialFlags: u16 {
        const PASSTHROUGH       = 0b000001;
        const SIGHTBLOCKER      = 0b000010;

        const SOLID             = 0b000010;
    }
}

#[non_exhaustive]
#[derive(Debug, PartialEq, Eq)]
pub struct Material {
    pub display_name: String,
    pub resource_name: String,
    pub obscured_resource_name: Option<String>,
    pub flags: MaterialFlags,
}

impl Material {
    pub fn new(
        display_name: impl ToString,
        resource_name: impl ToString,
        obscured_resource_name: Option<impl ToString>,
        flags: MaterialFlags,
    ) -> MaterialHandle {
        Rc::new(Material {
            display_name: display_name.to_string(),
            resource_name: resource_name.to_string(),
            obscured_resource_name: obscured_resource_name.as_ref().map(ToString::to_string),
            flags,
        })
    }
}