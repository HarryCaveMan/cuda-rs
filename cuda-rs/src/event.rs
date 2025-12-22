use crate::{
    error::CuResult,
    ffi,
    stream::CuStream,
};
use std::sync::Arc;

struct CUevent(ffi::CUevent);

impl Drop for CUevent {
    fn drop(&mut self) {
        unsafe { ffi::cuEventDestroy_v2(self.0) };
    }
}

enum Inner {
    Owned(Arc<CUevent>),
    Borrowed(ffi::CUevent),
}

pub struct CuEvent(Inner);

impl CuEvent {
    pub fn new() -> CuResult<Self> {
        let mut e = std::ptr::null_mut();
        let res = unsafe {
            ffi::cuEventCreate(&mut e, ffi::CUevent_flags_enum_CU_EVENT_DEFAULT)
        };
        let e = CuEvent(Inner::Owned(Arc::new(CUevent(e))));

        wrap!(e, res)
    }

    pub unsafe fn from_raw(e: ffi::CUevent) -> Self {
        CuEvent(Inner::Borrowed(e))
    }

    pub fn record(&self, stream: &CuStream) -> CuResult<()> {
        let res = unsafe {
            ffi::cuEventRecord(self.get_raw(), stream.get_raw())
        };

        wrap!((), res)
    }

    pub fn query(&self) -> CuResult<bool> {
        let res = unsafe { ffi::cuEventQuery(self.get_raw()) };

        if res == ffi::cudaError_enum_CUDA_SUCCESS || res == ffi::cudaError_enum_CUDA_ERROR_NOT_READY {
            Ok(res == ffi::cudaError_enum_CUDA_SUCCESS)
        } else {
            wrap!(false, res)
        }
    }

    pub fn synchronize(&self) -> CuResult<()> {
        let res = unsafe { ffi::cuEventSynchronize(self.get_raw()) };

        wrap!((), res)
    }

    pub fn elapsed_time(&self, start: &CuEvent) -> CuResult<f32> {
        let mut ms = 0.0;
        let res = unsafe {
            let start_e = start.get_raw();
            let end_e = self.get_raw();

            ffi::cuEventElapsedTime_v2(
                &mut ms as *mut f32, start_e, end_e
            )
        };

        wrap!(ms, res)
    }

    pub unsafe fn get_raw(&self) -> ffi::CUevent {
        match self.0 {
            Inner::Owned(ref e) => e.0,
            Inner::Borrowed(e) => e,
        }
    }
}

impl Clone for CuEvent {
    fn clone(&self) -> Self {
        match self.0 {
            Inner::Owned(ref e) => CuEvent(Inner::Owned(e.clone())),
            Inner::Borrowed(e) => CuEvent(Inner::Borrowed(e)),
        }
    }
}
