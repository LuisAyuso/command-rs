use std::mem;

pub struct CheatBuffer{
}



unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::std::mem::size_of::<T>(),
    )
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[cfg(test)]
mod tests {
    use super::*;

    fn caca<F>(f:F){

		let mut v = Vec::<u8>::with_capacity(mem::size_of_val(&f));
		
		let ptr = unsafe{ any_as_u8_slice(&f) };
		v.extend_from_slice(ptr);

//		unsafe{
//			let in_ptr = any_as_u8_slice(&f).as_ptr();
//			let target_ptr = v.as_mut_ptr();
//
//			// here comes the offset
//			
//			// copy buffers
//			::std::ptr::copy(in_ptr, target_ptr, mem::size_of_val(&f));
//		}


		println!("hello: {:?}", v);

    }

    #[test]
    fn cheat_buffer() {

{
        let lambda = |x:i32| x +1;
        assert_eq!(0, mem::size_of_val(&lambda));
        caca(lambda);
}
{
		let mut x = 1;
        let lambda = |y:i32| x += y;
        caca(lambda);
}
{
		let mut x = 1;
		let y = 2;
        let lambda = |z:i32| x += y + z;
        caca(lambda);
}

    }
}
