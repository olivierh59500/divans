use core;
use ::alloc;
use super::interface::{c_void, CAllocator};
#[cfg(not(feature="no-stdlib"))]
use std::vec::Vec;
#[cfg(not(feature="no-stdlib"))]
pub use std::boxed::Box;

#[cfg(not(feature="no-stdlib"))]
#[derive(Debug)]
pub struct MemoryBlock<Ty:Sized+Default>(Box<[Ty]>);
#[cfg(not(feature="no-stdlib"))]
impl<Ty:Sized+Default> Default for MemoryBlock<Ty> {
    fn default() -> Self {
        MemoryBlock(Vec::<Ty>::new().into_boxed_slice())
    }
}
#[cfg(not(feature="no-stdlib"))]
impl<Ty:Sized+Default> alloc::SliceWrapper<Ty> for MemoryBlock<Ty> {
    fn slice(&self) -> &[Ty] {
        &self.0[..]
    }
}
#[cfg(not(feature="no-stdlib"))]
impl<Ty:Sized+Default> alloc::SliceWrapperMut<Ty> for MemoryBlock<Ty> {
    fn slice_mut(&mut self) -> &mut [Ty] {
        &mut self.0[..]
    }
}
#[cfg(not(feature="no-stdlib"))]
impl<Ty:Sized+Default> core::ops::Index<usize> for MemoryBlock<Ty> {
    type Output = Ty;
    fn index(&self, index:usize) -> &Ty {
        &self.0[index]
    }
}
#[cfg(not(feature="no-stdlib"))]
impl<Ty:Sized+Default> core::ops::IndexMut<usize> for MemoryBlock<Ty> {

    fn index_mut(&mut self, index:usize) -> &mut Ty {
        &mut self.0[index]
    }
}
#[cfg(not(feature="no-stdlib"))]
impl<Ty:Sized+Default> Drop for MemoryBlock<Ty> {
    fn drop (&mut self) {
        if self.0.len() != 0 {
            print!("leaking memory block of length {} element size: {}\n", self.0.len(), core::mem::size_of::<Ty>());

            let to_forget = core::mem::replace(self, MemoryBlock::default());
            core::mem::forget(to_forget);// leak it -- it's the only safe way with custom allocators
        }
    }
}
pub struct SubclassableAllocator<Ty:Sized+Default> {
    _ty: core::marker::PhantomData<Ty>,
    alloc: CAllocator
    // have alternative ty here
}

impl<Ty:Sized+Default+Clone> SubclassableAllocator<Ty> {
    pub fn new(sub_alloc:CAllocator) -> Self {
        SubclassableAllocator::<Ty>{
            _ty:core::marker::PhantomData::<Ty>::default(),
            alloc:sub_alloc,
        }
    }
}
#[cfg(not(feature="no-stdlib"))]
impl<Ty:Sized+Default+Clone> alloc::Allocator<Ty> for SubclassableAllocator<Ty> {
    type AllocatedMemory = MemoryBlock<Ty>;
    fn alloc_cell(&mut self, size:usize) ->MemoryBlock<Ty>{
        if let Some(alloc_fn) = self.alloc.alloc_func {
            let ptr = alloc_fn(self.alloc.opaque, size * core::mem::size_of::<Ty>());
            let typed_ptr = unsafe {core::mem::transmute::<*mut c_void, *mut Ty>(ptr)};
            let slice_ref = unsafe {core::slice::from_raw_parts_mut(typed_ptr, size)};
            for item in slice_ref.iter_mut() {
                unsafe{core::ptr::write(item, Ty::default())};
            }
            return MemoryBlock(unsafe{Box::from_raw(slice_ref)})
        }
        MemoryBlock(vec![Ty::default();size].into_boxed_slice())
    }
    fn free_cell(&mut self, mut bv:MemoryBlock<Ty>) {
        if (*bv.0).len() != 0 {
            if let Some(_) = self.alloc.alloc_func {
                let slice_ptr = (*bv.0).as_mut_ptr();
                let _box_ptr = Box::into_raw(core::mem::replace(&mut bv.0, Vec::<Ty>::new().into_boxed_slice()));
                if let Some(free_fn) = self.alloc.free_func {
                    unsafe {free_fn(self.alloc.opaque, core::mem::transmute::<*mut Ty, *mut c_void>(slice_ptr))};
                }
            } else {
                let _to_free = core::mem::replace(&mut bv.0, Vec::<Ty>::new().into_boxed_slice());
            }
        }
    }
}











