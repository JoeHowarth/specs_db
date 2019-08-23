use specs::shred::{Accessor, AccessorCow, CastFrom, DynamicSystemData, MetaTable, Read, Resource,
                   ResourceId, Resources,
                   System, SystemData};
use specs::shred::cell::{Ref, RefMut};
use crate::serialize::Serialize;


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
            .map(|id| res
                .try_fetch_internal(id.clone())
                .expect("bug: the requested resource does not exist")
                .borrow()
            )
            .collect();
        let writes = accessor
            .writes
            .iter()
            .map(|id| res
                .try_fetch_internal(id.clone())
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
