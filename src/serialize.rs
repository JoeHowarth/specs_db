use std::borrow::ToOwned;
use specs::shred::CastFrom;
use specs::world::EntitiesRes;
use specs::Resources;

pub trait Serialize {
    fn to_wire_format(&self) -> String {
        "fake json!!".to_owned()
    }
    fn component(&self, res: &Resources)-> String { "not_implemented".to_owned() }
}

impl<T> CastFrom<T> for Serialize
    where T: Serialize + 'static
{
    fn cast(t: &T) -> &Self { t }
    fn cast_mut(t: &mut T) -> &mut Self { t }
}

#[derive(Debug)]
pub struct Id(pub u8);

impl Serialize for Id {
    fn to_wire_format(&self) -> String {
        self.0.to_string() + ": is my id!!"
    }
}

impl Serialize for () {
    fn to_wire_format(&self) -> String {
        "I'm a tuple!".to_owned()
    }
}
