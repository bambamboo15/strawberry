use crate::utils::*;

pub const SIDE_HASH: u64 = 0xa0f520a4c9fa5bcc;

#[inline(always)]
pub fn piece_square_hash(color: Color, piece: Piece, square: Square) -> u64 {
    let index = (color as usize * 8 + piece as usize) * 64 + square as usize;
    // SAFETY: The table has a length of 1024, and the index has a maximum value of 895.
    *unsafe { tables::PIECE_SQUARE_TABLE.get_unchecked(index) }
}

#[inline(always)]
pub fn en_passant_hash(square: Square) -> u64 {
    // SAFETY: The file of a square is <8, and the table has 8 elements, thus this indexing is safe.
    *unsafe { tables::EN_PASSANT_TABLE.get_unchecked(square.file()) }
}

#[inline(always)]
pub fn castling_hash(castling_rights: CastlingRights) -> u64 {
    // SAFETY: The value is only four bits, and this has length 16, thus this indexing is safe.
    *unsafe { tables::CASTLING_TABLE.get_unchecked(castling_rights.value() as usize) }
}

#[rustfmt::skip]
mod tables {
    pub static PIECE_SQUARE_TABLE: [u64; 2 * 8 * 64] = {
        const BYTES: &[u8; 2 * 8 * 64 * 8] = include_bytes!("bin/piece_square_table.bin");
        // SAFETY: This operation deals with raw binary data only.
        unsafe { std::mem::transmute(*BYTES) }
    };

    pub static EN_PASSANT_TABLE: [u64; 8] = [
        0x7d56d658294a9988, 0xa9b3f0ad4069bee5, 0x229d43362af3c697, 0x6cef3b131d75dc42,
        0xde0d71dd0844ad02, 0xe69238e766c44b4d, 0xf6d930ac4bc9584d, 0x586cdac18fc14df7,
    ];

    pub static CASTLING_TABLE: [u64; 16] = [
        0x67e85e44a0c80f99, 0x8cb3b4973dd5d7fe, 0xe2a7ad2f6a9172b4, 0x5e4fcb4dcee585c3,
        0x4416f6191e3975a5, 0x0c6f19a61656271d, 0x89805f6f8cac0c3c, 0xa617696c03834b1c,
        0x838e86519f962694, 0xc049b45051bcab2b, 0x9491712e92a85272, 0xd2d90a09c1085100,
        0x99e9a0ed5260434d, 0x23c40d5009d66ead, 0xc454a9bfe6a88045, 0x6cef81b4350535c6,
    ];
}
