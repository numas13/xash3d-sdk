use core::ffi::{c_char, c_int, c_uint, c_void, CStr};

use shared::{
    consts::{MAX_MODS, MAX_QPATH, NUM_AMBIENTS},
    raw::{byte, poolhandle_t, qboolean},
};

// HACK:
pub type va_list = c_void;

pub type fs_offset_t = libc::off_t;
pub type dword = c_uint;
pub type string = [c_char; 256];

pub const FS_API_VERSION: u32 = 3;
pub const FS_API_CREATEINTERFACE_TAG: &CStr = c"XashFileSystem002";
pub const FILESYSTEM_INTERFACE_VERSION: &CStr = c"VFileSystem009";
pub const GET_FS_API: &CStr = c"GetFSAPI";

pub const FS_STATIC_PATH: _bindgen_ty_8 = 1;
pub const FS_NOWRITE_PATH: _bindgen_ty_8 = 2;
pub const FS_GAMEDIR_PATH: _bindgen_ty_8 = 4;
pub const FS_CUSTOM_PATH: _bindgen_ty_8 = 8;
pub const FS_GAMERODIR_PATH: _bindgen_ty_8 = 16;
pub const FS_SKIP_ARCHIVED_WADS: _bindgen_ty_8 = 32;
pub const FS_LOAD_PACKED_WAD: _bindgen_ty_8 = 64;
pub const FS_MOUNT_HD: _bindgen_ty_8 = 128;
pub const FS_MOUNT_LV: _bindgen_ty_8 = 256;
pub const FS_MOUNT_ADDON: _bindgen_ty_8 = 512;
pub const FS_MOUNT_L10N: _bindgen_ty_8 = 1024;
pub const FS_GAMEDIRONLY_SEARCH_FLAGS: _bindgen_ty_8 = 28;
pub type _bindgen_ty_8 = c_uint;

#[repr(C)]
pub struct searchpath_s {
    _unused: [u8; 0],
}
pub type searchpath_t = searchpath_s;

pub const IAES_ONLY_REAL_ARCHIVES: _bindgen_ty_9 = 1;
pub type _bindgen_ty_9 = c_uint;

#[repr(C)]
pub struct search_t {
    pub numfilenames: c_int,
    pub filenames: *mut *mut c_char,
    pub filenamesbuffer: *mut c_char,
}

#[repr(C)]
pub struct gameinfo_s {
    pub gamefolder: [c_char; MAX_QPATH],
    pub basedir: [c_char; MAX_QPATH],
    pub falldir: [c_char; MAX_QPATH],
    pub startmap: [c_char; MAX_QPATH],
    pub trainmap: [c_char; MAX_QPATH],
    pub title: [c_char; 64],
    pub version: f32,
    pub dll_path: [c_char; MAX_QPATH],
    pub game_dll: [c_char; MAX_QPATH],
    pub iconpath: [c_char; MAX_QPATH],
    pub game_url: string,
    pub update_url: string,
    pub type_: [c_char; MAX_QPATH],
    pub date: [c_char; MAX_QPATH],
    pub size: usize,
    pub gamemode: c_int,
    pub secure: qboolean,
    pub nomodels: qboolean,
    pub noskills: qboolean,
    pub render_picbutton_text: qboolean,
    pub internal_vgui_support: qboolean,
    pub sp_entity: [c_char; 32],
    pub mp_entity: [c_char; 32],
    pub mp_filter: [c_char; 32],
    pub ambientsound: [[c_char; MAX_QPATH]; NUM_AMBIENTS],
    pub max_edicts: c_int,
    pub max_tents: c_int,
    pub max_beams: c_int,
    pub max_particles: c_int,
    pub game_dll_linux: [c_char; 64],
    pub game_dll_osx: [c_char; 64],
    pub added: qboolean,
    pub quicksave_aged_count: c_int,
    pub autosave_aged_count: c_int,
    pub hd_background: qboolean,
    pub animated_title: qboolean,
    pub demomap: [c_char; MAX_QPATH],
    pub rodir: qboolean,
    pub mtime: i64,
}

#[repr(C)]
pub struct fs_dllinfo_t {
    pub fullPath: [c_char; 2048],
    pub shortPath: string,
    pub encrypted: qboolean,
    pub custom_loader: qboolean,
}

#[repr(C)]
pub struct fs_globals_t {
    pub GameInfo: *mut gameinfo_s,
    pub games: [*mut gameinfo_s; MAX_MODS],
    pub numgames: c_int,
}

#[repr(C)]
pub struct file_s {
    _unused: [u8; 0],
}
pub type file_t = file_s;

