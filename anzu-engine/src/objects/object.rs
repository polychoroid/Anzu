use std::default::Default;

pub struct ObjectSpec {
    body: BodyState,
    display: DisplayState,
    light: LightSpec,
    material: MaterialSpec,
}

impl ObjectSpec {
    pub fn new(
        body: Option<BodyState>,
        display: Option<DisplayState>,
        light: Option<LightSpec>,
        material: Option<MaterialSpec>,
    ) -> self {
        Self {}
    }
}

pub struct ObjectRuntime {}

pub fn get_default_catalog() -> vec![ObjectSpec] {
    // let specs = vec![

    // ]
}
