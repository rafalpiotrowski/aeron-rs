use crate::aeron::Aeron;

pub struct Archive {
	aeron: Option<Aeron>,
}

impl Archive {
	pub fn new() -> Self {
		Self {
			aeron: None
		}
	}
}

pub struct Context {}
