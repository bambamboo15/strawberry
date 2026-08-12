use crate::movegen::{self, Black, ColorLogic, White};
use crate::utils::*;
use crate::zobrist;

/// Represents a chess game in action.
pub struct Game {
    positions: Vec<Position>,
    undo_state: Vec<UndoState>,
    color: Color,
}

impl Game {
    /// Construct a [`Game`] from the raw components: piece placement ([`Position`]),
    /// turn ([`Color`]), and other state.
    #[inline]
    #[must_use]
    pub fn from_raw(
        position: Position,
        color: Color,
        castling_rights: CastlingRights,
        en_passant_square: Option<Square>,
        half_move_counter: usize,
    ) -> Option<Self> {
        // Enforce bitboard invariants:
        //  - there must be one king of each side
        //  - pawns must not exist on the first and last ranks
        if (position.bitboard(Piece::King, Color::White)).count_ones() != 1
            || (position.bitboard(Piece::King, Color::Black)).count_ones() != 1
            || !(position.piece_bitboard(Piece::Pawn) & Bitboard(0xFF000000000000FF)).is_empty()
        {
            return None;
        }

        // Enforce state invariants:
        //  - castling rights (per set bit) must make sense with king and rook positionings
        //  - en-passant square (if any) must be on the correct rank
        //  - en-passant square (if any) must be under a pawn like it was double-pushed
        // TODO

        // Compute the Zobrist hash from scratch.
        let mut zobrist_hash = 0;
        for square in position.occupied() {
            if let Some((piece, color)) = position.piece_color_at(square) {
                zobrist_hash ^= zobrist::piece_square_hash(color, piece, square);
            }
        }
        if color == Color::Black {
            zobrist_hash ^= zobrist::SIDE_HASH;
        }
        if let Some(ep_square) = en_passant_square {
            zobrist_hash ^= zobrist::en_passant_hash(ep_square);
        }
        zobrist_hash ^= zobrist::castling_hash(castling_rights);

        Some(Game {
            positions: vec![position],
            undo_state: vec![UndoState {
                castling_rights,
                en_passant_square,
                zobrist_hash,
                half_move_counter,
            }],
            color,
        })
    }

    #[inline]
    #[must_use]
    pub fn from_branch(game: &Game) -> Self {
        Game {
            positions: vec![game.position().clone()],
            undo_state: vec![game.undo_state().clone()],
            color: game.color,
        }
    }

    /// Returns the [color][`Color`] of the current player.
    #[inline]
    #[must_use]
    pub fn color(&self) -> Color {
        self.color
    }

    /// Returns the current [position][`Position`] on the board.
    #[inline]
    #[must_use]
    pub fn position(&self) -> &Position {
        // SAFETY: There is always at least one position in the vector.
        unsafe { self.positions.last().unwrap_unchecked() }
    }

    /// Returns the current [castling rights][`CastlingRights`].
    #[inline]
    #[must_use]
    pub fn castling_rights(&self) -> CastlingRights {
        self.undo_state().castling_rights
    }

    /// Returns the current en-passant square (if any).
    #[inline]
    #[must_use]
    pub fn en_passant_square(&self) -> Option<Square> {
        self.undo_state().en_passant_square
    }

    /// Returns the Zobrist hash for the current position.
    #[inline]
    #[must_use]
    pub fn zobrist_hash(&self) -> u64 {
        self.undo_state().zobrist_hash
    }

    /// Returns the half-move counter for the current position.
    #[inline]
    #[must_use]
    pub fn half_move_counter(&self) -> usize {
        self.undo_state().half_move_counter
    }

    /// Returns the total number of half-moves (plies) made since the start of the game
    /// (from when the [`Game`] object was constructed).
    #[inline]
    #[must_use]
    pub fn ply_count(&self) -> usize {
        self.undo_state.len().wrapping_sub(1)
    }

