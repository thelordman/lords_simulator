#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Sex {
    Male,
    Female,
}

pub struct Name {
    first: String,
    middle: Vec<String>,
    last: String,
}

pub fn start(name: Name, sex: Sex) {
    
}