//! Parallel batch decoding.

use rayon::prelude::*;

use soldecoders_types::{DecodeError, DecodedEvent, RawInstruction};

use crate::registry::DecoderRegistry;

/// Decode a large batch of instructions in parallel.
///
/// Decoding is pure and CPU-bound, so the work fans out across the rayon thread
/// pool. Output order matches input order. For small batches the serial path in
/// [`DecoderRegistry::decode_instruction`] is preferable (no fork/join cost);
/// callers should reserve this for high-throughput ingestion.
///
/// Time complexity: `O(n)` over instructions, divided across the pool; space
/// `O(n)` for the result vector.
pub fn decode_batch(
    registry: &DecoderRegistry,
    instructions: &[RawInstruction],
) -> Vec<Result<DecodedEvent, DecodeError>> {
    instructions
        .par_iter()
        .map(|ix| registry.decode_instruction(ix))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InstructionDecoder;
    use soldecoders_types::Pubkey;

    #[test]
    fn batch_matches_serial_order() {
        let reg = DecoderRegistry::builtin();
        let system = crate::SystemDecoder::new().program_id().clone();
        let make = |lamports: u64| {
            let mut data = vec![2, 0, 0, 0];
            data.extend_from_slice(&lamports.to_le_bytes());
            RawInstruction::new(
                system.clone(),
                vec![Pubkey::new([1; 32]), Pubkey::new([2; 32])],
                data,
            )
        };
        let items: Vec<_> = (0..1000).map(make).collect();
        let parallel = decode_batch(&reg, &items);
        assert_eq!(parallel.len(), 1000);
        for (i, r) in parallel.iter().enumerate() {
            match r {
                Ok(DecodedEvent::SystemTransfer { lamports, .. }) => {
                    assert_eq!(*lamports, i as u64);
                }
                other => panic!("unexpected {other:?}"),
            }
        }
    }
}
