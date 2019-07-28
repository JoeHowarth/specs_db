#![allow(unused_imports, unused_variables, dead_code)]

use specs::prelude::*;
use specs::shred::{Accessor, AccessorCow, CastFrom, DynamicSystemData, MetaTable, Read, Resource,
                   ResourceId, Resources,
                   System, SystemData};
use specs::shred::cell::{Ref, RefMut};
use std::ops::Deref;
use std::collections::HashMap;
use std::panic::resume_unwind;
use crate::other::{DynRegister, fetch_serializable_by_string, test_dyn_component};
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
    world.register::<Tile>();
    world.register::<Hill>();

    world.create_entity().with(Tile { x: 4 }).with(Hill { y: 9 }).build();
    world.create_entity().with(Tile { x: 1 }).build();
    world.create_entity().with(Hill { y: 1 }).build();

    world.dyn_register_component::<Hill>("Hill");

    silly_join(&mut world);

    test_dyn_component(&mut world.res);


    world.dyn_register(Id(54), "Id");
    world.dyn_register((), "Unit");

    let serializables = fetch_serializable_by_string(
        &["Id", "Unit"],
        &world.res,
        |s| s.to_wire_format(),
    );
    serializables.iter()
                 .for_each(|s| { dbg!(s); });


    /*
    {
        let (meta, res_table): (ReadExpect<MetaTable<AnyStorage>>, ReadExpect<ResourceTable>) = SystemData::fetch(&world.res);
        let reads = &["Tile", "Hill"];

        let xs = reads
            .into_iter()
            .map(|&s| res_table.get(s))
            .map(|id| &world.res
                .try_fetch_internal(id.0)
                .expect("bug: the requested resource does not exist")
                .borrow_mut()
            ).collect::<Vec<_>>();

        xs.iter()
          .map(Box::as_ref)
          .map(|r| meta
              .get(r)
              .expect("Not in meta_table"))
          .map(f)
          .collect()
    }*/
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

#[derive(Debug)]
struct Tile {
    pub x: usize
}

#[derive(Debug)]
struct Hill {
    pub y: usize
}

impl Serialize for Hill {
    fn to_wire_format(&self) -> String {
        self.y.to_string() + " is my hill's height"
    }
}

impl Component for Hill {
    type Storage = specs::storage::VecStorage<Hill>;
}

impl Component for Tile {
    type Storage = specs::storage::VecStorage<Tile>;
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
    fn component(&self, res: &Resources)-> String {
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

