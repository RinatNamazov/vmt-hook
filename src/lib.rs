//! This library provides the ability to hook Virtual Method Tables (VMT).
//! It works by copying the original VMT and then swapping it out with the modified version.

use std::cell::RefCell;

/// Represents a structure responsible for hooking and managing the virtual function table (VTable) of a given type.
///
/// # Example
///
/// ```rust
/// use vmt_hook::VTableHook;
///
/// use windows::{
///     core::HRESULT,
///     Win32::{
///         Foundation::{HWND, RECT},
///         Graphics::{
///             Direct3D9::IDirect3DDevice9,
///             Gdi::RGNDATA,
///         },
///     },
/// };
///
/// type FnPresent = extern "stdcall" fn(
///     IDirect3DDevice9,
///     *const RECT,
///     *const RECT,
///     HWND,
///     *const RGNDATA,
/// ) -> HRESULT;
///
/// static mut ORIGINAL_PRESENT: Option<FnPresent> = None;
///
/// extern "stdcall" fn hk_present(
///     device: IDirect3DDevice9,
///     source_rect: *const RECT,
///     dest_rect: *const RECT,
///     dest_window_override: HWND,
///     dirty_region: *const RGNDATA,
/// ) -> HRESULT {
///     // Your code.
///
///     unsafe {
///         let original_present = ORIGINAL_PRESENT.unwrap();
///         original_present(device, source_rect, dest_rect, dest_window_override, dirty_region)
///     }
/// }
///
/// unsafe fn instal_d3d9_hook() {
///     let device: IDirect3DDevice9 = /* Your ptr. */;
///
///     // Creating a hook with automatic detection of the number of methods in its VMT.
///     // let hook = VTableHook::new(device);
///
///     // If you know the number of methods in the table, you can specify it manually.
///     let hook = VTableHook::with_count(device, 119);
///
///     // Getting the original method.
///     ORIGINAL_PRESENT = Some(std::mem::transmute(hook.get_original_method(17)));
///
///     // Replacing the method at index 17 in the VMT with our function.
///     hook.hook_method(17, hk_present as usize);
/// }
/// ````
pub struct VTableHook<T> {
    /// Pointer to the object whose VTable is being hooked.
    object: T,
    /// Pointer to the original VTable.
    original_vtbl: *const usize,
    /// Count of methods in the VTable.
    count: usize,
    /// New VTable containing hooked function address.
    new_vtbl: RefCell<Vec<usize>>,
}

impl<T> Drop for VTableHook<T> {
    /// Restoring the original VTable.
    fn drop(&mut self) {
        unsafe {
            *std::mem::transmute_copy::<_, *mut *const usize>(&self.object) = self.original_vtbl;
        }
    }
}

impl<T> VTableHook<T> {
    /// Creates a new VTableHook instance for the provided object and replaces its VTable with the hooked VTable.
    /// The count of methods is automatically determined.
    pub unsafe fn new(object: T) -> Self {
        Self::init(object, |vtable| Self::detect_vtable_methods_count(vtable))
    }

    /// Creates a new VTableHook instance for the provided object with a specified method count
    /// and replaces its VTable with the hooked VTable.
    pub unsafe fn with_count(object: T, count: usize) -> Self {
        Self::init(object, |_| count)
    }

    unsafe fn init<F>(object: T, count_fn: F) -> Self
    where
        F: FnOnce(*const usize) -> usize
    {
        let object_ptr = std::mem::transmute_copy::<T, *mut *const usize>(&object);
        let original_vtbl = *object_ptr;
        let count = count_fn(original_vtbl);
        let new_vtbl = Self::create_vmt_copy(original_vtbl, count);

        *object_ptr = new_vtbl.borrow().as_ptr();

        Self {
            object,
            original_vtbl,
            count,
            new_vtbl,
        }
    }

    /// Creates a copy of the original VTable.
    unsafe fn create_vmt_copy(original_vtbl: *const usize, count: usize) -> RefCell<Vec<usize>> {
        let mut vtbl = Vec::with_capacity(count);

        std::ptr::copy_nonoverlapping(original_vtbl, vtbl.as_mut_ptr(), count);
        vtbl.set_len(count);

        RefCell::new(vtbl)
    }

    /// Detects the number of methods in the provided VTable.
    unsafe fn detect_vtable_methods_count(vtable: *const usize) -> usize {
        let mut vmt = vtable;

        while std::ptr::read(vmt) != 0 {
            vmt = vmt.add(1);
        }

        (vmt as usize - vtable as usize) / std::mem::size_of::<usize>()
    }

    /// Retrieves the original method address at the specified index in the VTable.
    pub fn get_original_method(&self, id: usize) -> usize {
        if id > self.count {
            panic!("index out of bounds");
        }
        unsafe { std::ptr::read(self.original_vtbl.add(id)) }
    }

    /// Retrieves the hooked method address at the specified index in the VTable.
    pub fn get_hook_method(&self, id: usize) -> usize {
        self.new_vtbl.borrow()[id]
    }

    /// Hooks the method at the specified index in the VTable with a new function address.
    pub unsafe fn hook_method(&self, id: usize, func: usize) {
        self.new_vtbl.borrow_mut()[id] = func;
    }

    /// Restores the original method at the specified index in the VTable.
    pub unsafe fn restore_method(&self, id: usize) {
        self.new_vtbl.borrow_mut()[id] = self.get_original_method(id);
    }

    /// Restores all methods in the VTable to their original address.
    pub unsafe fn restore_all_methods(&self) {
        std::ptr::copy_nonoverlapping(self.original_vtbl, self.new_vtbl.borrow_mut().as_mut_ptr(), self.count);
    }

    /// Returns the original object.
    pub fn object(&self) -> &T {
        &self.object
    }
}
