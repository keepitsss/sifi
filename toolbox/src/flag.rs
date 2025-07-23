#[allow(private_bounds)]
pub trait Flag: Sealed {
    fn long_name(&self) -> String;
    fn short_name(&self) -> Option<String>;
    fn description(&self) -> Option<String>;

    fn full_properies(&self) -> FlagProperies {
        FlagProperies {
            long_name: self.long_name(),
            short_name: self.short_name(),
            description: self.description(),
        }
    }
}
pub(crate) trait Sealed {}
#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct FlagProperies {
    pub long_name: String,
    pub short_name: Option<String>,
    pub description: Option<String>,
}
impl FlagProperies {
    pub fn long_flag(&self) -> String {
        "--".to_string() + &self.long_name
    }
    pub fn short_flag(&self) -> Option<String> {
        self.short_name
            .as_ref()
            .map(|short| "-".to_string() + short)
    }
}

impl Sealed for ((&str,),) {}

impl Flag for ((&str,),) {
    fn long_name(&self) -> String {
        self.0.0.to_owned()
    }
    fn short_name(&self) -> Option<String> {
        None
    }
    fn description(&self) -> Option<String> {
        None
    }
}

impl Sealed for ((&str,), &str) {}

impl Flag for ((&str,), &str) {
    fn long_name(&self) -> String {
        self.0.0.to_owned()
    }
    fn short_name(&self) -> Option<String> {
        None
    }
    fn description(&self) -> Option<String> {
        Some(self.1.to_owned())
    }
}

impl Sealed for ((&str, &str),) {}

impl Flag for ((&str, &str),) {
    fn long_name(&self) -> String {
        self.0.0.to_owned()
    }
    fn short_name(&self) -> Option<String> {
        Some(self.0.1.to_owned())
    }
    fn description(&self) -> Option<String> {
        None
    }
}

impl Sealed for ((&str, &str), &str) {}

impl Flag for ((&str, &str), &str) {
    fn long_name(&self) -> String {
        self.0.0.to_owned()
    }
    fn short_name(&self) -> Option<String> {
        Some(self.0.1.to_owned())
    }
    fn description(&self) -> Option<String> {
        Some(self.1.to_owned())
    }
}
