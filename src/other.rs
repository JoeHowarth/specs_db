use specs::prelude::*;
use crate::ResourceTable;
use specs::shred::{Fetch, FetchMut, Accessor, AccessorCow, CastFrom, DynamicSystemData, MetaTable, Read, Resource,
                   ResourceId, Resources,
                   System, SystemData};
use specs::shred::cell::{Ref, RefMut};
use crate::serialize::Serialize;
use std::ops::{Deref, DerefMut};
use specs::storage::MaskedStorage;
use std::fmt::Debug;


pub trait DynRegister {
    fn dyn_register<R: Resource + Serialize>(&mut self, r: R, name: &str);
    fn dyn_register_component<C: Debug + Component + Resource + Serialize>(&mut self, name: &str)
        where C::Storage: Default;
}

impl DynRegister for World {
    fn dyn_register<R: Resource + Serialize>(&mut self, mut r: R, name: &str) {
        {
            let (mut meta, mut res_table): (WriteExpect<MetaTable<Serialize>>, WriteExpect<ResourceTable>) = SystemData::fetch(&mut self.res);

            meta.register(&mut r);
            res_table.register::<R>(name);
        }
        self.add_resource(r);
    }

    fn dyn_register_component<C: Debug + Component + Resource + Serialize>(&mut self, name: &str)
        where C::Storage: Default
    {
        self.register::<C>();
        self.res.fetch_mut::<MetaTable<Serialize>>().register(&*self.res.fetch::<MaskedStorage<C>>());
        self.res.fetch_mut::<ResourceTable>().register::<MaskedStorage<C>>(name);
    }
}

pub fn fetch_serializable_by_string<T>(reads: &[&str], res: &Resources, f: fn(&Serialize) -> T) -> Vec<T> {
    let (meta, res_table): (ReadExpect<MetaTable<Serialize>>, ReadExpect<ResourceTable>) = SystemData::fetch(res);

    let mut xs: Vec<RefMut<Box<Resource>>> = reads
        .into_iter()
        .map(|&s| res_table.get(s))
        .map(|id| res
            .try_fetch_internal(id.0)
            .expect("bug: the requested resource does not exist")
            .borrow_mut()
        ).collect();

    xs.iter_mut()
      .map(|x: &mut RefMut<Box<Resource>>| x.as_ref())
      .map(|r: &Resource| meta
          .get(r)
          .expect("Not in meta_table"))
      .map(f)
      .collect()
}

pub fn test_dyn_component(res: &mut Resources) -> String {
    let table: Fetch<ResourceTable> = res.fetch::<ResourceTable>();
    let hill = table.get("Hill");
    let r = res.try_fetch_internal(hill.0)
               .expect("bug dyn")
               .borrow_mut();
    let r1 = r.as_ref();
    let meta = res.fetch::<MetaTable<Serialize>>();
    let s = meta.get(r1).expect("bug dyn 2");
    dbg!(s.component(res))
}


/*
pub fn fetch_by_string<'a, T, R>(reads: &[&str], res: &Resources, f: fn(&T) -> R) -> Vec<R> {
    let (meta, res_table): (ReadExpect<'a, MetaTable<T>>, ReadExpect<ResourceTable>) = SystemData::fetch(res);

    let xs: Vec<RefMut<Box<Resource>>> = reads
        .into_iter()
        .map(|&s| res_table.get(s))
        .map(|id| res
            .try_fetch_internal(id.0)
            .expect("bug: the requested resource does not exist")
            .borrow_mut()
        ).collect();

    xs.iter()
      .map(|r| {
          meta.get(Box::as_ref(r)).expect("Not in meta_table")
      })
      .map(f)
      .collect()
}
*/
