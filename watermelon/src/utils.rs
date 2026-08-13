use std::mem::MaybeUninit;

/// Represents a player color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    White,
    Black,
}

impl std::ops::Not for Color {
    type Output = Color;

    fn not(self) -> Self::Output {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

/// Represents a piece independent of color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

/// Represents a square.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
#[rustfmt::skip]
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

impl Square {
    #[inline]
    #[must_use]
    pub fn from_index(index: usize) -> Option<Self> {
        if index < 64 {
            // SAFETY: `Square` is represented entirely by `u32`, and we have guaranteed the
            // index lies in the range 0..64, which is what the enumeration represents.
            Some(unsafe { std::mem::transmute(index as u32) })
        } else {
            None
        }
    }

    #[inline]
    #[must_use]
    pub fn from_file_and_rank(file: usize, rank: usize) -> Option<Self> {
        if file < 8 && rank < 8 {
            Self::from_index(file + 8 * rank)
        } else {
            None
        }
    }

    #[inline]
    #[must_use]
    pub fn file(self) -> usize {
        (self as usize) & 7
    }

    #[inline]
    #[must_use]
    pub fn rank(self) -> usize {
        (self as usize) >> 3
    }

    /// Offset a square by a number without checking anything.
    ///
    /// ## Safety
    ///
    /// The addition must not overflow, nor result in an invalid square.
    pub(crate) unsafe fn offset_by_unchecked(self, offset: i32) -> Self {
        unsafe { std::mem::transmute((self as u32 as i32).unchecked_add(offset)) }
    }
}

/// 64-bit value corresponding to squares on a chessboard, with correspondence as shown:
///
/// ```txt
/// 8  56 57 58 59 60 61 62 63
/// 7  48 49 50 51 52 53 54 55
/// 6  40 41 42 43 44 45 46 47
/// 5  32 33 34 35 36 37 38 39
/// 4  24 25 26 27 28 29 30 31
/// 3  16 17 18 19 20 21 22 23
/// 2  08 09 10 11 12 13 14 15
/// 1  00 01 02 03 04 05 06 07
///     a  b  c  d  e  f  g  h
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub const EMPTY: Bitboard = Bitboard(0);
    pub const FULL: Bitboard = Bitboard(0xFFFFFFFFFFFFFFFF);
    pub const FILE_A: Bitboard = Bitboard(0x0101010101010101);
    pub const FILE_B: Bitboard = Bitboard(0x0202020202020202);
    pub const FILE_C: Bitboard = Bitboard(0x0404040404040404);
    pub const FILE_D: Bitboard = Bitboard(0x0808080808080808);
    pub const FILE_E: Bitboard = Bitboard(0x1010101010101010);
    pub const FILE_F: Bitboard = Bitboard(0x2020202020202020);
    pub const FILE_G: Bitboard = Bitboard(0x4040404040404040);
    pub const FILE_H: Bitboard = Bitboard(0x8080808080808080);
    pub const RANK_1: Bitboard = Bitboard(0x00000000000000FF);
    pub const RANK_2: Bitboard = Bitboard(0x000000000000FF00);
    pub const RANK_3: Bitboard = Bitboard(0x0000000000FF0000);
    pub const RANK_4: Bitboard = Bitboard(0x00000000FF000000);
    pub const RANK_5: Bitboard = Bitboard(0x000000FF00000000);
    pub const RANK_6: Bitboard = Bitboard(0x0000FF0000000000);
    pub const RANK_7: Bitboard = Bitboard(0x00FF000000000000);
    pub const RANK_8: Bitboard = Bitboard(0xFF00000000000000);

    #[inline]
    #[must_use]
    pub fn from_square(square: Square) -> Self {
        Bitboard(1u64 << (square as u32))
    }

    #[inline]
    #[must_use]
    pub fn is_set_at(self, square: Square) -> bool {
        (self.0 >> (square as u32)) & 1 == 1
    }

    #[inline]
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline]
    #[must_use]
    pub fn count_ones(self) -> u32 {
        self.0.count_ones()
    }
}

impl Iterator for Bitboard {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 != 0 {
            let trailing_zeros = self.0.trailing_zeros();
            self.0 &= self.0 - 1;

            // SAFETY: `Square` is represented entirely by `u32`, and we have guaranteed the
            // index lies in the range 0..64, which is what the enumeration represents.
            Some(unsafe { std::mem::transmute(trailing_zeros) })
        } else {
            None
        }
    }
}

impl std::ops::BitAnd for Bitboard {
    type Output = Bitboard;

