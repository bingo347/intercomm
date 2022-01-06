macro_rules! id {
    ($t:ty) => {
        ::std::any::TypeId::of::<$t>()
    };
}

mod static_type_map;

pub(crate) use static_type_map::StaticTypeMap;
