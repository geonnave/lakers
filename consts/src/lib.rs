#![no_std]

pub use consts::*;
pub use helpers::*;
pub use structs::*;

mod consts {
    use super::structs::*;

    // TODO: find a way to configure the buffer size
    // need 128 to handle EAD fields, and 192 for the EAD_1 voucher
    pub const MAX_MESSAGE_SIZE_LEN: usize = 128 + 64;
    pub type EADMessageBuffer = EdhocMessageBuffer; // TODO: make it of size MAX_EAD_SIZE_LEN

    pub const ID_CRED_LEN: usize = 4;
    pub const SUITES_LEN: usize = 9;
    pub const SUPPORTED_SUITES_LEN: usize = 1;
    pub const EDHOC_METHOD: u8 = 3u8; // stat-stat is the only supported method
    pub const P256_ELEM_LEN: usize = 32;
    pub const SHA256_DIGEST_LEN: usize = 32;
    pub const AES_CCM_KEY_LEN: usize = 16;
    pub const AES_CCM_IV_LEN: usize = 13;
    pub const AES_CCM_TAG_LEN: usize = 8;
    pub const MAC_LENGTH: usize = 8; // used for EAD Zeroconf
    pub const MAC_LENGTH_2: usize = MAC_LENGTH;
    pub const MAC_LENGTH_3: usize = MAC_LENGTH_2;
    pub const ENCODED_VOUCHER_LEN: usize = 1 + MAC_LENGTH; // 1 byte for the length of the bstr-encoded voucher

    // maximum supported length of connection identifier for R
    pub const MAX_KDF_CONTEXT_LEN: usize = 150;
    pub const MAX_KDF_LABEL_LEN: usize = 15; // for "KEYSTREAM_2"
    pub const MAX_BUFFER_LEN: usize = 220;
    pub const CBOR_BYTE_STRING: u8 = 0x58u8;
    pub const CBOR_TEXT_STRING: u8 = 0x78u8;
    pub const CBOR_UINT_1BYTE: u8 = 0x18u8;
    pub const CBOR_NEG_INT_1BYTE_START: u8 = 0x20u8;
    pub const CBOR_NEG_INT_1BYTE_END: u8 = 0x37u8;
    pub const CBOR_UINT_1BYTE_START: u8 = 0x0u8;
    pub const CBOR_UINT_1BYTE_END: u8 = 0x17u8;
    pub const CBOR_MAJOR_TEXT_STRING: u8 = 0x60u8;
    pub const CBOR_MAJOR_BYTE_STRING: u8 = 0x40u8;
    pub const CBOR_MAJOR_BYTE_STRING_MAX: u8 = 0x57u8;
    pub const CBOR_MAJOR_ARRAY: u8 = 0x80u8;
    pub const CBOR_MAJOR_ARRAY_MAX: u8 = 0x97u8;
    pub const MAX_INFO_LEN: usize = 2 + SHA256_DIGEST_LEN + // 32-byte digest as bstr
				            1 + MAX_KDF_LABEL_LEN +     // label <24 bytes as tstr
						    1 + MAX_KDF_CONTEXT_LEN +   // context <24 bytes as bstr
						    1; // length as u8

    pub const ENC_STRUCTURE_LEN: usize = 8 + 5 + SHA256_DIGEST_LEN; // 8 for ENCRYPT0

    pub const EDHOC_SUITES: BytesSuites = [0, 1, 2, 3, 4, 5, 6, 24, 25]; // all but private cipher suites
    pub const EDHOC_SUPPORTED_SUITES: BytesSupportedSuites = [0x2u8];

    pub const MAX_EAD_SIZE_LEN: usize = 64;
    pub const EAD_ZEROCONF_LABEL: u8 = 0x1; // NOTE: in lake-authz-draft-02 it is still TBD1
    pub const EAD_ZEROCONF_INFO_K_1_LABEL: u8 = 0x0;
    pub const EAD_ZEROCONF_INFO_IV_1_LABEL: u8 = 0x1;
    pub const EAD_ZEROCONF_ENC_STRUCTURE_LEN: usize = 2 + 8 + 3;
}

mod structs {
    use super::consts::*;

    pub type BytesEad2 = [u8; 0];
    pub type BytesIdCred = [u8; ID_CRED_LEN];
    pub type BytesSuites = [u8; SUITES_LEN];
    pub type BytesSupportedSuites = [u8; SUPPORTED_SUITES_LEN];
    pub type Bytes8 = [u8; 8];
    pub type BytesCcmKeyLen = [u8; AES_CCM_KEY_LEN];
    pub type BytesCcmIvLen = [u8; AES_CCM_IV_LEN];
    pub type BufferPlaintext2 = EdhocMessageBuffer;
    pub type BufferPlaintext3 = EdhocMessageBuffer;
    pub type BytesMac2 = [u8; MAC_LENGTH_2];
    pub type BytesMac3 = [u8; MAC_LENGTH_3];
    pub type BufferMessage1 = EdhocMessageBuffer;
    pub type BufferMessage3 = EdhocMessageBuffer;
    pub type BufferCiphertext2 = EdhocMessageBuffer;
    pub type BufferCiphertext3 = EdhocMessageBuffer;
    pub type BytesHashLen = [u8; SHA256_DIGEST_LEN];
    pub type BytesP256ElemLen = [u8; P256_ELEM_LEN];
    pub type BufferMessage2 = EdhocMessageBuffer;
    pub type BytesMaxBuffer = [u8; MAX_BUFFER_LEN];
    pub type BytesMaxContextBuffer = [u8; MAX_KDF_CONTEXT_LEN];
    pub type BytesMaxInfoBuffer = [u8; MAX_INFO_LEN];
    pub type BytesMaxLabelBuffeer = [u8; MAX_KDF_LABEL_LEN];
    pub type BytesEncStructureLen = [u8; ENC_STRUCTURE_LEN];

