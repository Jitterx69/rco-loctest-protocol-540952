//! # Two-Phase Commit (2PC) State Machine
//!
//! Implements the PREPARE → VOTE → COMMIT finite state machine that
//! ensures atomicity of batch + anchor persistence.
//!
//! ## State Transitions
//!
//! ```text
//! IDLE → PREPARE → VOTE_YES → COMMITTED
//!                → VOTE_NO  → ABORTED
//! ```
//!
//! ## Invariant
//!
//! At no point can a batch be committed without its anchor, or vice versa.
//! The 2PC ensures both are written atomically to the WAL.

use rco_types::error::RcoError;
use rco_types::HashDigest;

/// The state of a 2PC transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TpcState {
    /// No transaction in progress.
    Idle,
    /// Transaction has been prepared (data validated, serialized).
    Prepared,
    /// WAL write succeeded — voter says YES.
    VoteYes,
    /// WAL write failed — voter says NO.
    VoteNo,
    /// Transaction committed to durable storage.
    Committed,
    /// Transaction aborted (rolled back).
    Aborted,
}

impl TpcState {
    /// Returns `true` if this is a terminal state.
    #[must_use]
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Committed | Self::Aborted)
    }

    /// Returns `true` if we can transition to the given target state.
    #[must_use]
    pub fn can_transition_to(self, target: TpcState) -> bool {
        matches!(
            (self, target),
            (Self::Idle, Self::Prepared)
                | (Self::Prepared, Self::VoteYes)
                | (Self::Prepared, Self::VoteNo)
                | (Self::VoteYes, Self::Committed)
                | (Self::VoteNo, Self::Aborted)
        )
    }
}

/// A prepared transaction holding the data needed for commit.
#[derive(Debug)]
pub struct PreparedTransaction {
    /// Batch index.
    pub batch_index: u64,
    /// Serialized batch (Bencoded).
    pub encoded_batch: Vec<u8>,
    /// Computed RML anchor.
    pub anchor: HashDigest,
    /// WAL entry offset (set after VOTE_YES).
    pub wal_offset: Option<u64>,
}

/// Two-Phase Commit controller.
///
/// Manages the FSM state and enforces transition rules.
pub struct TwoPhaseCommit {
    /// Current FSM state.
    state: TpcState,
    /// The currently prepared transaction (if any).
    prepared: Option<PreparedTransaction>,
    /// Total number of committed transactions.
    committed_count: u64,
    /// Total number of aborted transactions.
    aborted_count: u64,
}

impl TwoPhaseCommit {
    /// Creates a new 2PC controller in the IDLE state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: TpcState::Idle,
            prepared: None,
            committed_count: 0,
            aborted_count: 0,
        }
    }

    /// Returns the current FSM state.
    #[must_use]
    pub fn state(&self) -> TpcState {
        self.state
    }

    /// Returns the number of committed transactions.
    #[must_use]
    pub fn committed_count(&self) -> u64 {
        self.committed_count
    }

    /// Returns the number of aborted transactions.
    #[must_use]
    pub fn aborted_count(&self) -> u64 {
        self.aborted_count
    }

    /// Transitions from IDLE → PREPARED.
    ///
    /// Validates the batch and computes the RML anchor.
    ///
    /// # Errors
    ///
    /// Returns `RcoError::TwoPcAbort` if not in `Idle` state.
    pub fn prepare(
        &mut self,
        batch_index: u64,
        encoded_batch: Vec<u8>,
        anchor: HashDigest,
    ) -> Result<(), RcoError> {
        self.transition(TpcState::Prepared)?;

        self.prepared = Some(PreparedTransaction {
            batch_index,
            encoded_batch,
            anchor,
            wal_offset: None,
        });

        Ok(())
    }

    /// Transitions from PREPARED → VOTE_YES after successful WAL write.
    ///
    /// # Arguments
    ///
    /// * `wal_offset` — The file offset returned by `WalEngine::prepare()`.
    ///
    /// # Errors
    ///
    /// Returns `RcoError::TwoPcAbort` if not in `Prepared` state.
    pub fn vote_yes(&mut self, wal_offset: u64) -> Result<(), RcoError> {
        self.transition(TpcState::VoteYes)?;

        if let Some(ref mut txn) = self.prepared {
            txn.wal_offset = Some(wal_offset);
        }

        Ok(())
    }

    /// Transitions from PREPARED → VOTE_NO after failed WAL write.
    ///
    /// # Errors
    ///
    /// Returns `RcoError::TwoPcAbort` if not in `Prepared` state.
    pub fn vote_no(&mut self) -> Result<(), RcoError> {
        self.transition(TpcState::VoteNo)?;
        Ok(())
    }

    /// Transitions from VOTE_YES → COMMITTED (finalize).
    ///
    /// Consumes the prepared transaction and returns it for caller use.
    ///
    /// # Errors
    ///
    /// Returns `RcoError::TwoPcAbort` if not in `VoteYes` state.
    pub fn commit(&mut self) -> Result<PreparedTransaction, RcoError> {
        self.transition(TpcState::Committed)?;
        self.committed_count += 1;

        let txn = self.prepared.take().ok_or(RcoError::TwoPcAbort)?;

        // Reset to idle for the next transaction
        self.state = TpcState::Idle;

        Ok(txn)
    }

    /// Transitions from VOTE_NO → ABORTED (rollback).
    ///
    /// # Errors
    ///
    /// Returns `RcoError::TwoPcAbort` if not in `VoteNo` state.
    pub fn abort(&mut self) -> Result<(), RcoError> {
        self.transition(TpcState::Aborted)?;
        self.aborted_count += 1;
        self.prepared = None;

        // Reset to idle for the next transaction
        self.state = TpcState::Idle;

        Ok(())
    }

    /// Returns a reference to the currently prepared transaction.
    #[must_use]
    pub fn prepared_transaction(&self) -> Option<&PreparedTransaction> {
        self.prepared.as_ref()
    }

    /// Attempts a state transition, returning an error if invalid.
    fn transition(&mut self, target: TpcState) -> Result<(), RcoError> {
        if self.state.can_transition_to(target) {
            self.state = target;
            Ok(())
        } else {
            Err(RcoError::TwoPcAbort)
        }
    }
}