#[repr(C)]
pub struct fs_api_t {
    pub InitStdio: Option<
        unsafe extern "C" fn(
            unused_set_to_true: qboolean,
            rootdir: *const c_char,
            basedir: *const c_char,
            gamedir: *const c_char,
            rodir: *const c_char,
        ) -> qboolean,
    >,
    pub ShutdownStdio: Option<unsafe extern "C" fn()>,
    pub Rescan: Option<unsafe extern "C" fn(flags: u32, language: *const c_char)>,
    pub ClearSearchPath: Option<unsafe extern "C" fn()>,
    pub AllowDirectPaths: Option<unsafe extern "C" fn(enable: qboolean)>,
    pub AddGameDirectory: Option<unsafe extern "C" fn(dir: *const c_char, flags: c_uint)>,
    pub AddGameHierarchy: Option<unsafe extern "C" fn(dir: *const c_char, flags: c_uint)>,
    pub Search: Option<
        unsafe extern "C" fn(
            pattern: *const c_char,
            caseinsensitive: c_int,
            gamedironly: c_int,
        ) -> *mut search_t,
    >,
    pub SetCurrentDirectory: Option<unsafe extern "C" fn(path: *const c_char) -> c_int>,
    pub FindLibrary: Option<
        unsafe extern "C" fn(
            dllname: *const c_char,
            directpath: qboolean,
            dllinfo: *mut fs_dllinfo_t,
        ) -> qboolean,
    >,
    pub Path_f: Option<unsafe extern "C" fn()>,
    pub LoadGameInfo: Option<unsafe extern "C" fn(rootfolder: *const c_char)>,
    pub Open: Option<
        unsafe extern "C" fn(
            filepath: *const c_char,
            mode: *const c_char,
            gamedironly: qboolean,
        ) -> *mut file_t,
    >,
    pub Write: Option<
        unsafe extern "C" fn(
            file: *mut file_t,
            data: *const c_void,
            datasize: usize,
        ) -> fs_offset_t,
    >,
    pub Read: Option<
        unsafe extern "C" fn(
            file: *mut file_t,
            buffer: *mut c_void,
            buffersize: usize,
        ) -> fs_offset_t,
    >,
    pub Seek: Option<
        unsafe extern "C" fn(file: *mut file_t, offset: fs_offset_t, whence: c_int) -> c_int,
    >,
    pub Tell: Option<unsafe extern "C" fn(file: *mut file_t) -> fs_offset_t>,
    pub Eof: Option<unsafe extern "C" fn(file: *mut file_t) -> qboolean>,
    pub Flush: Option<unsafe extern "C" fn(file: *mut file_t) -> c_int>,
    pub Close: Option<unsafe extern "C" fn(file: *mut file_t) -> c_int>,
    pub Gets: Option<
        unsafe extern "C" fn(file: *mut file_t, string: *mut c_char, bufsize: usize) -> c_int,
    >,
    pub UnGetc: Option<unsafe extern "C" fn(file: *mut file_t, c: c_char) -> c_int>,
    pub Getc: Option<unsafe extern "C" fn(file: *mut file_t) -> c_int>,
    pub VPrintf: Option<
        unsafe extern "C" fn(file: *mut file_t, format: *const c_char, ap: *mut va_list) -> c_int,
    >,
    pub Printf:
        Option<unsafe extern "C" fn(file: *mut file_t, format: *const c_char, ...) -> c_int>,
    pub Print: Option<unsafe extern "C" fn(file: *mut file_t, msg: *const c_char) -> c_int>,
    pub FileLength: Option<unsafe extern "C" fn(f: *mut file_t) -> fs_offset_t>,
    pub FileCopy: Option<
        unsafe extern "C" fn(
            pOutput: *mut file_t,
            pInput: *mut file_t,
            fileSize: c_int,
        ) -> qboolean,
    >,
    pub LoadFile: Option<
        unsafe extern "C" fn(
            path: *const c_char,
            filesizeptr: *mut fs_offset_t,
            gamedironly: qboolean,
        ) -> *mut byte,
    >,
    pub LoadDirectFile: Option<
        unsafe extern "C" fn(path: *const c_char, filesizeptr: *mut fs_offset_t) -> *mut byte,
    >,
    pub WriteFile: Option<
        unsafe extern "C" fn(
            filename: *const c_char,
            data: *const c_void,
            len: fs_offset_t,
        ) -> qboolean,
    >,
    pub CRC32_File:
        Option<unsafe extern "C" fn(crcvalue: *mut dword, filename: *const c_char) -> qboolean>,
    pub MD5_HashFile: Option<
        unsafe extern "C" fn(
            digest: *mut [byte; 16],
            pszFileName: *const c_char,
            seed: *mut [c_uint; 4],
        ) -> qboolean,
    >,
    pub FileExists:
        Option<unsafe extern "C" fn(filename: *const c_char, gamedironly: c_int) -> c_int>,
    pub FileTime:
        Option<unsafe extern "C" fn(filename: *const c_char, gamedironly: qboolean) -> c_int>,
    pub FileSize:
        Option<unsafe extern "C" fn(filename: *const c_char, gamedironly: qboolean) -> fs_offset_t>,
    pub Rename:
        Option<unsafe extern "C" fn(oldname: *const c_char, newname: *const c_char) -> qboolean>,
    pub Delete: Option<unsafe extern "C" fn(path: *const c_char) -> qboolean>,
    pub SysFileExists: Option<unsafe extern "C" fn(path: *const c_char) -> qboolean>,
    pub GetDiskPath:
        Option<unsafe extern "C" fn(name: *const c_char, gamedironly: qboolean) -> *const c_char>,
    pub ArchivePath: Option<unsafe extern "C" fn(f: *mut file_t) -> *const c_char>,
    pub MountArchive_Fullpath:
        Option<unsafe extern "C" fn(path: *const c_char, flags: c_int) -> *mut c_void>,
    pub GetFullDiskPath: Option<
        unsafe extern "C" fn(
            buffer: *mut c_char,
            size: usize,
            name: *const c_char,
            gamedironly: qboolean,
        ) -> qboolean,
    >,
    pub LoadFileMalloc: Option<
        unsafe extern "C" fn(
            path: *const c_char,
            filesizeptr: *mut fs_offset_t,
            gamedironly: qboolean,
        ) -> *mut byte,
    >,
    pub IsArchiveExtensionSupported:
        Option<unsafe extern "C" fn(ext: *const c_char, flags: c_uint) -> qboolean>,
    pub GetArchiveByName: Option<
        unsafe extern "C" fn(name: *const c_char, prev: *mut searchpath_t) -> *mut searchpath_t,
    >,
    pub FindFileInArchive: Option<
        unsafe extern "C" fn(
            sp: *mut searchpath_t,
            path: *const c_char,
            outpath: *mut c_char,
            len: usize,
        ) -> c_int,
    >,
    pub OpenFileFromArchive: Option<
        unsafe extern "C" fn(
            arg1: *mut searchpath_t,
            path: *const c_char,
            mode: *const c_char,
            pack_ind: c_int,
        ) -> *mut file_t,
    >,
    pub LoadFileFromArchive: Option<
        unsafe extern "C" fn(
            sp: *mut searchpath_t,
            path: *const c_char,
            pack_ind: c_int,
            filesizeptr: *mut fs_offset_t,
            sys_malloc: qboolean,
        ) -> *mut byte,
    >,
    pub GetRootDirectory: Option<unsafe extern "C" fn(path: *mut c_char, size: usize) -> qboolean>,
    pub MakeGameInfo: Option<unsafe extern "C" fn()>,
}