    pub type BytesMac = [u8; MAC_LENGTH];
    pub type BytesEncodedVoucher = [u8; ENCODED_VOUCHER_LEN];

    #[repr(C)]
    #[derive(Default, PartialEq, Copy, Clone, Debug)]
    pub enum EDHOCState {
        #[default]
        Start = 0, // initiator and responder
        WaitMessage2 = 1,      // initiator
        ProcessedMessage2 = 2, // initiator
        ProcessedMessage1 = 3, // responder
        WaitMessage3 = 4,      // responder
        Completed = 5,         // initiator and responder
    }

    #[repr(C)]
    #[derive(PartialEq, Debug)]
    pub enum EDHOCError {
        Success = 0,
        UnknownPeer = 1,
        MacVerificationFailed = 2,
        UnsupportedMethod = 3,
        UnsupportedCipherSuite = 4,
        ParsingError = 5,
        WrongState = 6,
        EADError = 7,
        UnknownError = 8,
    }

    #[repr(C)]
    #[derive(Default, Copy, Clone, Debug)]
    pub struct State(
        pub EDHOCState,
        pub BytesP256ElemLen, // x or y, ephemeral private key of myself
        pub u8,               // c_i, connection identifier chosen by the initiator
        pub BytesP256ElemLen, // g_y or g_x, ephemeral public key of the peer
        pub BytesHashLen,     // prk_3e2m
        pub BytesHashLen,     // prk_4e3m
        pub BytesHashLen,     // prk_out
        pub BytesHashLen,     // prk_exporter
        pub BytesHashLen,     // h_message_1
        pub BytesHashLen,     // th_3
    );

    #[repr(C)]
    #[derive(PartialEq, Debug, Copy, Clone)]
    pub struct EdhocMessageBuffer {
        pub content: [u8; MAX_MESSAGE_SIZE_LEN],
        pub len: usize,
    }

    impl Default for EdhocMessageBuffer {
        fn default() -> Self {
            EdhocMessageBuffer {
                content: [0; MAX_MESSAGE_SIZE_LEN],
                len: 0,
            }
        }
    }

    pub trait MessageBufferTrait {
        fn new() -> Self;
        fn from_hex(hex: &str) -> Self;
    }

    impl MessageBufferTrait for EdhocMessageBuffer {
        fn new() -> Self {
            EdhocMessageBuffer {
                content: [0u8; MAX_MESSAGE_SIZE_LEN],
                len: 0,
            }
        }
        fn from_hex(hex: &str) -> Self {
            let mut buffer = EdhocMessageBuffer::new();
            buffer.len = hex.len() / 2;
            for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
                let chunk_str = core::str::from_utf8(chunk).unwrap();
                buffer.content[i] = u8::from_str_radix(chunk_str, 16).unwrap();
            }
            buffer
        }
    }

    impl TryInto<EdhocMessageBuffer> for &[u8] {
        type Error = ();

        fn try_into(self) -> Result<EdhocMessageBuffer, Self::Error> {
            if self.len() <= MAX_MESSAGE_SIZE_LEN {
                let mut buffer = [0u8; MAX_MESSAGE_SIZE_LEN];
                for i in 0..self.len() {
                    buffer[i] = self[i];
                }

                Ok(EdhocMessageBuffer {
                    content: buffer,
                    len: self.len(),
                })
            } else {
                Err(())
            }
        }
    }

    #[derive(Debug)]
    pub struct EADItem {
        pub label: u8,
        pub is_critical: bool,
        // TODO[ead]: have adjustable (smaller) length for this buffer
        pub value: Option<EdhocMessageBuffer>,
    }

    pub trait EADTrait {
        fn new() -> Self;
    }

    impl EADTrait for EADItem {
        fn new() -> Self {
            EADItem {
                label: 0,
                is_critical: false,
                value: None,
            }
        }
    }

    #[derive(Debug)]
    pub enum IdCred<'a> {
        CompactKid(u8),
        FullCredential(&'a Credential<'a>),
    }

    #[derive(Default, Copy, Clone, Debug)]
    pub struct Credential<'a> {
        pub value: EdhocMessageBuffer,
        pub g: &'a [u8],
        pub kid: u8,
    }

    impl<'a> Credential<'a> {
        pub fn new(value: &'a [u8]) -> Self {
            let (g, kid) = Self::parse_cred(value);
            let value: EdhocMessageBuffer = value.try_into().unwrap();

            Credential { value, kid, g }
        }

        pub fn parse_cred(cred: &'a [u8]) -> (&'a [u8], u8) {
            let subject_len = (cred[2] - CBOR_MAJOR_TEXT_STRING) as usize;
            let id_cred_offset: usize = 3 + subject_len + 8;
            let g_a_x_offset: usize = id_cred_offset + 6;

            (
                &cred[g_a_x_offset..g_a_x_offset + P256_ELEM_LEN],
                cred[id_cred_offset],
            )
        }

        pub fn get_id_cred(self) -> BytesIdCred {
            [0xa1, 0x04, 0x41, self.kid]
        }

        pub fn get_value_as_slice(&'a self) -> &'a [u8] {
            &self.value.content[..self.value.len]
        }
    }
}

