<<<<<<< HEAD
use std::ffi::c_char;

#[repr(C)]
pub struct LibreOfficeKit {
    pub pClass: *const LibreOfficeKitClass,
}

#[repr(C)]
pub struct LibreOfficeKitClass {
    pub nSize: usize,
    pub destroy: Option<unsafe extern "C" fn(pThis: *mut LibreOfficeKit)>,
    pub documentLoad: Option<
        unsafe extern "C" fn(pThis: *mut LibreOfficeKit, pURL: *const c_char) -> *mut LibreOfficeKitDocument,
    >,
    pub getError: Option<unsafe extern "C" fn(pThis: *mut LibreOfficeKit) -> *mut c_char>,
    pub documentLoadWithOptions: Option<
        unsafe extern "C" fn(
            pThis: *mut LibreOfficeKit,
            pURL: *const c_char,
            pOptions: *const c_char,
        ) -> *mut LibreOfficeKitDocument,
    >,
    pub freeError: Option<unsafe extern "C" fn(pFree: *mut c_char)>,
}

#[repr(C)]
pub struct LibreOfficeKitDocument {
    pub pClass: *const LibreOfficeKitDocumentClass,
}

#[repr(C)]
pub struct LibreOfficeKitDocumentClass {
    pub nSize: usize,
    pub destroy: Option<unsafe extern "C" fn(pThis: *mut LibreOfficeKitDocument)>,
    pub saveAs: Option<
        unsafe extern "C" fn(
            pThis: *mut LibreOfficeKitDocument,
            pUrl: *const c_char,
            pFormat: *const c_char,
            pFilterOptions: *const c_char,
        ) -> i32,
    >,
    pub getDocumentType: Option<unsafe extern "C" fn(pThis: *mut LibreOfficeKitDocument) -> i32>,
    pub getParts: Option<unsafe extern "C" fn(pThis: *mut LibreOfficeKitDocument) -> i32>,
}

pub type LokInitFn = unsafe extern "C" fn(pInstallPath: *const c_char) -> *mut LibreOfficeKit;

pub unsafe fn office_destroy(office: *mut LibreOfficeKit) {
    if office.is_null() {
        return;
    }
    let class = (*office).pClass;
    if class.is_null() {
        return;
    }
    if let Some(destroy) = (*class).destroy {
        destroy(office);
    }
}

pub unsafe fn office_document_load(office: *mut LibreOfficeKit, url: *const c_char) -> *mut LibreOfficeKitDocument {
    if office.is_null() {
        return std::ptr::null_mut();
    }
    let class = (*office).pClass;
    if class.is_null() {
        return std::ptr::null_mut();
    }
    if let Some(load_with_options) = (*class).documentLoadWithOptions {
        return load_with_options(office, url, std::ptr::null());
    }
    if let Some(load) = (*class).documentLoad {
        return load(office, url);
    }
    std::ptr::null_mut()
}

pub unsafe fn document_destroy(doc: *mut LibreOfficeKitDocument) {
    if doc.is_null() {
        return;
    }
    let class = (*doc).pClass;
    if class.is_null() {
        return;
    }
    if let Some(destroy) = (*class).destroy {
        destroy(doc);
    }
}

pub unsafe fn document_get_parts(doc: *mut LibreOfficeKitDocument) -> i32 {
    if doc.is_null() {
        return 0;
    }
    let class = (*doc).pClass;
    if class.is_null() {
        return 0;
    }
    (*class).getParts.map(|f| f(doc)).unwrap_or(0)
}

=======
use std::ffi::c_char;

#[repr(C)]
pub struct LibreOfficeKit {
    pub pClass: *const LibreOfficeKitClass,
}

#[repr(C)]
pub struct LibreOfficeKitClass {
    pub nSize: usize,
    pub destroy: Option<unsafe extern "C" fn(pThis: *mut LibreOfficeKit)>,
    pub documentLoad: Option<
        unsafe extern "C" fn(pThis: *mut LibreOfficeKit, pURL: *const c_char) -> *mut LibreOfficeKitDocument,
    >,
    pub getError: Option<unsafe extern "C" fn(pThis: *mut LibreOfficeKit) -> *mut c_char>,
    pub documentLoadWithOptions: Option<
        unsafe extern "C" fn(
            pThis: *mut LibreOfficeKit,
            pURL: *const c_char,
            pOptions: *const c_char,
        ) -> *mut LibreOfficeKitDocument,
    >,
    pub freeError: Option<unsafe extern "C" fn(pFree: *mut c_char)>,
}

#[repr(C)]
pub struct LibreOfficeKitDocument {
    pub pClass: *const LibreOfficeKitDocumentClass,
}

#[repr(C)]
pub struct LibreOfficeKitDocumentClass {
    pub nSize: usize,
    pub destroy: Option<unsafe extern "C" fn(pThis: *mut LibreOfficeKitDocument)>,
    pub saveAs: Option<
        unsafe extern "C" fn(
            pThis: *mut LibreOfficeKitDocument,
            pUrl: *const c_char,
            pFormat: *const c_char,
            pFilterOptions: *const c_char,
        ) -> i32,
    >,
    pub getDocumentType: Option<unsafe extern "C" fn(pThis: *mut LibreOfficeKitDocument) -> i32>,
    pub getParts: Option<unsafe extern "C" fn(pThis: *mut LibreOfficeKitDocument) -> i32>,
}

pub type LokInitFn = unsafe extern "C" fn(pInstallPath: *const c_char) -> *mut LibreOfficeKit;

pub unsafe fn office_destroy(office: *mut LibreOfficeKit) {
    if office.is_null() {
        return;
    }
    let class = (*office).pClass;
    if class.is_null() {
        return;
    }
    if let Some(destroy) = (*class).destroy {
        destroy(office);
    }
}

pub unsafe fn office_document_load(office: *mut LibreOfficeKit, url: *const c_char) -> *mut LibreOfficeKitDocument {
    if office.is_null() {
        return std::ptr::null_mut();
    }
    let class = (*office).pClass;
    if class.is_null() {
        return std::ptr::null_mut();
    }
    if let Some(load_with_options) = (*class).documentLoadWithOptions {
        return load_with_options(office, url, std::ptr::null());
    }
    if let Some(load) = (*class).documentLoad {
        return load(office, url);
    }
    std::ptr::null_mut()
}

pub unsafe fn document_destroy(doc: *mut LibreOfficeKitDocument) {
    if doc.is_null() {
        return;
    }
    let class = (*doc).pClass;
    if class.is_null() {
        return;
    }
    if let Some(destroy) = (*class).destroy {
        destroy(doc);
    }
}

pub unsafe fn document_get_parts(doc: *mut LibreOfficeKitDocument) -> i32 {
    if doc.is_null() {
        return 0;
    }
    let class = (*doc).pClass;
    if class.is_null() {
        return 0;
    }
    (*class).getParts.map(|f| f(doc)).unwrap_or(0)
}

>>>>>>> 0dc035f57a1c694c8225272cdbd0bfc9c9d60bb9
