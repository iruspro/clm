//! State of the Transactions screen.

/// Placeholder for the Transactions screen's state.
///
/// The screen exists in the tab bar but has no behaviour yet; this type reserves
/// its slot in [`State`](crate::app::State) so wiring it up later doesn't ripple
/// through the accessors.
#[derive(Debug, Clone, Default)]
pub struct TransactionsState {}
