use specs::prelude::*;
use crate::{Serialize, ResourceTable};
use specs::shred::{Accessor, AccessorCow, CastFrom, DynamicSystemData, MetaTable, Read, Resource,
                   ResourceId, Resources,
                   System, SystemData};
use specs::shred::cell::{Ref, RefMut};


pub trait DynRegister {
    fn dyn_register<R: Resource + Serialize>(&mut self, r: R, name: &str);
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
}

pub fn fetch_serializable_by_string<T>(reads: &[&str], res: &Resources, f: fn(&Serialize) -> T) -> Vec<T> {
    let (meta, res_table): (ReadExpect<MetaTable<Serialize>>, ReadExpect<ResourceTable>) = SystemData::fetch(res);

    let xs: Vec<_> = reads
        .into_iter()
        .map(|&s| res_table.get(s))
        .map(|id| res
            .try_fetch_internal(id.0)
            .expect("bug: the requested resource does not exist")
            .borrow_mut()
        ).collect();

    xs.iter()
      .map(Box::as_ref)
      .map(|r| meta.get(r)
                   .expect("Not in meta_table"))
      .map(f)
      .collect()
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
