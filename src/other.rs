use specs::prelude::*;
use crate::ResourceTable;
use specs::shred::{Fetch, FetchMut, Accessor, AccessorCow, CastFrom, DynamicSystemData, MetaTable, Read, Resource,
                   ResourceId, Resources,
                   System, SystemData};
use specs::shred::cell::{Ref, RefMut};
use crate::serialize::Serialize;
use std::ops::{Deref, DerefMut};
use specs::storage::{MaskedStorage, UnprotectedStorage};
use std::fmt::Debug;
use std::marker::PhantomData;
use core::borrow::BorrowMut;
use hibitset::BitSetLike;


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
        self.res.fetch_mut::<MetaTable<GivesBitSet>>().register(&*self.res.fetch::<MaskedStorage<C>>());
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
    let table = res.fetch::<ResourceTable>();
    let meta = res.fetch::<MetaTable<Serialize>>();

    let hill = table.get("Hill");
    let r = res.try_fetch_internal(hill.0)
               .expect("bug dyn")
               .borrow_mut();
    let r1 = r.as_ref();
    let t = meta.get(r1).expect("bug dyn 2");
    dbg!(t.component(res))
}

pub fn test_dyn_join(res: &mut Resources) {
    let table = res.fetch::<ResourceTable>();
    let meta = res.fetch::<MetaTable<GivesBitSet>>();
    let vec = ["Hill", "Tile"].iter().map(|&s| {
        res.try_fetch_internal(table.get(s).0)
           .expect("bug")
           .borrow_mut()
    }).collect::<Vec<_>>();
    let vec2 = vec.iter().map(|r| r.as_ref())
                  .map(|r| meta.get(r).expect("bug"))
                  .collect::<Vec<_>>();
    let mut bitset = vec2[0].get_bit_set(res);
    for &e in &vec2 {
        bitset &= &e.get_bit_set(res);
    }
    for i in bitset {
       let s = vec2.iter().fold(String::new(), |acc, &e| acc + &e.get(res, i) );
       println!("{:?}", s);
    }
}


impl<T: Component + Debug> GivesBitSet for MaskedStorage<T> {
    fn get_bit_set(&self, res: &Resources) -> BitSet {
        let storage = Storage::new(res.fetch(), self);
        storage.mask().clone()
    }
    fn get(&self, res: &Resources, idx: u32) -> String {
        let storage = Storage::new(res.fetch(), self);
//        let raw: &UnprotectedStorage<T> = storage.unprotected_storage();
        format!("{:?}", unsafe { storage.unprotected_storage().get(idx) })
    }
}

pub trait GivesBitSet {
    fn get_bit_set(&self, res: &Resources) -> BitSet;
    fn get(&self, res: &Resources, idx: u32) -> String;
}

impl<T> CastFrom<T> for GivesBitSet
    where T: GivesBitSet + 'static
{
    fn cast(t: &T) -> &Self { t }
    fn cast_mut(t: &mut T) -> &mut Self { t }
}

/*
impl Join for DynWrapper {
    type Type = String;
    type Value = T::Storage;
    type Mask = BitSet;

    unsafe fn open(self) -> (Self::Mask, Self::Value) {
        unimplemented!()
    }

    unsafe fn get(value: &mut Self::Value, id: u32) -> Self::Type {
        unimplemented!()
    }
}
*/



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
