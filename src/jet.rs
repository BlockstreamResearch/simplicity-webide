use simplicity::ffi::c_jets::frame_ffi::{c_readBit, c_writeBit};
use simplicity::ffi::c_jets::uword_width;
use simplicity::ffi::ffi::UWORD;
use simplicity::ffi::CFrameItem;
use simplicity::jet::Jet;
use simplicity::types::Final;
use simplicity::Value;
use simplicityhl::simplicity;

pub struct JetFailed;

/// Create new C read frame from the given `input` value and `ty`pe.
///
/// Return C read frame together with underlying buffer.
///
/// ## Safety
///
/// The returned frame must outlive its buffer or there is a dangling pointer.
unsafe fn get_input_frame(input: &Value, ty: &Final) -> (CFrameItem, Vec<UWORD>) {
    let uword_width = uword_width(ty.bit_width());
    let mut buffer = vec![0; uword_width];

    // Copy bits from value to input frame
    let buffer_end = buffer.as_mut_ptr().add(uword_width);
    let mut write_frame = CFrameItem::new_write(ty.bit_width(), buffer_end);
    for bit in input.iter_padded() {
        c_writeBit(&mut write_frame, bit);
    }

    // Convert input frame into read frame
    let buffer_ptr = buffer.as_mut_ptr();
    let read_frame = CFrameItem::new_read(ty.bit_width(), buffer_ptr);

    (read_frame, buffer)
}

/// Create C write frame that is as wide as `bit_width`.
///
/// Return C write frame together with underlying buffer.
///
/// ## Safety
///
/// The returned frame must outlive its buffer or there is a dangling pointer.
unsafe fn get_output_frame(bit_width: usize) -> (CFrameItem, Vec<UWORD>) {
    let uword_width = uword_width(bit_width);
    let mut buffer = vec![0; uword_width];

    // Return output frame as write frame
    let buffer_end = buffer.as_mut_ptr().add(uword_width);
    let write_frame = CFrameItem::new_write(bit_width, buffer_end);

    (write_frame, buffer)
}

/// Write `bit_width` many bits from `buffer` into active write frame.
///
/// ## Panics
///
/// Buffer has fewer than bits than `ty` is wide (converted to UWORDs).
fn value_from_frame(ty: &Final, buffer: &mut [UWORD]) -> Value {
    assert!(uword_width(ty.bit_width()) <= buffer.len());
    let buffer_ptr = buffer.as_ptr();
    let mut read_frame = unsafe { CFrameItem::new_read(ty.bit_width(), buffer_ptr) };

    let mut it = (0..ty.bit_width()).map(|_| unsafe { c_readBit(&mut read_frame) });

    Value::from_padded_bits(&mut it, ty).expect("Jets return values that fit their output type")
}

/// Execute a jet on an input and inside an environment. Return the output.
pub fn execute_jet_with_env<J: Jet>(
    jet: &J,
    input: &Value,
    env: &J::Environment,
) -> Result<Value, JetFailed> {
    let input_type = jet.source_ty().to_final();
    let output_type = jet.target_ty().to_final();

    let (input_read_frame, _input_buffer) = unsafe { get_input_frame(input, &input_type) };
    let (mut output_write_frame, mut output_buffer) =
        unsafe { get_output_frame(output_type.bit_width()) };

    let jet_fn = jet.c_jet_ptr();
    let c_env = J::c_jet_env(env);
    let success = jet_fn(&mut output_write_frame, input_read_frame, c_env);

    if success {
        Ok(value_from_frame(&output_type, &mut output_buffer))
    } else {
        Err(JetFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[wasm_bindgen_test::wasm_bindgen_test]
    #[allow(dead_code)]
    fn wasm_sanity_checks() {
        assert!(simplicity::ffi::c_jets::sanity_checks());
    }
}