impl Default for TwoPhaseCommit {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rco_types::HASH_SIZE;

    fn dummy_anchor() -> HashDigest {
        [0xAB; HASH_SIZE]
    }

    // ── Happy Path ──────────────────────────────────────────────────

    #[test]
    fn test_full_commit_cycle() {
        let mut tpc = TwoPhaseCommit::new();
        assert_eq!(tpc.state(), TpcState::Idle);

        // PREPARE
        tpc.prepare(1, b"batch".to_vec(), dummy_anchor()).unwrap();
        assert_eq!(tpc.state(), TpcState::Prepared);

        // VOTE YES
        tpc.vote_yes(1024).unwrap();
        assert_eq!(tpc.state(), TpcState::VoteYes);

        // COMMIT
        let txn = tpc.commit().unwrap();
        assert_eq!(txn.batch_index, 1);
        assert_eq!(txn.wal_offset, Some(1024));

        // Should be back to idle
        assert_eq!(tpc.state(), TpcState::Idle);
        assert_eq!(tpc.committed_count(), 1);
    }

    #[test]
    fn test_full_abort_cycle() {
        let mut tpc = TwoPhaseCommit::new();

        tpc.prepare(1, b"batch".to_vec(), dummy_anchor()).unwrap();
        tpc.vote_no().unwrap();
        assert_eq!(tpc.state(), TpcState::VoteNo);

        tpc.abort().unwrap();
        assert_eq!(tpc.state(), TpcState::Idle);
        assert_eq!(tpc.aborted_count(), 1);
    }

    // ── Invalid Transitions ─────────────────────────────────────────

    #[test]
    fn test_cannot_commit_from_idle() {
        let mut tpc = TwoPhaseCommit::new();
        assert!(matches!(tpc.commit(), Err(RcoError::TwoPcAbort)));
    }

    #[test]
    fn test_cannot_vote_from_idle() {
        let mut tpc = TwoPhaseCommit::new();
        assert!(matches!(tpc.vote_yes(0), Err(RcoError::TwoPcAbort)));
    }

    #[test]
    fn test_cannot_commit_from_prepared() {
        let mut tpc = TwoPhaseCommit::new();
        tpc.prepare(1, b"batch".to_vec(), dummy_anchor()).unwrap();
        assert!(matches!(tpc.commit(), Err(RcoError::TwoPcAbort)));
    }

    #[test]
    fn test_cannot_abort_from_vote_yes() {
        let mut tpc = TwoPhaseCommit::new();
        tpc.prepare(1, b"batch".to_vec(), dummy_anchor()).unwrap();
        tpc.vote_yes(0).unwrap();
        assert!(matches!(tpc.abort(), Err(RcoError::TwoPcAbort)));
    }

    #[test]
    fn test_cannot_commit_from_vote_no() {
        let mut tpc = TwoPhaseCommit::new();
        tpc.prepare(1, b"batch".to_vec(), dummy_anchor()).unwrap();
        tpc.vote_no().unwrap();
        assert!(matches!(tpc.commit(), Err(RcoError::TwoPcAbort)));
    }

    // ── Multiple Cycles ─────────────────────────────────────────────

    #[test]
    fn test_multiple_commit_cycles() {
        let mut tpc = TwoPhaseCommit::new();

        for i in 1..=5u64 {
            tpc.prepare(i, format!("batch_{i}").into_bytes(), dummy_anchor())
                .unwrap();
            tpc.vote_yes(i * 100).unwrap();
            tpc.commit().unwrap();
        }

        assert_eq!(tpc.committed_count(), 5);
        assert_eq!(tpc.aborted_count(), 0);
    }

    #[test]
    fn test_interleaved_commit_and_abort() {
        let mut tpc = TwoPhaseCommit::new();

        // Commit
        tpc.prepare(1, b"b1".to_vec(), dummy_anchor()).unwrap();
        tpc.vote_yes(100).unwrap();
        tpc.commit().unwrap();

        // Abort
        tpc.prepare(2, b"b2".to_vec(), dummy_anchor()).unwrap();
        tpc.vote_no().unwrap();
        tpc.abort().unwrap();

        // Commit again
        tpc.prepare(3, b"b3".to_vec(), dummy_anchor()).unwrap();
        tpc.vote_yes(200).unwrap();
        tpc.commit().unwrap();

        assert_eq!(tpc.committed_count(), 2);
        assert_eq!(tpc.aborted_count(), 1);
    }

    // ── State Queries ───────────────────────────────────────────────

    #[test]
    fn test_terminal_states() {
        assert!(TpcState::Committed.is_terminal());
        assert!(TpcState::Aborted.is_terminal());
        assert!(!TpcState::Idle.is_terminal());
        assert!(!TpcState::Prepared.is_terminal());
    }
}
