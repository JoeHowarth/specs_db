use std::borrow::ToOwned;
use specs::shred::CastFrom;
use specs::world::EntitiesRes;
use specs::World;

pub trait Serialize {
    fn to_wire_format(&self) -> String {
        "fake json!!".to_owned()
    }
    fn component(&self, res: &World)-> String { "not_implemented".to_owned() }
}

unsafe impl<T> CastFrom<T> for dyn Serialize
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
