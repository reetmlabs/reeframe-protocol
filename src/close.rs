/// Why the Coordinator closed a control channel. The code goes to the backend
/// as the QUIC application close code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CloseReason {
    /// The backend closed the channel itself.
    Closed,
    /// The backend's key is unknown, or its site is not active.
    UnknownKey,
    /// A newer connection for the same site took over.
    Replaced,
    /// No message arrived within the heartbeat timeout.
    Timeout,
    /// The site was revoked.
    Revoked,
    /// The backend broke the protocol.
    Protocol,
}

impl CloseReason {
    /// Every reason, in code order.
    pub const ALL: [Self; 6] = [
        Self::Closed,
        Self::UnknownKey,
        Self::Replaced,
        Self::Timeout,
        Self::Revoked,
        Self::Protocol,
    ];

    /// The application close code on the wire.
    pub fn code(self) -> u32 {
        match self {
            Self::Closed => 0,
            Self::UnknownKey => 1,
            Self::Replaced => 2,
            Self::Timeout => 3,
            Self::Revoked => 4,
            Self::Protocol => 5,
        }
    }

    /// The reason for a close code, or `None` for a code this version does
    /// not know, so a newer Coordinator can add codes.
    pub fn from_code(code: u32) -> Option<Self> {
        Self::ALL.into_iter().find(|reason| reason.code() == code)
    }

    /// A short text sent along with the close code.
    pub fn text(self) -> &'static str {
        match self {
            Self::Closed => "closed",
            Self::UnknownKey => "unknown or inactive site key",
            Self::Replaced => "replaced by a newer connection",
            Self::Timeout => "heartbeat timeout",
            Self::Revoked => "site revoked",
            Self::Protocol => "protocol error",
        }
    }
}

impl From<CloseReason> for u32 {
    fn from(reason: CloseReason) -> Self {
        reason.code()
    }
}

impl TryFrom<u32> for CloseReason {
    type Error = u32;

    /// Fails with the code itself when it is unknown.
    fn try_from(code: u32) -> Result<Self, Self::Error> {
        Self::from_code(code).ok_or(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The numbers are on the wire, so they must never change.
    #[test]
    fn codes_have_fixed_numbers() {
        let codes: Vec<u32> = CloseReason::ALL.iter().map(|r| r.code()).collect();
        assert_eq!(codes, [0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn a_code_converts_back_to_its_reason() {
        for reason in CloseReason::ALL {
            assert_eq!(CloseReason::try_from(u32::from(reason)), Ok(reason));
        }
        assert_eq!(CloseReason::from_code(6), None);
        assert_eq!(CloseReason::try_from(6), Err(6));
    }
}
