#![allow(non_snake_case)]

use std::ffi::c_void;

use minhook_detours::*;
use windows_sys::{
	Win32::{
		Foundation::{HANDLE, NTSTATUS},
		System::{
			IO::{IO_STATUS_BLOCK, PIO_APC_ROUTINE},
			LibraryLoader::{GetModuleHandleW, GetProcAddress},
		},
	},
	s, w,
};

type NtReadFileFunc = extern "system" fn(
	filehandle: HANDLE,
	event: HANDLE,
	apcroutine: PIO_APC_ROUTINE,
	apccontext: *const c_void,
	iostatusblock: *mut IO_STATUS_BLOCK,
	buffer: *mut c_void,
	length: u32,
	byteoffset: *const i64,
	key: *const u32,
) -> NTSTATUS;

static mut ORIGINAL_NT_READ_FILE_FN: NtReadFileFunc = DetourNtReadFile;

extern "system" fn DetourNtReadFile(
	filehandle: HANDLE,
	event: HANDLE,
	apcroutine: PIO_APC_ROUTINE,
	apccontext: *const c_void,
	iostatusblock: *mut IO_STATUS_BLOCK,
	buffer: *mut c_void,
	length: u32,
	byteoffset: *const i64,
	key: *const u32,
) -> NTSTATUS {
	println!("NtReadFile intercepted!");

	unsafe {
		// call the typed function pointer directly (no transmute here)
		ORIGINAL_NT_READ_FILE_FN(
			filehandle,
			event,
			apcroutine,
			apccontext,
			iostatusblock,
			buffer,
			length,
			byteoffset,
			key,
		)
	}
}

fn main() {
	unsafe {
		let module = GetModuleHandleW(w!("ntdll.dll"));
		let nt_read_file_addr = GetProcAddress(module, s!("NtReadFile")).unwrap();

		let res = MH_Initialize();
		assert_eq!(res, MH_OK);

		let mut original = std::ptr::null_mut();

		let res = MH_CreateHook(
			nt_read_file_addr as *mut _,
			DetourNtReadFile as *mut _,
			&mut original as *mut _,
		);
		assert_eq!(res, MH_OK);

		let res = MH_EnableHook(nt_read_file_addr as *mut _);
		assert_eq!(res, MH_OK);

		let typed_orig = std::mem::transmute(original);
		ORIGINAL_NT_READ_FILE_FN = typed_orig;

		std::fs::read("C:\\Windows\\System32\\notepad.exe").unwrap();

		let res = MH_DisableHook(nt_read_file_addr as *mut _);
		assert_eq!(res, MH_OK);

		let res = MH_Uninitialize();
		assert_eq!(res, MH_OK);
	}
}
