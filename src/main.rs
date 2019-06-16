use specs::prelude::*;
use specs::shred::{Accessor, AccessorCow, CastFrom, DynamicSystemData, MetaTable, Read, Resource,
                   ResourceId, Resources,
                   System, SystemData};
use specs::shred::cell::{Ref, RefMut};
use std::ops::Deref;
use std::collections::HashMap;
use std::panic::resume_unwind;
use crate::other::{DynRegister, fetch_serializable_by_string};

mod other;

fn main() {
    let mut world = World::new();
    let mut meta_table = MetaTable::<Serialize>::new();
    let mut resource_table = ResourceTable::new();

    world.add_resource(resource_table);
    world.add_resource(meta_table);
    world.dyn_register(Id(54), "Id");
    world.dyn_register((), "Unit");

//    let mut dispatch = DispatcherBuilder::new()
//        .with(FakeSystem {
//            dependencies: Dependencies {
//                reads: vec![
//                    resource_table.get("Id"),
//                    resource_table.get("Unit"),
//                ],
//                writes: vec![],
//            }
//        }, "fakesystem", &[])
//        .build();


    let serializables = fetch_serializable_by_string(
        &["Id", "Unit"],
        &world.res,
        |s| s.to_wire_format(),
    );
    serializables.iter().for_each(|s| { dbg!(s); });

    //dispatch.dispatch(&world.res);
}

struct FakeSystemData<'a> {
    meta_table: Read<'a, MetaTable<Serialize>>,
    reads: Vec<Ref<'a, Box<Resource + 'static>>>,
    writes: Vec<RefMut<'a, Box<Resource + 'static>>>,
}

impl<'a> DynamicSystemData<'a> for FakeSystemData<'a> {
    type Accessor = Dependencies;

    fn setup(_accessor: &Self::Accessor, _res: &mut Resources) {}

    fn fetch(accessor: &Dependencies, res: &'a Resources) -> Self {
        let reads = accessor
            .reads
            .iter()
            .map(|id| id.0)
            .map(|id| res
                .try_fetch_internal(id)
                .expect("bug: the requested resource does not exist")
                .borrow()
            )
            .collect();
        let writes = accessor
            .writes
            .iter()
            .map(|id| id.0)
            .map(|id| res
                .try_fetch_internal(id)
                .expect("bug: the requested resource does not exist")
                .borrow_mut()
            )
            .collect();

        FakeSystemData {
            meta_table: SystemData::fetch(res),
            reads,
            writes,
        }
    }
}

struct Dependencies {
    reads: Vec<ResourceId>,
    writes: Vec<ResourceId>,
}

impl Accessor for Dependencies {
    fn try_new() -> Option<Self> {
        // there's no default for this
        None
    }

    fn reads(&self) -> Vec<ResourceId> {
        let mut reads = self.reads.clone();
        reads.push(ResourceId::new::<MetaTable<Serialize>>());
        reads
    }

    fn writes(&self) -> Vec<ResourceId> {
        self.writes.clone()
    }
}

trait Serialize {
    fn to_wire_format(&self) -> String {
        "fake json!!".to_owned()
    }
}

impl<T> CastFrom<T> for Serialize
    where T: Serialize + 'static
{
    fn cast(t: &T) -> &Self { t }
    fn cast_mut(t: &mut T) -> &mut Self { t }
}

struct FakeSystem {
    dependencies: Dependencies
}

impl<'a> System<'a> for FakeSystem {
    type SystemData = FakeSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        dbg!(data.reads.len());

        let meta = data.meta_table;
        data.reads.iter()
            .map(|resource| {
                meta.get(Box::as_ref(resource)).expect("Not registered in meta table")
            })
            .for_each(|s| { dbg!(s.to_wire_format()); });
    }

    fn accessor<'b>(&'b self) -> AccessorCow<'a, 'b, Self> {
        AccessorCow::Ref(&self.dependencies)
    }

    fn setup(&mut self, _res: &mut Resources) {
        // this could call a setup function of the script
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

#[derive(Debug)]
struct Id(pub u8);

//impl Id {
//    pub fn say_hi(&self) -> String {
//        self.0.to_string() + "you're nice!"
//    }
//}

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
