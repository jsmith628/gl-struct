use super::*;
use std::mem::MaybeUninit;

gl_resource! {
    pub struct Query {
        gl = GL15,
        target = !,
        ident = Query,
        gen = GenQueries,
        is = IsQuery,
        delete = DeleteQueries
    }
}

pub trait QueryTarget<T> {
    type GL: GLVersion;
}

impl Query {

    pub fn result_available(&self) -> bool {
        unsafe {
            let mut dest = MaybeUninit::uninit();
            gl::GetQueryObjectiv(self.id(), gl::QUERY_RESULT_AVAILABLE, dest.as_mut_ptr());
            dest.assume_init() != 0
        }        
    }

}