    fn bitand(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl std::ops::BitOr for Bitboard {
    type Output = Bitboard;

    fn bitor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitXor for Bitboard {
    type Output = Bitboard;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl std::ops::Not for Bitboard {
    type Output = Bitboard;

    fn not(self) -> Self::Output {
        Bitboard(!self.0)
    }
}

impl std::ops::Shl<u32> for Bitboard {
    type Output = Bitboard;

    fn shl(self, rhs: u32) -> Self::Output {
        Bitboard(self.0 << rhs)
    }
}

impl std::ops::Shr<u32> for Bitboard {
    type Output = Bitboard;

    fn shr(self, rhs: u32) -> Self::Output {
        Bitboard(self.0 >> rhs)
    }
}

/// Castling rights for both sides: white/black + kingside/queenside.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CastlingRights(u8);

impl CastlingRights {
    pub const NONE: CastlingRights = CastlingRights(0b0000);
    pub const ALL: CastlingRights = CastlingRights(0b1111);
    pub const WHITE_KINGSIDE: CastlingRights = CastlingRights(0b0001);
    pub const WHITE_QUEENSIDE: CastlingRights = CastlingRights(0b0010);
    pub const BLACK_KINGSIDE: CastlingRights = CastlingRights(0b0100);
    pub const BLACK_QUEENSIDE: CastlingRights = CastlingRights(0b1000);

    #[inline]
    #[must_use]
    pub fn from_value(value: u8) -> Option<Self> {
        if value < 16 {
            Some(CastlingRights(value))
        } else {
            None
        }
    }

    #[inline]
    #[must_use]
    pub fn value(self) -> u8 {
        self.0
    }

    #[inline]
    #[must_use]
    pub fn has_any(self, other: CastlingRights) -> bool {
        self.0 & other.0 != 0
    }

    // TODO: Remove when Rust finally stabalizes const traits.
    #[inline]
    #[must_use]
    pub(crate) const fn const_or(self, other: CastlingRights) -> CastlingRights {
        CastlingRights(self.0 | other.0)
    }
}

impl std::ops::BitAnd for CastlingRights {
    type Output = CastlingRights;

    fn bitand(self, rhs: Self) -> Self::Output {
        CastlingRights(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for CastlingRights {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl std::ops::BitOr for CastlingRights {
    type Output = CastlingRights;

    fn bitor(self, rhs: Self) -> Self::Output {
        CastlingRights(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for CastlingRights {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::Not for CastlingRights {
    type Output = CastlingRights;

    fn not(self) -> Self::Output {
        CastlingRights(self.0 ^ 0b00001111)
    }
}

/// Flags that can be used to quickly look up useful characteristics of a [`Move`], such as capture
/// status or promoted-to piece.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MoveFlags {
    Quiet = 0b0000,
    DoublePawnPush = 0b0001,
    KingCastle = 0b0010,
    QueenCastle = 0b0011,
    Capture = 0b0100,
    EnPassantCapture = 0b0101,
    KnightPromotion = 0b1000,
    BishopPromotion = 0b1001,
    RookPromotion = 0b1010,
    QueenPromotion = 0b1011,
    KnightPromotionCapture = 0b1100,
    BishopPromotionCapture = 0b1101,
    RookPromotionCapture = 0b1110,
    QueenPromotionCapture = 0b1111,
}

impl MoveFlags {
    pub fn is_capture(self) -> bool {
        (self as u32) & 0b0100 != 0
    }

    pub fn is_en_passant(self) -> bool {
        self == Self::EnPassantCapture
    }

    pub fn is_promotion(self) -> bool {
        (self as u32) & 0b1000 != 0
    }

    pub fn promotion_piece(self) -> Option<Piece> {
        if self.is_promotion() {
            match (self as u32) & 0b0011 {
                0b00 => Some(Piece::Knight),
                0b01 => Some(Piece::Bishop),
                0b10 => Some(Piece::Rook),
                0b11 => Some(Piece::Queen),
                // This is unreachable but the optimizer may take care of that.
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn is_castle(self) -> bool {
        (self as u32) & 0b1110 == 0b0010
    }

    pub fn is_kingside_castle(self) -> bool {
        self == MoveFlags::KingCastle
    }

    pub fn is_queenside_castle(self) -> bool {
        self == MoveFlags::QueenCastle
    }
}

/// Compact representation of a chess move.
///
/// - For a natural representation see [`MoveIntent`].
/// - For a UCI representation see [`MoveIntent`].
/// - For a SAN representation see [`SanMove`].
///
/// [`MoveIntent`]: crate::notation::MoveIntent
/// [`SanMove`]: crate::notation::SanMove
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Move(u16);

impl Move {
    pub const NULL: Self = Move(0);

    #[inline]
    #[must_use]
    pub const fn new(from: Square, to: Square, flags: MoveFlags) -> Self {
        Move((flags as u16) << 12 | (from as u16) << 6 | (to as u16))
    }

    #[inline]
    #[must_use]
    pub const fn from(self) -> Square {
        // SAFETY: Safe by construction.
        unsafe { std::mem::transmute((self.0 as u32 >> 6) & 0x3F) }
    }

    #[inline]
    #[must_use]
    pub const fn to(self) -> Square {
        // SAFETY: Safe by construction.
        unsafe { std::mem::transmute((self.0 as u32) & 0x3F) }
    }

    #[inline]
    #[must_use]
    pub const fn flags(self) -> MoveFlags {
        // SAFETY: Safe by construction.
        unsafe { std::mem::transmute(self.0 as u32 >> 12) }
    }
}

/// Represents the positions of pieces on a board.
///
/// ## Safety
///
/// The only invariants this structure has is that none of the piece bitboards overlap, none of the
/// color bitboards overlap, and the bitwise-or of the piece bitboards equals the bitwise-or of the
/// color bitboards. Using a safe API on this structure guarantees these invariants.
///
/// Notice that this by itself does not enforce legality requirements, such as king or pawn
/// positioning. If you want an API that does those, see [`Position`].
#[derive(Clone, PartialEq, Eq)]
#[repr(align(64))]
pub struct RawPosition {
    piece_bitboards: [Bitboard; 6],
    color_bitboards: [Bitboard; 2],
}

impl RawPosition {
    /// Construct a [`RawPosition`] from a group of eight [`Bitboard`] structures: six for the pieces
    /// in order as they appear in the [`Piece`] enumeration, and two for the colors in order as they
    /// appear in the [`Color`] enumeration.
    #[inline]
    #[must_use]
    pub fn from_raw(
        piece_bitboards: [Bitboard; 6],
        color_bitboards: [Bitboard; 2],
    ) -> Option<Self> {
        // Enforce the invariant that none of the piece bitboards overlap.
        for i in 0..6 {
            for j in (i + 1)..6 {
                if piece_bitboards[i] & piece_bitboards[j] != Bitboard::EMPTY {
                    return None;
                }
            }
        }

        // Enforce the invariant that none of the color bitboards overlap.
        if color_bitboards[0] & color_bitboards[1] != Bitboard::EMPTY {
            return None;
        }

        // Enforce the invariant that the bitwise-or of both groups must equal.
        let piece_occupied = piece_bitboards
            .iter()
            .fold(Bitboard::EMPTY, |acc, &x| acc | x);
        let color_occupied = color_bitboards
            .iter()
            .fold(Bitboard::EMPTY, |acc, &x| acc | x);
        if piece_occupied != color_occupied {
            return None;
        }

        Some(RawPosition {
            piece_bitboards,
            color_bitboards,
        })
    }

    /// Returns the bitboard for a piece (all colors included).
    #[inline]
    #[must_use]
    pub fn piece_bitboard(&self, piece: Piece) -> Bitboard {
        // LLVM_FAILS_ELIDE_BOUNDS_CHECK_WHEN_INLINING
        unsafe { *self.piece_bitboards.get_unchecked(piece as usize) }
    }

    /// Returns the bitboard for a color (all pieces included).
    #[inline]
    #[must_use]
    pub fn color_bitboard(&self, color: Color) -> Bitboard {
        // LLVM_FAILS_ELIDE_BOUNDS_CHECK_WHEN_INLINING
        unsafe { *self.color_bitboards.get_unchecked(color as usize) }
    }

    /// Returns the bitboard for a specific piece and color.
    #[inline]
    #[must_use]
    pub fn bitboard(&self, piece: Piece, color: Color) -> Bitboard {
        self.piece_bitboard(piece) & self.color_bitboard(color)
    }

    /// Returns the bitboard representing all occupied squares.
    #[inline]
    #[must_use]
    pub fn occupied(&self) -> Bitboard {
        self.color_bitboard(Color::White) | self.color_bitboard(Color::Black)
    }

    /// Determines if a square is occupied.
    #[inline]
    #[must_use]
    pub fn is_occupied_at(&self, square: Square) -> bool {
        self.occupied().is_set_at(square)
    }

    /// Determines the piece and color on a square (if any).
    #[inline]
    #[must_use]
    pub fn piece_color_at(&self, square: Square) -> Option<(Piece, Color)> {
        if let Some(piece) = self.piece_at(square) {
            if let Some(color) = self.color_at(square) {
                Some((piece, color))
            } else {
                // SAFETY: By invariant a square must have a piece and color or neither.
                unsafe { std::hint::unreachable_unchecked() }
            }
        } else {
            None
        }
    }

    /// Determines the color on a square (if any).
    #[inline]
    #[must_use]
    pub fn color_at(&self, square: Square) -> Option<Color> {
        let is_white = (self.color_bitboards[0].0 >> (square as u32)) & 1 == 1;
        let is_black = (self.color_bitboards[1].0 >> (square as u32)) & 1 == 1;

        match (is_white, is_black) {
            // SAFETY: By invariant a square cannot be both black and white.
            (true, true) => unsafe { std::hint::unreachable_unchecked() },
            (true, false) => Some(Color::White),
            (false, true) => Some(Color::Black),
            (false, false) => None,
        }
    }

    /// Determines the piece on a square (if any).
    #[inline]
    #[must_use]
    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        if self.is_occupied_at(square) {
            Some(unsafe { self.piece_at_unchecked(square) })
        } else {
            None
        }
    }

    /// Removes the piece on a square (if any).
    #[inline]
    pub fn remove_piece_at(&mut self, square: Square) {
        for bitboard in self.piece_bitboards.iter_mut() {
            *bitboard &= !Bitboard::from_square(square);
        }
        for bitboard in self.color_bitboards.iter_mut() {
            *bitboard &= !Bitboard::from_square(square);
        }
    }

    /// Places a piece on a square (if any).
    #[inline]
    pub fn add_piece_at(&mut self, square: Square, piece: Piece, color: Color) {
        self.remove_piece_at(square);

        let mask = Bitboard::from_square(square);
        self.piece_bitboards[piece as usize] ^= mask;
        self.color_bitboards[color as usize] ^= mask;
    }

    /// Returns the piece on a square without checking if a piece actually lies there.
    ///
    /// ## Implementation detail
    ///
    /// If you are a user do not rely on this behavior: this does not trigger undefined behavior
    /// on empty squares and returns [`Piece::Pawn`] instead.
    // LLVM_FAILS_ELIDE_UNREACHABLE_BRANCH_WHEN_INLINING
    #[inline]
    #[must_use]
    pub unsafe fn piece_at_unchecked(&self, square: Square) -> Piece {
        let layer_0 = self.piece_bitboards[1] | self.piece_bitboards[3] | self.piece_bitboards[5];
        let layer_1 = self.piece_bitboards[2] | self.piece_bitboards[3];
        let layer_2 = self.piece_bitboards[4] | self.piece_bitboards[5];
        let bit_0 = (layer_0.0 >> (square as u32)) & 1;
        let bit_1 = (layer_1.0 >> (square as u32)) & 1;
        let bit_2 = (layer_2.0 >> (square as u32)) & 1;
        let decoded = bit_0 | (bit_1 << 1) | (bit_2 << 2);

        // SAFETY: This is always safe, the result is three bits.
        unsafe { std::mem::transmute(decoded as u8) }
    }

    #[inline]
    #[must_use]
    pub unsafe fn piece_bitboard_mut(&mut self, piece: Piece) -> &mut Bitboard {
        // LLVM_FAILS_ELIDE_BOUNDS_CHECK_WHEN_INLINING
        unsafe { self.piece_bitboards.get_unchecked_mut(piece as usize) }
    }

    #[inline]
    #[must_use]
    pub unsafe fn color_bitboard_mut(&mut self, color: Color) -> &mut Bitboard {
        // LLVM_FAILS_ELIDE_BOUNDS_CHECK_WHEN_INLINING
        unsafe { self.color_bitboards.get_unchecked_mut(color as usize) }
    }
}

/// Represents the positions of pieces on a board while enforcing these invariants:
/// - There must be one of each king on the board.
/// - Pawns must not be on the first or last ranks.
#[derive(Clone, PartialEq, Eq)]
pub struct Position(RawPosition);

impl Position {
    /// Construct a [`Position`] from a [`RawPosition`], checking if the invariants are met.
    #[inline]
    #[must_use]
    pub fn from_raw(raw_position: RawPosition) -> Option<Self> {
        if (raw_position.bitboard(Piece::King, Color::White)).count_ones() == 1
            && (raw_position.bitboard(Piece::King, Color::Black)).count_ones() == 1
            && (raw_position.piece_bitboard(Piece::Pawn) & Bitboard(0xFF000000000000FF)).is_empty()
        {
            Some(Position(raw_position))
        } else {
            None
        }
    }

    #[inline]
    #[must_use]
    pub fn as_raw(&self) -> &RawPosition {
        &self.0
    }

    #[inline]
    pub unsafe fn as_raw_mut(&mut self) -> &mut RawPosition {
        &mut self.0
    }

    #[inline]
    #[must_use]
    pub fn into_raw(self) -> RawPosition {
        self.0
    }

    #[inline]
    #[must_use]
    pub fn piece_bitboard(&self, piece: Piece) -> Bitboard {
        self.0.piece_bitboard(piece)
    }

    #[inline]
    #[must_use]
    pub fn color_bitboard(&self, color: Color) -> Bitboard {
        self.0.color_bitboard(color)
    }

    #[inline]
    #[must_use]
    pub fn bitboard(&self, piece: Piece, color: Color) -> Bitboard {
        self.0.bitboard(piece, color)
    }

    #[inline]
    #[must_use]
    pub fn occupied(&self) -> Bitboard {
        self.0.occupied()
    }

    #[inline]
    #[must_use]
    pub fn is_occupied_at(&self, square: Square) -> bool {
        self.0.is_occupied_at(square)
    }

    #[inline]
    #[must_use]
    pub fn piece_color_at(&self, square: Square) -> Option<(Piece, Color)> {
        self.0.piece_color_at(square)
    }

    #[inline]
    #[must_use]
    pub fn color_at(&self, square: Square) -> Option<Color> {
        self.0.color_at(square)
    }

    #[inline]
    #[must_use]
    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        self.0.piece_at(square)
    }

    /// Returns the square the king of the provided color is on.
    #[inline]
    #[must_use]
    pub fn king_square(&self, color: Color) -> Square {
        let square = self.0.bitboard(Piece::King, color).next();
        // SAFETY: There is always exactly one king of each color.
        unsafe { square.unwrap_unchecked() }
    }
}

/// Stack-allocated list of moves.
pub struct MoveList {
    moves: [MaybeUninit<Move>; 271],
    length: usize,
}

impl MoveList {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            moves: unsafe { MaybeUninit::uninit().assume_init() },
            length: 0,
        }
    }

    /// Pushes a move onto the list.
    ///
    /// ## Safety
    ///
    /// Undefined behavior occurs when there are at least 271 moves before calling this method.
    #[inline]
    pub unsafe fn push_unchecked(&mut self, mv: Move) {
        // SAFETY: The guarantee upon construction implies that `length <= 271` holds
        // after this call, which means `length < 271`, thus the indexing is safe.
        let uninit_slot = unsafe { self.moves.get_unchecked_mut(self.length) };
        uninit_slot.write(mv);
        self.length += 1;
    }

    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[Move] {
        // SAFETY: By construction `0..self.length` are valid and initialized offsets in the
        // `self.moves` array.
        unsafe {
            let uninit_slice = self.moves.get_unchecked(..self.length);
            &*(uninit_slice as *const [MaybeUninit<Move>] as *const [Move])
        }
    }

    #[inline]
    #[must_use]
    pub fn as_mut_slice(&mut self) -> &mut [Move] {
        // SAFETY: By construction `0..self.length` are valid and initialized offsets in the
        // `self.moves` array.
        unsafe {
            let uninit_slice = self.moves.get_unchecked_mut(..self.length);
            &mut *(uninit_slice as *mut [MaybeUninit<Move>] as *mut [Move])
        }
    }

    #[inline]
    pub fn retain<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&Move) -> bool,
    {
        let slice = self.as_mut_slice();

        let mut write_index = 0;
        for read_index in 0..slice.len() {
            if predicate(&slice[read_index]) {
                slice.swap(write_index, read_index);
                write_index += 1;
            }
        }
    }

    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> Move {
        let slice = self.as_mut_slice();
        let move_mut = &mut slice[index];
        let removed_move = *move_mut;
        let move_ptr = move_mut as *mut Move;

        // SAFETY: Successfully checked-indexing the slice means that there is at least one
        // element, which means there is a last element. The `move_ptr` is completely valid
        // as we just extracted it from a reference.
        unsafe {
            std::ptr::copy(slice.last().unwrap_unchecked(), move_ptr, 1);
        }

        self.length -= 1;
        removed_move
    }
}

impl AsRef<[Move]> for MoveList {
    fn as_ref(&self) -> &[Move] {
        self.as_slice()
    }
}

impl AsMut<[Move]> for MoveList {
    fn as_mut(&mut self) -> &mut [Move] {
        self.as_mut_slice()
    }
}

impl std::ops::Deref for MoveList {
    type Target = [Move];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl std::ops::DerefMut for MoveList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}
