<div align="center">

# `vmt-hook`

[![Crates.io][crates-badge]][crates-url]
[![docs.rs][docs-badge]][docs-url]
[![License][license-badge]][license-url]

</div>

This library provides the ability to hook Virtual Method Tables (VMT).

It works by copying the original VMT and then swapping it out with the modified version.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
vmt-hook = { version = "0.2.0" }
```

## Example

- Hooking the 'Present' method in DirectX 9.

```rust
use vmt_hook::VTableHook;

use windows::{
    core::HRESULT,
    Win32::{
        Foundation::{HWND, RECT},
        Graphics::{
            Direct3D9::IDirect3DDevice9,
            Gdi::RGNDATA,
        },
    },
};

type FnPresent = extern "stdcall" fn(
    IDirect3DDevice9,
    *const RECT,
    *const RECT,
    HWND,
    *const RGNDATA,
) -> HRESULT;

static mut ORIGINAL_PRESENT: Option<FnPresent> = None;

extern "stdcall" fn hk_present(
    device: IDirect3DDevice9,
    source_rect: *const RECT,
    dest_rect: *const RECT,
    dest_window_override: HWND,
    dirty_region: *const RGNDATA,
) -> HRESULT {
    // Your code.

    unsafe {
        let original_present = ORIGINAL_PRESENT.unwrap();
        original_present(device, source_rect, dest_rect, dest_window_override, dirty_region)
    }
}

unsafe fn instal_d3d9_hook() {
    let device: IDirect3DDevice9 = /* Your ptr. */;

    // Creating a hook with automatic detection of the number of methods in its VMT.
    // let hook = VTableHook::new(device);

    // If you know the number of methods in the table, you can specify it manually.
    let hook = VTableHook::with_count(device, 119);

    // Getting the original method.
    ORIGINAL_PRESENT = Some(std::mem::transmute(hook.get_original_method(17)));

    // Replacing the method at index 17 in the VMT with our function.
    hook.replace_method(17, hk_present as usize);
}
```

<!-- Links -->
[crates-badge]: https://img.shields.io/crates/v/vmt-hook.svg
[crates-url]: https://crates.io/crates/vmt-hook

[docs-badge]: https://docs.rs/vmt-hook/badge.svg
[docs-url]: https://docs.rs/vmt-hook

[license-badge]: https://img.shields.io/crates/l/vmt-hook
[license-url]: ./LICENSE