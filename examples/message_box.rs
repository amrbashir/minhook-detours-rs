#![allow(non_snake_case)]

use minhook_detours::*;
use windows_sys::{
    Win32::{
        Foundation::HWND,
        System::LibraryLoader::{GetModuleHandleW, GetProcAddress},
        UI::WindowsAndMessaging::{MESSAGEBOX_STYLE, MessageBoxW},
    },
    core::PCWSTR,
    s, w,
};

type MessageBoxWFn = unsafe extern "system" fn(
    hwnd: HWND,
    lptext: PCWSTR,
    lpcaption: PCWSTR,
    utype: MESSAGEBOX_STYLE,
) -> i32;

static mut ORIGINAL_MESSAGE_BOX_W_FN: MessageBoxWFn = DetourMessageBoxW;

extern "system" fn DetourMessageBoxW(
    hwnd: HWND,
    _lptext: PCWSTR,
    _lpcaption: PCWSTR,
    utype: MESSAGEBOX_STYLE,
) -> i32 {
    let new_text = w!("Hooked Message Box!");
    let new_caption = w!("Hooked Caption");

    unsafe {
        // call the typed function pointer directly (no transmute here)
        ORIGINAL_MESSAGE_BOX_W_FN(hwnd, new_text, new_caption, utype)
    }
}

fn main() {
    unsafe {
        let module = GetModuleHandleW(w!("user32.dll"));
        let message_box_addr = GetProcAddress(module, s!("MessageBoxW")).unwrap();

        let res = MH_Initialize();
        assert_eq!(res, MH_OK);

        let mut original = std::ptr::null_mut();

        let res = MH_CreateHook(
            message_box_addr as *mut _,
            DetourMessageBoxW as *mut _,
            &mut original as *mut _,
        );
        assert_eq!(res, MH_OK);

        let res = MH_EnableHook(message_box_addr as *mut _);
        assert_eq!(res, MH_OK);

        let typed_orig = std::mem::transmute(original);
        ORIGINAL_MESSAGE_BOX_W_FN = typed_orig;

        MessageBoxW(
            std::ptr::null_mut(),
            w!("Original Message Box Text"),
            w!("Original Caption"),
            0,
        );

        let res = MH_DisableHook(message_box_addr as *mut _);
        assert_eq!(res, MH_OK);

        let res = MH_Uninitialize();
        assert_eq!(res, MH_OK);
    }
}
