#![allow(unused_imports, unused_variables, dead_code)]

use specs::prelude::*;
use specs::shred::{Accessor, AccessorCow, CastFrom, DynamicSystemData, MetaTable, Read, Resource,
                   ResourceId, Resources,
                   System, SystemData};
use specs_derive::*;
use specs::shred::cell::{Ref, RefMut};
use std::ops::Deref;
use std::collections::HashMap;
use std::panic::resume_unwind;
use crate::other::{DynRegister, fetch_serializable_by_string, test_dyn_component, GivesBitSet, test_dyn_join};
use std::process::Command;
use specs::storage::{AnyStorage, MaskedStorage};
use crate::serialize::{Serialize, Id};
use specs::world::EntitiesRes;
use std::marker::PhantomData;
use std::fmt::Debug;

mod other;
mod serialize;
mod fake_system;

fn main() {
    let mut world = World::new();
    let meta_serialize_table: MetaTable<Serialize> = MetaTable::new();
    let resource_table = ResourceTable::new();

    world.add_resource(resource_table);
    world.add_resource(meta_serialize_table);
    world.add_resource(MetaTable::<GivesBitSet>::new());
    world.register::<Tile>();
    world.register::<Hill>();

    world.create_entity().with(Tile { x: 4 }).with(Hill { y: 9 }).build();
    world.create_entity().with(Tile { x: 10 }).with(Hill { y: 2 }).build();
    world.create_entity().with(Tile { x: 1 }).build();
    world.create_entity().with(Hill { y: 1 }).build();

    world.dyn_register_component::<Hill>("Hill");
    world.dyn_register_component::<Tile>("Tile");

    silly_join(&mut world);
    test_dyn_component(&mut world.res);
    test_dyn_join(&mut world.res);

    world.dyn_register(Id(54), "Id");
    world.dyn_register((), "Unit");

    let serializables = fetch_serializable_by_string(
        &["Id", "Unit"],
        &world.res,
        |s| s.to_wire_format(),
    );
    serializables.iter()
                 .for_each(|s| { dbg!(s); });
}

fn silly_join(world: &mut World) {
    let (h, t): (ReadStorage<Hill>, ReadStorage<Tile>) = world.system_data();
    let j = (&h, &t).join();
}

struct Query<'a> {
    joins: &'a [&'a str],
    keyed: &'a [usize],
}

impl<'a> Query<'a> {
    fn new(joins: &'a [&str], keyed: &'a [usize]) -> Self {
        Query { joins, keyed }
    }

    fn accept(res: &Resources) {
        unimplemented!()
    }
}

#[derive(Component, Debug)]
struct Tile {
    pub x: usize
}

#[derive(Component, Debug)]
struct Hill {
    pub y: usize
}

impl Serialize for Hill {
    fn to_wire_format(&self) -> String {
        self.y.to_string() + " is my hill's height"
    }
}

impl Serialize for Tile {
    fn to_wire_format(&self) -> String {
        self.x.to_string() + " is x from Tile"
    }
}

//impl<'e, T, D> Serialize for Storage<'e, T, D> {
//    fn to_wire_format(&self) -> String {
//        "constant".to_owned()
//    }
//}

impl<T: Debug + Component + Serialize> Serialize for MaskedStorage<T> {
    fn to_wire_format(&self) -> String {
        "constant".to_owned()
    }
    fn component(&self, res: &Resources) -> String {
        let c = Storage::new(res.fetch(), self);
        let mut s = String::new();
        for x in c.join() {
            dbg!(x);
            s += " ";
            s += &x.to_wire_format();
        }
        s
    }
}

impl<A: Debug + Component + Serialize, B: Debug + Component + Serialize> Serialize for (MaskedStorage<A>, MaskedStorage<B>) {
    fn to_wire_format(&self) -> String {
        "constant".to_owned()
    }
    fn component(&self, res: &Resources) -> String {
        let (a, b) = self;
        let store_a = Storage::new(res.fetch(), a);
        let store_b = Storage::new(res.fetch(), b);
        for (aa, bb) in (&store_a, &store_b).join() {
            dbg!(aa);
            dbg!(bb);
        }

        "hi".to_owned()
    }
}

struct ResourceTable {
    map: HashMap<String, ResourceId>
}

impl ResourceTable {
    fn new() -> Self {
        ResourceTable {
            map: HashMap::default(),
        }
    }

    fn register<T: Resource>(&mut self, name: &str) {
        self.map.insert(name.to_owned(), ResourceId::new::<T>());
    }

    fn get(&self, name: &str) -> ResourceId {
        *self.map.get(name).unwrap()
    }
}

