macro_rules! id {
    ($t:ty) => {
        ::std::any::TypeId::of::<$t>()
    };
}

mod once_cell;
mod static_type_map;
mod untyped_box;

pub(crate) use once_cell::OnceCell;
pub(crate) use static_type_map::StaticTypeMap;
pub(crate) use untyped_box::UntypedBox;
