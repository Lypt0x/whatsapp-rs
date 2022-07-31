#![allow(unused)]

macro_rules! declare_tag {
    (
        $($name:ident => $val:expr)*
    ) => {
        $(
            pub(crate) const $name: i32 = $val;
        )*
    };
}

declare_tag! {
    UNKMOWN => -1
    LIST_EMPTY => 0
    STREAM_END => 2
    DICTIONARY_ZERO => 236
    DICTIONARY_ONE => 237
    DICTIONARY_TWO => 238
    DICTIONARY_THREE => 239
    COMPANION_JID => 247
    LIST_EIGHT => 248
    LIST_SIXTEEN => 249
    JID_PAIR => 250
    HEX_EIGHT => 251
    BINARY_EIGHT => 252
    BINARY_TWENTY => 253
    BINARY_THIRTY_TWO => 254
    NIBBLE_EIGHT => 255
    SINGLE_BYTE_MAX => 256
    PACKED_MAX => 254
}