#[repr(C)]
pub struct fs_interface_t {
    pub _Con_Printf: Option<unsafe extern "C" fn(fmt: *const c_char, ...)>,
    pub _Con_DPrintf: Option<unsafe extern "C" fn(fmt: *const c_char, ...)>,
    pub _Con_Reportf: Option<unsafe extern "C" fn(fmt: *const c_char, ...)>,
    pub _Sys_Error: Option<unsafe extern "C" fn(fmt: *const c_char, ...)>,
    pub _Mem_AllocPool: Option<
        unsafe extern "C" fn(
            name: *const c_char,
            filename: *const c_char,
            fileline: c_int,
        ) -> poolhandle_t,
    >,
    pub _Mem_FreePool: Option<
        unsafe extern "C" fn(poolptr: *mut poolhandle_t, filename: *const c_char, fileline: c_int),
    >,
    pub _Mem_Alloc: Option<
        unsafe extern "C" fn(
            poolptr: poolhandle_t,
            size: usize,
            clear: qboolean,
            filename: *const c_char,
            fileline: c_int,
        ) -> *mut c_void,
    >,
    pub _Mem_Realloc: Option<
        unsafe extern "C" fn(
            poolptr: poolhandle_t,
            memptr: *mut c_void,
            size: usize,
            clear: qboolean,
            filename: *const c_char,
            fileline: c_int,
        ) -> *mut c_void,
    >,
    pub _Mem_Free:
        Option<unsafe extern "C" fn(data: *mut c_void, filename: *const c_char, fileline: c_int)>,
    pub _Sys_GetNativeObject: Option<unsafe extern "C" fn(object: *const c_char) -> *mut c_void>,
}

pub type FSAPI = Option<
    unsafe extern "C" fn(
        version: c_int,
        api: *mut fs_api_t,
        globals: *mut *mut fs_globals_t,
        interface: *const fs_interface_t,
    ) -> c_int,
>;
