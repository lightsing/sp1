#![allow(unused_imports)]
use crate::io;
use crate::syscall_uint256_mul;
use crate::unconstrained;
use num::{BigUint, Integer};

/// Performs division on 256-bit unsigned integers represented as little endian byte arrays.
///
/// This function divides `x` by `y`, both of which are 256-bit unsigned integers
/// represented as arrays of bytes in little-endian order. It returns the quotient
/// of the division as a 256-bit unsigned integer in the same byte array format.
pub fn uint256_div(x: &mut [u8; 32], y: &[u8; 32]) -> [u8; 32] {
    // Assert that the divisor is not zero.
    assert!(y != &[0; 32], "division by zero");
    cfg_if::cfg_if! {
        if #[cfg(all(target_os = "zkvm", target_vendor = "succinct"))] {
            let dividend = BigUint::from_bytes_le(x);

            unconstrained!{
                let divisor = BigUint::from_bytes_le(y);
                let (quotient, remainder) = dividend.div_rem(&divisor);

                let mut quotient_bytes = quotient.to_bytes_le();
                quotient_bytes.resize(32, 0u8);
                io::hint_slice(&quotient_bytes);

                let mut remainder_bytes = remainder.to_bytes_le();
                remainder_bytes.resize(32, 0u8);
                io::hint_slice(&remainder_bytes);
            };

            let mut quotient_bytes: [u8; 32] = io::read_vec().try_into().unwrap();

            let mut remainder_bytes: [u8; 32] = io::read_vec().try_into().unwrap();

            let remainder = BigUint::from_bytes_le(&remainder_bytes);

            *x = quotient_bytes;

            unsafe {
                syscall_uint256_mul(quotient_bytes.as_mut_ptr() as *mut u32, y.as_ptr() as *const u32);
            }

            let quotient_times_divisor = BigUint::from_bytes_le(&quotient_bytes);
            assert_eq!(quotient_times_divisor, dividend - remainder);

            *x
        } else {
            let result_biguint = BigUint::from_bytes_le(x) / BigUint::from_bytes_le(y);
            let mut result_biguint_bytes = result_biguint.to_bytes_le();
            result_biguint_bytes.resize(32, 0u8);
            result_biguint_bytes.try_into().unwrap_or([0; 32])
        }
    }
}
