use core::marker::PhantomData;
use frame_support::{log, traits::ProcessMessageError, weights::Weight};
use xcm::latest::prelude::*;
use xcm_executor::traits::ShouldExecute;

/// Type alias to conveniently refer to `frame_system`'s `Config::AccountId`.
pub type AccountIdOf<R> = <R as frame_system::Config>::AccountId;

// //TODO: move DenyThenTry to polkadot's xcm module.
// /// Deny executing the XCM if it matches any of the Deny filter regardless of anything else.
// /// If it passes the Deny, and matches one of the Allow cases then it is let through.
// pub struct DenyThenTry<Deny, Allow>(PhantomData<Deny>, PhantomData<Allow>)
// where
// 	Deny: ShouldExecute,
// 	Allow: ShouldExecute;
//
// impl<Deny, Allow> ShouldExecute for DenyThenTry<Deny, Allow>
// where
// 	Deny: ShouldExecute,
// 	Allow: ShouldExecute,
// {
// 	fn should_execute<RuntimeCall>(
// 		origin: &MultiLocation,
// 		message: &mut [Instruction<RuntimeCall>],
// 		max_weight: Weight,
// 		weight_credit: &mut Weight,
// 	) -> Result<(), ProcessMessageError> {
// 		Deny::should_execute(origin, message, max_weight, weight_credit)?;
// 		Allow::should_execute(origin, message, max_weight, weight_credit)
// 	}
// }
//
// // See issue <https://github.com/paritytech/polkadot/issues/5233>
// pub struct DenyReserveTransferToRelayChain;
// impl ShouldExecute for DenyReserveTransferToRelayChain {
// 	fn should_execute<RuntimeCall>(
// 		origin: &MultiLocation,
// 		message: &mut [Instruction<RuntimeCall>],
// 		_max_weight: Weight,
// 		_weight_credit: &mut Weight,
// 	) -> Result<(), ProcessMessageError> {
// 		if message.iter().any(|inst| {
// 			matches!(
// 				inst,
// 				InitiateReserveWithdraw {
// 					reserve: MultiLocation { parents: 1, interior: Here },
// 					..
// 				} | DepositReserveAsset { dest: MultiLocation { parents: 1, interior: Here }, .. } |
// 					TransferReserveAsset {
// 						dest: MultiLocation { parents: 1, interior: Here },
// 						..
// 					}
// 			)
// 		}) {
// 			return Err(ProcessMessageError::Unsupported) // Deny
// 		}
//
// 		// An unexpected reserve transfer has arrived from the Relay Chain. Generally, `IsReserve`
// 		// should not allow this, but we just log it here.
// 		if matches!(origin, MultiLocation { parents: 1, interior: Here }) &&
// 			message.iter().any(|inst| matches!(inst, ReserveAssetDeposited { .. }))
// 		{
// 			log::warn!(
// 				target: "xcm::barriers",
// 				"Unexpected ReserveAssetDeposited from the Relay Chain",
// 			);
// 		}
// 		// Permit everything else
// 		Ok(())
// 	}
// }
