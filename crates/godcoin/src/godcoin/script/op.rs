use crate::crypto::PublicKey;

#[derive(PartialEq)]
#[repr(u8)]
pub enum Operand {
    // Push value
    PushFalse = 0x0,
    PushTrue = 0x1,
    PushPubKey = 0x2,

    // Control
    OpIf = 0x10,
    OpElse = 0x11,
    OpEndIf = 0x12,
    OpReturn = 0x13,

    // Crypto
    OpCheckSigLaxMode = 0x20,
    OpCheckSig = 0x21,
    OpCheckMultiSig = 0x22,
}

impl From<Operand> for u8 {
    fn from(op: Operand) -> u8 {
        op as u8
    }
}

#[derive(PartialEq)]
pub enum OpFrame {
    // Push value
    False,
    True,
    PubKey(PublicKey),

    // Control
    OpIf,
    OpElse,
    OpEndIf,
    OpReturn,

    // Crypto
    OpCheckSigLaxMode,
    OpCheckSig,
    OpCheckMultiSig(u8, u8), // M of N: minimum threshold to number of keys
}

impl From<bool> for OpFrame {
    fn from(b: bool) -> OpFrame {
        if b {
            OpFrame::True
        } else {
            OpFrame::False
        }
    }
}