mod helpers {
    use super::consts::*;
    use super::structs::*;

    /// Check for: an unsigned integer encoded as a single byte
    #[inline(always)]
    pub fn is_cbor_uint_1byte(byte: u8) -> bool {
        return byte >= CBOR_UINT_1BYTE_START && byte <= CBOR_UINT_1BYTE_END;
    }

    /// Check for: an unsigned integer encoded as two bytes
    #[inline(always)]
    pub fn is_cbor_uint_2bytes(byte: u8) -> bool {
        return byte == CBOR_UINT_1BYTE;
    }

    /// Check for: a negative integer encoded as a single byte
    #[inline(always)]
    pub fn is_cbor_neg_int_1byte(byte: u8) -> bool {
        return byte >= CBOR_NEG_INT_1BYTE_START && byte <= CBOR_NEG_INT_1BYTE_END;
    }

    /// Check for: a bstr denoted by a single byte which encodes both type and content length
    #[inline(always)]
    pub fn is_cbor_bstr_1byte_prefix(byte: u8) -> bool {
        return byte >= CBOR_MAJOR_BYTE_STRING && byte <= CBOR_MAJOR_BYTE_STRING_MAX;
    }

    /// Check for: a bstr denoted by two bytes, one for type the other for content length
    #[inline(always)]
    pub fn is_cbor_bstr_2bytes_prefix(byte: u8) -> bool {
        return byte == CBOR_BYTE_STRING;
    }

    /// Check for: a tstr denoted by two bytes, one for type the other for content length
    #[inline(always)]
    pub fn is_cbor_tstr_2bytes_prefix(byte: u8) -> bool {
        return byte == CBOR_TEXT_STRING;
    }

    /// Check for: an array denoted by a single byte which encodes both type and content length
    #[inline(always)]
    pub fn is_cbor_array_1byte_prefix(byte: u8) -> bool {
        return byte >= CBOR_MAJOR_ARRAY && byte <= CBOR_MAJOR_ARRAY_MAX;
    }

    pub fn encode_info(
        label: u8,
        context: &BytesMaxContextBuffer,
        context_len: usize,
        length: usize,
    ) -> (BytesMaxInfoBuffer, usize) {
        let mut info: BytesMaxInfoBuffer = [0x00; MAX_INFO_LEN];

        // construct info with inline cbor encoding
        info[0] = label;
        let mut info_len = if context_len < 24 {
            info[1] = context_len as u8 | CBOR_MAJOR_BYTE_STRING;
            info[2..2 + context_len].copy_from_slice(&context[..context_len]);
            2 + context_len
        } else {
            info[1] = CBOR_BYTE_STRING;
            info[2] = context_len as u8;
            info[3..3 + context_len].copy_from_slice(&context[..context_len]);
            3 + context_len
        };

        info_len = if length < 24 {
            info[info_len] = length as u8;
            info_len + 1
        } else {
            info[info_len] = CBOR_UINT_1BYTE;
            info[info_len + 1] = length as u8;
            info_len + 2
        };

        (info, info_len)
    }
}

#[cfg(test)]
mod test {
    use super::structs::*;
    use hexlit::hex;

    const CRED_TV: &[u8] = &hex!("a2026b6578616d706c652e65647508a101a501020241322001215820bbc34960526ea4d32e940cad2a234148ddc21791a12afbcbac93622046dd44f02258204519e257236b2a0ce2023f0931f1f386ca7afda64fcde0108c224c51eabf6072");
    const G_A_TV: &[u8] = &hex!("BBC34960526EA4D32E940CAD2A234148DDC21791A12AFBCBAC93622046DD44F0");
    const ID_CRED_TV: &[u8] = &hex!("a1044132");

    #[test]
    fn test_parse_cred() {
        let (g_a, kid) = Credential::parse_cred(CRED_TV);
        assert_eq!(g_a, G_A_TV);
        assert_eq!(kid, ID_CRED_TV[3]);
    }

    #[test]
    fn test_new_credential() {
        let cred_tv: EdhocMessageBuffer = CRED_TV.try_into().unwrap();

        let cred = Credential::new(CRED_TV);
        assert_eq!(cred.g, G_A_TV);
        assert_eq!(cred.kid, ID_CRED_TV[3]);
        assert_eq!(cred.value, cred_tv);
    }
}