#[cfg(feature="no-stdlib")]
static mut G_SLICE:&mut[u8] = &mut[];
#[cfg(feature="no-stdlib")]
#[derive(Debug)]
pub struct MemoryBlock<Ty:Sized+Default>(*mut[Ty]);
#[cfg(feature="no-stdlib")]
impl<Ty:Sized+Default> Default for MemoryBlock<Ty> {
    fn default() -> Self {
        MemoryBlock(unsafe{core::mem::transmute::<*mut [u8], *mut[Ty]>(G_SLICE.as_mut())})
    }
}
#[cfg(feature="no-stdlib")]
impl<Ty:Sized+Default> alloc::SliceWrapper<Ty> for MemoryBlock<Ty> {
    fn slice(&self) -> &[Ty] {
        if unsafe{(*self.0).len()} == 0 {
            &[]
        } else {
            unsafe{core::slice::from_raw_parts(&(*self.0)[0], (*self.0).len())}
        }
    }
}
#[cfg(feature="no-stdlib")]
impl<Ty:Sized+Default> alloc::SliceWrapperMut<Ty> for MemoryBlock<Ty> {
    fn slice_mut(&mut self) -> &mut [Ty] {
        if unsafe{(*self.0).len()} == 0 {
            &mut []
        } else {
            unsafe{core::slice::from_raw_parts_mut(&mut (*self.0)[0], (*self.0).len())}
        }
    }
}

#[cfg(feature="no-stdlib")]
#[cfg(not(feature="no-stdlib-rust-binding"))]
#[lang="panic_fmt"]
extern fn panic_fmt(_: ::core::fmt::Arguments, _: &'static str, _: u32) -> ! {
    loop {}
}
#[cfg(feature="no-stdlib")]
#[cfg(not(feature="no-stdlib-rust-binding"))]
#[lang = "eh_personality"]
extern "C" fn eh_personality() {
}

#[cfg(feature="no-stdlib")]
impl<Ty:Sized+Default> core::ops::Index<usize> for MemoryBlock<Ty> {
    type Output = Ty;
    fn index(&self, index:usize) -> &Ty {
        unsafe{&(*self.0)[index]}
    }
}
#[cfg(feature="no-stdlib")]
impl<Ty:Sized+Default> core::ops::IndexMut<usize> for MemoryBlock<Ty> {

    fn index_mut(&mut self, index:usize) -> &mut Ty {
        unsafe{&mut (*self.0)[index]}
    }
}

#[cfg(feature="no-stdlib")]
impl<Ty:Sized+Default+Clone> alloc::Allocator<Ty> for SubclassableAllocator<Ty> {
    type AllocatedMemory = MemoryBlock<Ty>;
    fn alloc_cell(&mut self, size:usize) ->MemoryBlock<Ty>{
        if let Some(alloc_fn) = self.alloc.alloc_func {
            let ptr = alloc_fn(self.alloc.opaque, size * core::mem::size_of::<Ty>());
            let typed_ptr = unsafe {core::mem::transmute::<*mut c_void, *mut Ty>(ptr)};
            let slice_ref = unsafe {core::slice::from_raw_parts_mut(typed_ptr, size)};
            for item in slice_ref.iter_mut() {
                unsafe{core::ptr::write(item, Ty::default())};
            }
            return MemoryBlock(slice_ref.as_mut())
        } else {
            panic!("Must provide allocators in no-stdlib code");
        }
    }
    fn free_cell(&mut self, mut bv:MemoryBlock<Ty>) {
        use alloc::SliceWrapper;
        use alloc::SliceWrapperMut;
        if bv.slice().len() != 0 {
            if let Some(_) = self.alloc.alloc_func {
                if let Some(free_fn) = self.alloc.free_func {
                    unsafe {free_fn(self.alloc.opaque, core::mem::transmute::<*mut Ty, *mut c_void>(&mut bv.slice_mut()[0]))};
                }
                core::mem::replace(&mut bv, MemoryBlock::<Ty>::default());
            } else {
                panic!("Must provide allocators in no-stdlib code");
            }
        }
    }
}


#[cfg(feature="no-stdlib")]
pub fn free_stdlib<T>(_data: *mut T, _size: usize) {
    panic!("Must supply allocators if calling divans when compiled with features=no-stdlib");
}
#[cfg(feature="no-stdlib")]
pub fn alloc_stdlib<T:Sized+Default+Copy+Clone>(_size: usize) -> *mut T {
    panic!("Must supply allocators if calling divans when compiled with features=no-stdlib");
}

#[cfg(not(feature="no-stdlib"))]
pub unsafe fn free_stdlib<T>(ptr: *mut T, size: usize) {
    let slice_ref = core::slice::from_raw_parts_mut(ptr, size);
    Box::from_raw(slice_ref); // free on drop
}
#[cfg(not(feature="no-stdlib"))]
pub fn alloc_stdlib<T:Sized+Default+Copy+Clone>(size: usize) -> *mut T {
    let mut newly_allocated = vec![T::default();size].into_boxed_slice();
    let slice_ptr = newly_allocated.as_mut_ptr();
    let _box_ptr = Box::into_raw(newly_allocated);
    slice_ptr
}