    /// Returns a [`ZobristHashHistory`] object that references all Zobrist hashes from the
    /// starting position to the current one.
    #[inline]
    #[must_use]
    pub fn zobrist_hash_history(&self) -> ZobristHashHistory<'_> {
        ZobristHashHistory(self)
    }

    fn undo_state(&self) -> &UndoState {
        // SAFETY: There is always at least one undo state (representing current position) in the
        // vector.
        unsafe { self.undo_state.last().unwrap_unchecked() }
    }

    /// Returns the square the king of the provided color is on.
    #[inline]
    #[must_use]
    pub fn king_square(&self, color: Color) -> Square {
        let square = self.position().bitboard(Piece::King, color).next();
        // SAFETY: There is always exactly one king of each color.
        unsafe { square.unwrap_unchecked() }
    }

    /// Determines if the current player is in check.
    #[inline]
    #[must_use]
    pub fn is_in_check(&self) -> bool {
        let position = self.position();
        let color = self.color;
        let square = self.king_square(self.color);
        // SAFETY: This will always represent a valid chess position.
        let attackers = unsafe { crate::movegen::square_attackers(position, !color, square) };
        attackers != Bitboard::EMPTY
    }

    /// Checks if the game is over due to [checkmate][`MateStatus::Checkmate`] or
    /// [stalemate][`MateStatus::Stalemate`].
    #[inline]
    #[must_use]
    pub fn mate_status(&self) -> Option<MateStatus> {
        if self.legal_moves_raw().is_empty() {
            if self.is_in_check() {
                Some(MateStatus::Checkmate)
            } else {
                Some(MateStatus::Stalemate)
            }
        } else {
            None
        }
    }

    /// Determines if the position is a draw by the fifty-move rule **without considering checkmate**.
    ///
    /// According to official FIDE chess rules, checkmate overrides the 50-move rule.
    /// Callers must check [`Game::mate_status`] first; if it returns `Some(MateStatus::Checkmate)`
    /// the game is a win, and this draw condition should be ignored.
    #[inline]
    #[must_use]
    pub fn is_fifty_move_draw(&self) -> bool {
        self.half_move_counter() > 99
    }

    /// Determines if the **current position** has occurred three or more times.
    ///
    /// Past threefold repetitions that do not match the current position will not be detected.
    #[inline]
    #[must_use]
    pub fn is_threefold_repetition(&self) -> bool {
        std::iter::zip(&self.positions, &self.undo_state)
            .filter(|&(position, undo_state)| {
                undo_state.zobrist_hash == self.zobrist_hash()
                    && undo_state.castling_rights == self.castling_rights()
                    && undo_state.en_passant_square == self.en_passant_square()
                    && position == self.position()
            })
            .count()
            >= 3
    }

    /// If the provided move is legal, this plays it and returns `true`, returning `false` otherwise.
    #[inline]
    pub fn try_play(&mut self, mv: Move) -> bool {
        if self.is_legal_move_raw(mv) {
            unsafe { self.play_unchecked(mv) };
            true
        } else {
            false
        }
    }

    /// If there was a last move, this undoes it and returns `true`, returning `false` otherwise.
    #[inline]
    pub fn try_undo(&mut self) -> bool {
        if self.positions.len() > 1 {
            unsafe { self.undo_unchecked() };
            true
        } else {
            false
        }
    }

    /// Plays a move without checking its legality.
    ///
    /// ## Safety
    ///
    /// The move must be legal.
    #[inline]
    pub unsafe fn play_unchecked(&mut self, mv: Move) {
        let position = self.position().clone();
        let undo_state = self.undo_state().clone();
        // TODO: As of now this results in two branches as it is not guaranteed that the capacities
        // are equal. I don't think this is worth it to optimize though.
        // SAFETY: All forms of initialization bring them to the same length. Every play appends one
        // to both, and every undo removes one from both. Thus they always have the same length.
        unsafe { std::hint::assert_unchecked(self.positions.len() == self.undo_state.len()) };
        let position = self.positions.push_mut(position);
        let undo_state = self.undo_state.push_mut(undo_state);

        let (from, to, flags) = (mv.from(), mv.to(), mv.flags());
        let piece_from = unsafe { position.piece_at_unchecked(from) };
        let piece_to = unsafe { position.piece_at_unchecked(to) };

        // Update the half-move counter.
        undo_state.half_move_counter = std::hint::select_unpredictable(
            piece_from == Piece::Pawn || mv.flags().is_capture(),
            0,
            undo_state.half_move_counter + 1,
        );

        // En-passant.
        if let Some(ep_square) = undo_state.en_passant_square {
            undo_state.zobrist_hash ^= zobrist::en_passant_hash(ep_square);
        }

        undo_state.en_passant_square = if flags == MoveFlags::DoublePawnPush {
            let square = unsafe { from.offset_by_unchecked(8 - 16 * self.color as i32) };
            undo_state.zobrist_hash ^= zobrist::en_passant_hash(square);
            Some(square)
        } else {
            None
        };

        // Castling rights.
        undo_state.zobrist_hash ^= zobrist::castling_hash(undo_state.castling_rights);
        if self.color == Color::White {
            if piece_to == Piece::Rook {
                if to == Black::KINGSIDE_CASTLE_ROOK_FROM_SQUARE {
                    undo_state.castling_rights &= !Black::KINGSIDE_CASTLE_FLAG;
                } else if to == Black::QUEENSIDE_CASTLE_ROOK_FROM_SQUARE {
                    undo_state.castling_rights &= !Black::QUEENSIDE_CASTLE_FLAG;
                }
            }
            if piece_from == Piece::King {
                undo_state.castling_rights &=
                    !(White::KINGSIDE_CASTLE_FLAG | White::QUEENSIDE_CASTLE_FLAG);
            } else if piece_from == Piece::Rook {
                if from == White::KINGSIDE_CASTLE_ROOK_FROM_SQUARE {
                    undo_state.castling_rights &= !White::KINGSIDE_CASTLE_FLAG;
                } else if from == White::QUEENSIDE_CASTLE_ROOK_FROM_SQUARE {
                    undo_state.castling_rights &= !White::QUEENSIDE_CASTLE_FLAG;
                }
            }
        } else {
            if piece_to == Piece::Rook {
                if to == White::KINGSIDE_CASTLE_ROOK_FROM_SQUARE {
                    undo_state.castling_rights &= !White::KINGSIDE_CASTLE_FLAG;
                } else if to == White::QUEENSIDE_CASTLE_ROOK_FROM_SQUARE {
                    undo_state.castling_rights &= !White::QUEENSIDE_CASTLE_FLAG;
                }
            }
            if piece_from == Piece::King {
                undo_state.castling_rights &=
                    !(Black::KINGSIDE_CASTLE_FLAG | Black::QUEENSIDE_CASTLE_FLAG);
            } else if piece_from == Piece::Rook {
                if from == Black::KINGSIDE_CASTLE_ROOK_FROM_SQUARE {
                    undo_state.castling_rights &= !Black::KINGSIDE_CASTLE_FLAG;
                } else if from == Black::QUEENSIDE_CASTLE_ROOK_FROM_SQUARE {
                    undo_state.castling_rights &= !Black::QUEENSIDE_CASTLE_FLAG;
                }
            }
        }
        undo_state.zobrist_hash ^= zobrist::castling_hash(undo_state.castling_rights);

        // Capture.
        if flags.is_capture() {
            let (captured_piece, captured_square) = if flags.is_en_passant() {
                (Piece::Pawn, unsafe {
                    to.offset_by_unchecked(if self.color == Color::White { -8 } else { 8 })
                })
            } else {
                (piece_to, to)
            };

            undo_state.zobrist_hash ^=
                zobrist::piece_square_hash(!self.color, captured_piece, captured_square);

            let mask = Bitboard::from_square(captured_square);
            unsafe {
                *position.piece_bitboard_mut(captured_piece) ^= mask;
                *position.color_bitboard_mut(!self.color) ^= mask;
            }
        }

        // Promotion and normal movement.
        let from_mask = Bitboard::from_square(from);
        let to_mask = Bitboard::from_square(to);
        let placed_piece = flags.promotion_piece().unwrap_or(piece_from);
        unsafe {
            *position.piece_bitboard_mut(piece_from) ^= from_mask;
            *position.piece_bitboard_mut(placed_piece) ^= to_mask;
            *position.color_bitboard_mut(self.color) ^= from_mask ^ to_mask;
        }
        undo_state.zobrist_hash ^= zobrist::piece_square_hash(self.color, piece_from, from);
        undo_state.zobrist_hash ^= zobrist::piece_square_hash(self.color, placed_piece, to);

        // Castling.
        if flags.is_castle() {
            let (rook_from, rook_to) = if flags.is_kingside_castle() {
                if self.color == Color::White {
                    (
                        White::KINGSIDE_CASTLE_ROOK_FROM_SQUARE,
                        White::KINGSIDE_CASTLE_ROOK_TO_SQUARE,
                    )
                } else {
                    (
                        Black::KINGSIDE_CASTLE_ROOK_FROM_SQUARE,
                        Black::KINGSIDE_CASTLE_ROOK_TO_SQUARE,
                    )
                }
            } else {
                if self.color == Color::White {
                    (
                        White::QUEENSIDE_CASTLE_ROOK_FROM_SQUARE,
                        White::QUEENSIDE_CASTLE_ROOK_TO_SQUARE,
                    )
                } else {
                    (
                        Black::QUEENSIDE_CASTLE_ROOK_FROM_SQUARE,
                        Black::QUEENSIDE_CASTLE_ROOK_TO_SQUARE,
                    )
                }
            };

            undo_state.zobrist_hash ^=
                zobrist::piece_square_hash(self.color, Piece::Rook, rook_from);
            undo_state.zobrist_hash ^= zobrist::piece_square_hash(self.color, Piece::Rook, rook_to);

            let mask = Bitboard::from_square(rook_from) ^ Bitboard::from_square(rook_to);
            unsafe {
                *position.piece_bitboard_mut(Piece::Rook) ^= mask;
                *position.color_bitboard_mut(self.color) ^= mask;
            }
        }

        self.color = !self.color;
        undo_state.zobrist_hash ^= zobrist::SIDE_HASH;
    }

    /// Undoes the last move without checking if there was a move to undo.
    ///
    /// ## Safety
    ///
    /// This game must have at least one move currently played.
    #[inline]
    pub unsafe fn undo_unchecked(&mut self) {
        // SAFETY: The user upheld that there is at least one move (at least two positions).
        // The position and undo vectors have equal lengths (see safety comment above).
        unsafe {
            self.positions.pop().unwrap_unchecked();
            self.undo_state.pop().unwrap_unchecked();
        }
        self.color = !self.color;
    }

    #[inline]
    #[must_use]
    pub(crate) fn legal_moves_raw(&self) -> MoveList {
        // SAFETY: This will always represent a valid chess position.
        unsafe {
            movegen::generate_all_legal_moves(
                self.position(),
                self.color,
                self.undo_state().castling_rights,
                self.undo_state().en_passant_square,
            )
        }
    }

    #[inline]
    #[must_use]
    pub(crate) fn is_legal_move_raw(&self, mv: Move) -> bool {
        self.legal_moves_raw().contains(&mv)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct UndoState {
    pub castling_rights: CastlingRights,
    pub en_passant_square: Option<Square>,
    pub zobrist_hash: u64,
    pub half_move_counter: usize,
}

/// Represents a checkmate or stalemate.
#[derive(Debug, Clone, Copy)]
pub enum MateStatus {
    Stalemate,
    Checkmate,
}

/// References a history of Zobrist hashes, with optimized implementations for various operations.
pub struct ZobristHashHistory<'game>(&'game Game);
impl ZobristHashHistory<'_> {
    /// Returns the historical hash `index` plies back, provided it hasn't crossed the last
    /// half-move reset.
    pub fn get_historical_after_reset(&self, index: usize) -> Option<u64> {
        let game = self.0;
        if index <= game.half_move_counter() {
            let index = game.ply_count().wrapping_sub(index);
            // SAFETY: `half_move_counter <= ply_count`, so `[original]index <= ply_count`
            // and thus `[new]index <= ply_count`. `undo_state.len() == ply_count + 1`,
            // so this would imply `index < undo_state.len()`.
            let undo_state = unsafe { game.undo_state.get_unchecked(index) };
            Some(undo_state.zobrist_hash)
        } else {
            None
        }
    }

    pub fn get(&self, index: usize) -> Option<u64> {
        (self.0)
            .undo_state
            .get(index)
            .map(|undo_state| undo_state.zobrist_hash)
    }
}
