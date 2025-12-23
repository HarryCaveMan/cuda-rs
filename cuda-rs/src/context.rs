use crate::{
    device::CuDevice,
    error::CuResult,
    ffi,
};
use std::sync::Arc;

struct CUcontext(ffi::CUcontext);

impl Drop for CUcontext {
    fn drop(&mut self) {
        unsafe { ffi::cuCtxDestroy_v2(self.0) };
    }
}

enum Inner {
    Owned(Arc<CUcontext>),
    Borrowed(ffi::CUcontext),
}

pub struct CuContext(Inner);

impl CuContext {
    pub fn new(device: &CuDevice) -> CuResult<Self> {
        let mut ctx = std::ptr::null_mut();
        let mut affinity_param = ffi::CUexecAffinityParam {
            type_: ffi::CUexecAffinityType_enum_CU_EXEC_AFFINITY_TYPE_SM_COUNT,
            param: ffi::CUexecAffinityParam_st__bindgen_ty_1 {
                smCount: ffi::CUexecAffinitySmCount {
                    val: 0, // 0 means use all available SMs
                },
            },
        };
    
        let mut params = ffi::CUctxCreateParams {
            execAffinityParams: &mut affinity_param as *mut ffi::CUexecAffinityParam,
            numExecAffinityParams: 1,
            cigParams: std::ptr::null_mut(),
        };
        let res = unsafe {
            ffi::cuCtxCreate_v4(&mut ctx, &mut params, 0, device.get_raw())
        };
        let ctx = CuContext(Inner::Owned(Arc::new(CUcontext(ctx))));

        wrap!(ctx, res)
    }

    pub unsafe fn from_raw(ctx: ffi::CUcontext) -> Self {
        CuContext(Inner::Borrowed(ctx))
    }

    pub fn retain_primary_context(device: &CuDevice) -> CuResult<Self> {
        let mut ctx = std::ptr::null_mut();
        let res = unsafe {
            ffi::cuDevicePrimaryCtxRetain(&mut ctx, device.get_raw())
        };
        let ctx = CuContext(Inner::Borrowed(ctx));

        wrap!(ctx, res)
    }

    pub fn current() -> CuResult<Self> {
        let mut ctx = std::ptr::null_mut();
        let res = unsafe { ffi::cuCtxGetCurrent(&mut ctx) };
        let ctx = CuContext(Inner::Borrowed(ctx));

        wrap!(ctx, res)
    }

    pub unsafe fn get_raw(&self) -> ffi::CUcontext {
        match self.0 {
            Inner::Owned(ref ctx) => ctx.0,
            Inner::Borrowed(ctx) => ctx,
        }
    }

    pub fn push(&self) -> CuResult<()> {
        let res = unsafe {
            let ctx = self.get_raw();
            ffi::cuCtxPushCurrent_v2(ctx)
        };

        wrap!((), res)
    }

    pub fn pop() -> CuResult<()> {
        let res = unsafe {
            ffi::cuCtxPopCurrent_v2(&mut std::ptr::null_mut())
        };

        wrap!((), res)
    }

    pub fn synchronize() -> CuResult<()> {
        let res = unsafe { ffi::cuCtxSynchronize() };

        wrap!((), res)
    }

    pub fn guard(self) -> CuResult<CuContextGuard> {
        CuContextGuard::new(self)
    }
}

impl Clone for CuContext {
    fn clone(&self) -> Self {
        match self.0 {
            Inner::Owned(ref ctx) => CuContext(Inner::Owned(ctx.clone())),
            Inner::Borrowed(ctx) => CuContext(Inner::Borrowed(ctx)),
        }
    }
}

pub struct CuContextGuard(pub CuContext);

impl CuContextGuard {
    pub fn new(ctx: CuContext) -> CuResult<Self> {
        ctx.push()?;
        Ok(CuContextGuard(ctx))
    }
}

impl Drop for CuContextGuard {
    fn drop(&mut self) {
        CuContext::pop().unwrap();
    }
}
