use crate::lookup;
use crate::utils::*;

/// Inlined logic for each color. The movegenerator has an entirely separate function for each color
/// to guarantee maximum performance, which is why this happens. Inside a tranditional perft, this
/// branch will be taken in an alternating fashion, which is hopefully easier for branch prediction.
pub trait ColorLogic {
    type Opposite: ColorLogic;

    const COLOR: Color;

    const PAWN_FIRST_RANK: Bitboard;
    const PAWN_LAST_RANK: Bitboard;

    const KINGSIDE_CASTLE_FLAG: CastlingRights;
    const QUEENSIDE_CASTLE_FLAG: CastlingRights;
    const KINGSIDE_CASTLE_ROOK_FROM_SQUARE: Square;
    const QUEENSIDE_CASTLE_ROOK_FROM_SQUARE: Square;
    const KINGSIDE_CASTLE_ROOK_TO_SQUARE: Square;
    const QUEENSIDE_CASTLE_ROOK_TO_SQUARE: Square;

    const SHOULD_UNOCCUPIED_KINGSIDE_DURING_CASTLING: Bitboard;
    const SHOULD_UNOCCUPIED_QUEENSIDE_DURING_CASTLING: Bitboard;
    const SHOULD_NOT_ATTACKED_KINGSIDE_DURING_CASTLING: Bitboard;
    const SHOULD_NOT_ATTACKED_QUEENSIDE_DURING_CASTLING: Bitboard;

    const KINGSIDE_CASTLE_MOVE: Move;
    const QUEENSIDE_CASTLE_MOVE: Move;

    const FORWARD_SQUARE_OFFSET: i32;
    const DOUBLE_FORWARD_SQUARE_OFFSET: i32;

    fn forward(bitboard: Bitboard) -> Bitboard;
    fn double_forward(bitboard: Bitboard) -> Bitboard;

    #[inline(always)]
    fn right_pawn_attack(pawns: Bitboard) -> Bitboard {
        Self::forward(pawns & !Bitboard::FILE_H) << 1
    }
    #[inline(always)]
    fn left_pawn_attack(pawns: Bitboard) -> Bitboard {
        Self::forward(pawns & !Bitboard::FILE_A) >> 1
    }
}

pub struct White;
impl ColorLogic for White {
    type Opposite = Black;

    const COLOR: Color = Color::White;

    const PAWN_FIRST_RANK: Bitboard = Bitboard::RANK_2;
    const PAWN_LAST_RANK: Bitboard = Bitboard::RANK_7;

    const KINGSIDE_CASTLE_FLAG: CastlingRights = CastlingRights::WHITE_KINGSIDE;
    const QUEENSIDE_CASTLE_FLAG: CastlingRights = CastlingRights::WHITE_QUEENSIDE;
    const KINGSIDE_CASTLE_ROOK_FROM_SQUARE: Square = Square::H1;
    const QUEENSIDE_CASTLE_ROOK_FROM_SQUARE: Square = Square::A1;
    const KINGSIDE_CASTLE_ROOK_TO_SQUARE: Square = Square::F1;
    const QUEENSIDE_CASTLE_ROOK_TO_SQUARE: Square = Square::D1;

    const SHOULD_UNOCCUPIED_KINGSIDE_DURING_CASTLING: Bitboard = Bitboard(0x60);
    const SHOULD_UNOCCUPIED_QUEENSIDE_DURING_CASTLING: Bitboard = Bitboard(0xE);
    const SHOULD_NOT_ATTACKED_KINGSIDE_DURING_CASTLING: Bitboard = Bitboard(0x70);
    const SHOULD_NOT_ATTACKED_QUEENSIDE_DURING_CASTLING: Bitboard = Bitboard(0x1C);

    const KINGSIDE_CASTLE_MOVE: Move = Move::new(Square::E1, Square::G1, MoveFlags::KingCastle);
    const QUEENSIDE_CASTLE_MOVE: Move = Move::new(Square::E1, Square::C1, MoveFlags::QueenCastle);

    const FORWARD_SQUARE_OFFSET: i32 = 8;
    const DOUBLE_FORWARD_SQUARE_OFFSET: i32 = 16;

    #[inline(always)]
    fn forward(bitboard: Bitboard) -> Bitboard {
        bitboard << 8
    }
    #[inline(always)]
    fn double_forward(bitboard: Bitboard) -> Bitboard {
        bitboard << 16
    }
}

pub struct Black;
impl ColorLogic for Black {
    type Opposite = White;

    const COLOR: Color = Color::Black;

    const PAWN_FIRST_RANK: Bitboard = Bitboard::RANK_7;
    const PAWN_LAST_RANK: Bitboard = Bitboard::RANK_2;

    const KINGSIDE_CASTLE_FLAG: CastlingRights = CastlingRights::BLACK_KINGSIDE;
    const QUEENSIDE_CASTLE_FLAG: CastlingRights = CastlingRights::BLACK_QUEENSIDE;
    const KINGSIDE_CASTLE_ROOK_FROM_SQUARE: Square = Square::H8;
    const QUEENSIDE_CASTLE_ROOK_FROM_SQUARE: Square = Square::A8;
    const KINGSIDE_CASTLE_ROOK_TO_SQUARE: Square = Square::F8;
    const QUEENSIDE_CASTLE_ROOK_TO_SQUARE: Square = Square::D8;

    const SHOULD_UNOCCUPIED_KINGSIDE_DURING_CASTLING: Bitboard = Bitboard(0x6000000000000000);
    const SHOULD_UNOCCUPIED_QUEENSIDE_DURING_CASTLING: Bitboard = Bitboard(0xE00000000000000);
    const SHOULD_NOT_ATTACKED_KINGSIDE_DURING_CASTLING: Bitboard = Bitboard(0x7000000000000000);
    const SHOULD_NOT_ATTACKED_QUEENSIDE_DURING_CASTLING: Bitboard = Bitboard(0x1C00000000000000);

    const KINGSIDE_CASTLE_MOVE: Move = Move::new(Square::E8, Square::G8, MoveFlags::KingCastle);
    const QUEENSIDE_CASTLE_MOVE: Move = Move::new(Square::E8, Square::C8, MoveFlags::QueenCastle);

    const FORWARD_SQUARE_OFFSET: i32 = -8;
    const DOUBLE_FORWARD_SQUARE_OFFSET: i32 = -16;

    #[inline(always)]
    fn forward(bitboard: Bitboard) -> Bitboard {
        bitboard >> 8
    }
    #[inline(always)]
    fn double_forward(bitboard: Bitboard) -> Bitboard {
        bitboard >> 16
    }
}

#[inline(always)]
fn quiet_or_capture(occupancy: Bitboard, to: Square) -> MoveFlags {
    if occupancy.is_set_at(to) {
        MoveFlags::Capture
    } else {
        MoveFlags::Quiet
    }
}

unsafe fn compute_attacked_without_king<Color: ColorLogic>(position: &Position) -> Bitboard {
    let king = position.bitboard(Piece::King, Color::COLOR);
    let mut banned = Bitboard(0);

    // Calculate attack from enemy pawns.
    let pl = Color::Opposite::left_pawn_attack(position.bitboard(Piece::Pawn, !Color::COLOR));
    let pr = Color::Opposite::right_pawn_attack(position.bitboard(Piece::Pawn, !Color::COLOR));
    banned |= pl | pr;

    // Calculate attack from enemy king.
    banned |= lookup::king_attack(unsafe {
        position
            .bitboard(Piece::King, !Color::COLOR)
            .next()
            .unwrap_unchecked()
    });

    // Calculate attack from enemy knights.
    for from in position.bitboard(Piece::Knight, !Color::COLOR) {
        banned |= lookup::knight_attack(from);
    }

    // Calculate attack from enemy bishops.
    for from in position.bitboard(Piece::Bishop, !Color::COLOR)
        | position.bitboard(Piece::Queen, !Color::COLOR)
    {
        banned |= lookup::bishop_attack(from, position.occupied() ^ king);
    }

    // Calculate attack from enemy rooks.
    for from in position.bitboard(Piece::Rook, !Color::COLOR)
        | position.bitboard(Piece::Queen, !Color::COLOR)
    {
        banned |= lookup::rook_attack(from, position.occupied() ^ king);
    }

    banned
}

/// Compute the checkmask, orthogonal-pinmask, and diagonal-pinmask all in one go. These are
/// returned in that order.
unsafe fn compute_checkmask_and_pinmasks<Color: ColorLogic>(
    king_square: Square,
    position: &Position,
) -> (Bitboard, Bitboard, Bitboard) {
    let enemy_pawns = position.bitboard(Piece::Pawn, !Color::COLOR);
    let enemy_knights = position.bitboard(Piece::Knight, !Color::COLOR);
    let enemy_queens = position.bitboard(Piece::Queen, !Color::COLOR);
    let enemy_bishops = position.bitboard(Piece::Bishop, !Color::COLOR) | enemy_queens;
    let enemy_rooks = position.bitboard(Piece::Rook, !Color::COLOR) | enemy_queens;
    let king = position.bitboard(Piece::King, Color::COLOR);
    let occupied = position.occupied();

    let mut checkmask = Bitboard::FULL;
    let mut orthogonal_pinmask = Bitboard::EMPTY;
    let mut diagonal_pinmask = Bitboard::EMPTY;

    // Compute the orthogonal pinmask, but enrich the checkmask in the process.
    // This is done by sending a ray from the king, but only considering the enemy rooks.
    // For any hit we check the number of pieces in between: 0 = check, 1 = pin.
    let ray = lookup::rook_attack(king_square, enemy_rooks);
    for square in ray & enemy_rooks {
        // This will represent the squares in **between**: not including the king, but including
        // the piece (potential orthogonal checker or pinner) we are interested in.
        let between_including_target =
            (lookup::rook_attack(square, king) & ray) | Bitboard::from_square(square);
        // This bitboard will include the target piece, so: 1 = check, 2 = pin.
        match (between_including_target & occupied).count_ones() {
            1 => checkmask &= between_including_target,
            2 => orthogonal_pinmask |= between_including_target,
            _ => {}
        }
    }

    // Compute the diagonal pinmask. This follows about the same logic.
    let ray = lookup::bishop_attack(king_square, enemy_bishops);
    for square in ray & enemy_bishops {
        let between_including_target =
            (lookup::bishop_attack(square, king) & ray) | Bitboard::from_square(square);
        match (between_including_target & occupied).count_ones() {
            1 => checkmask &= between_including_target,
            2 => diagonal_pinmask |= between_including_target,
            _ => {}
        }
    }

    // Knights and pawns can check the king too, so we use a simple branchless implementation.
    // Either zero of either, one knight, or one pawn can check in a perfectly legal position.
    // However, manufactured positions can have more, at which case we handle it specially with
    // an extremely cold path.
    let knight_attack = lookup::knight_attack(king_square);
    let pawn_attack = Color::left_pawn_attack(king) | Color::right_pawn_attack(king);
    let checker = (knight_attack & enemy_knights) | (pawn_attack & enemy_pawns);

    assert!(checker.0 & (checker.0 - 1) == 0, "yes I'm lazy");
    checkmask &=
        std::hint::select_unpredictable(checker != Bitboard::EMPTY, checker, Bitboard::FULL);

    (checkmask, orthogonal_pinmask, diagonal_pinmask)
}

#[inline(never)]
unsafe fn generate_all_legal_moves_impl<Color: ColorLogic>(
    moves: &mut MoveList,
    position: &Position,
    castling_rights: CastlingRights,
    en_passant_square: Option<Square>,
) {
    // Obtain all pieces.
    let pawns = position.bitboard(Piece::Pawn, Color::COLOR);
    let knights = position.bitboard(Piece::Knight, Color::COLOR);
    let bishops = position.bitboard(Piece::Bishop, Color::COLOR);
    let rooks = position.bitboard(Piece::Rook, Color::COLOR);
    let queens = position.bitboard(Piece::Queen, Color::COLOR);
    let king = position.bitboard(Piece::King, Color::COLOR);
    let self_occupancy = position.color_bitboard(Color::COLOR);
    let other_occupancy = position.color_bitboard(!Color::COLOR);
    let occupied = position.occupied();

    let king_square = unsafe { king.clone().next().unwrap_unchecked() };
    let banned = unsafe { compute_attacked_without_king::<Color>(position) };
    let (checkmask, pin_hv, pin_d) =
        unsafe { compute_checkmask_and_pinmasks::<Color>(king_square, position) };
    let moveable = !self_occupancy & checkmask;

    // Generate legal pawn moves.
    {
        let pawns_uhv = pawns & !pin_hv;
        let pawns_ud = pawns & !pin_d;

        // Calculate the naive bitboards for the four possible normal pawn moves:
        // quiet push, double push, left capture, right capture.
        let mut quiet = pawns_ud & Color::Opposite::forward(!occupied);
        let mut double =
            quiet & Color::PAWN_FIRST_RANK & Color::Opposite::double_forward(!occupied & checkmask);
        let mut left_capture =
            pawns_uhv & Color::Opposite::right_pawn_attack(other_occupancy & checkmask);
        let mut right_capture =
            pawns_uhv & Color::Opposite::left_pawn_attack(other_occupancy & checkmask);
        quiet &= Color::Opposite::forward(checkmask);

        // Pruning quiet push moves:
        //   We already pruned all [diagonal]-pinned pawns, so we can split pawns into [orthogonal]-
        //   pinned and unpinned. [orthogonal]-pinned pawns can only perform a quiet pawn move if the
        //   resulting square is still on the [orthogonal]-pinmask.
        quiet &= Color::Opposite::forward(pin_hv) | pawns_uhv;

        // Pruning double push moves:
        //   Same logic as quiet push moves.
        double &= Color::Opposite::double_forward(pin_hv) | pawns_uhv;

        // Pruning left captures:
        //   We have already pruned all [orthogonal]-pinned left captures. This means that left
        //   captures can be split into [diagonal]-pinned and unpinned. Such pinned captures can
        //   only happen when the resulting square is in the [diagonal]-pinmask.
        left_capture &= Color::Opposite::right_pawn_attack(pin_d) | pawns_ud;

        // Pruning right captures:
        //   Same logic as left captures.
        right_capture &= Color::Opposite::left_pawn_attack(pin_d) | pawns_ud;

        // Branch on if pawns are on last rank or not. There's a lot of extreme optimizations
        // being done at the pawns level because there's normally a lot of pawns on the board.
        if !(pawns & Color::PAWN_LAST_RANK).is_empty() {
            std::hint::cold_path();

            let quiet_promotion = quiet & Color::PAWN_LAST_RANK;
            let left_capture_promotion = left_capture & Color::PAWN_LAST_RANK;
            let right_capture_promotion = right_capture & Color::PAWN_LAST_RANK;
            quiet &= !Color::PAWN_LAST_RANK;
            left_capture &= !Color::PAWN_LAST_RANK;
            right_capture &= !Color::PAWN_LAST_RANK;

            unsafe {
                for from in quiet_promotion {
                    let to = from.offset_by_unchecked(Color::FORWARD_SQUARE_OFFSET);
                    moves.push_unchecked(Move::new(from, to, MoveFlags::QueenPromotion));
                    moves.push_unchecked(Move::new(from, to, MoveFlags::RookPromotion));
                    moves.push_unchecked(Move::new(from, to, MoveFlags::KnightPromotion));
                    moves.push_unchecked(Move::new(from, to, MoveFlags::BishopPromotion));
                }
                for from in left_capture_promotion {
                    let to = from.offset_by_unchecked(Color::FORWARD_SQUARE_OFFSET - 1);
                    moves.push_unchecked(Move::new(from, to, MoveFlags::QueenPromotionCapture));
                    moves.push_unchecked(Move::new(from, to, MoveFlags::RookPromotionCapture));
                    moves.push_unchecked(Move::new(from, to, MoveFlags::KnightPromotionCapture));
                    moves.push_unchecked(Move::new(from, to, MoveFlags::BishopPromotionCapture));
                }
                for from in right_capture_promotion {
                    let to = from.offset_by_unchecked(Color::FORWARD_SQUARE_OFFSET + 1);
                    moves.push_unchecked(Move::new(from, to, MoveFlags::QueenPromotionCapture));
                    moves.push_unchecked(Move::new(from, to, MoveFlags::RookPromotionCapture));
                    moves.push_unchecked(Move::new(from, to, MoveFlags::KnightPromotionCapture));
                    moves.push_unchecked(Move::new(from, to, MoveFlags::BishopPromotionCapture));
                }
            }
        }

        unsafe {
            for from in quiet {
                let to = from.offset_by_unchecked(Color::FORWARD_SQUARE_OFFSET);
                moves.push_unchecked(Move::new(from, to, MoveFlags::Quiet));
            }
            for from in double {
                let to = from.offset_by_unchecked(Color::DOUBLE_FORWARD_SQUARE_OFFSET);
                moves.push_unchecked(Move::new(from, to, MoveFlags::DoublePawnPush));
            }
            for from in left_capture {
                let to = from.offset_by_unchecked(Color::FORWARD_SQUARE_OFFSET - 1);
                moves.push_unchecked(Move::new(from, to, MoveFlags::Capture));
            }
            for from in right_capture {
                let to = from.offset_by_unchecked(Color::FORWARD_SQUARE_OFFSET + 1);
                moves.push_unchecked(Move::new(from, to, MoveFlags::Capture));
            }
        }

        if let Some(ep_square) = en_passant_square {
            let ep_spot = Bitboard::from_square(ep_square);
            let ep_target = Color::Opposite::forward(ep_spot) & checkmask;

            let mut left_ep = pawns_uhv & !Bitboard::FILE_A & (ep_target << 1);
            let mut right_ep = pawns_uhv & !Bitboard::FILE_H & (ep_target >> 1);

            if !(left_ep | right_ep).is_empty()
                && ((left_ep | right_ep).0.count_ones() == 2
                    || (lookup::rook_attack(
                        king_square,
                        occupied ^ (left_ep | right_ep | ep_spot | ep_target),
                    ) & (position.bitboard(Piece::Rook, !Color::COLOR)
                        | position.bitboard(Piece::Queen, !Color::COLOR)))
                    .is_empty())
            {
                left_ep &= Color::Opposite::right_pawn_attack(pin_d) | !pin_d;
                right_ep &= Color::Opposite::left_pawn_attack(pin_d) | !pin_d;

                if let Some(left_ep_square) = left_ep.next() {
                    let mv = Move::new(left_ep_square, ep_square, MoveFlags::EnPassantCapture);
                    unsafe { moves.push_unchecked(mv) };
                }
                if let Some(right_ep_square) = right_ep.next() {
                    let mv = Move::new(right_ep_square, ep_square, MoveFlags::EnPassantCapture);
                    unsafe { moves.push_unchecked(mv) };
                }
            }
        }
    }

    // Generate legal knight moves.
    {
        let unpinned_knights = knights & !(pin_hv | pin_d);
        for from in unpinned_knights {
            let legal = lookup::knight_attack(from) & moveable;
            for to in legal {
                let mv = Move::new(from, to, quiet_or_capture(other_occupancy, to));
                unsafe { moves.push_unchecked(mv) };
            }
        }
    }

    // Generate legal bishop moves (queens included).
    {
        let bishops_queens = (bishops | queens) & !pin_hv;
        let unpinned_bishops = bishops_queens & !pin_d;
        let pinned_bishops = bishops_queens & pin_d & checkmask;
        for from in unpinned_bishops {
            let legal = lookup::bishop_attack(from, occupied) & moveable;
            for to in legal {
                let mv = Move::new(from, to, quiet_or_capture(other_occupancy, to));
                unsafe { moves.push_unchecked(mv) };
            }
        }
        for from in pinned_bishops {
            let legal = lookup::bishop_attack(from, occupied) & moveable & pin_d;
            for to in legal {
                let mv = Move::new(from, to, quiet_or_capture(other_occupancy, to));
                unsafe { moves.push_unchecked(mv) };
            }
        }
    }

    // Generate legal rook moves (queens included).
    {
        let rooks_queens = (rooks | queens) & !pin_d;
        let unpinned_rooks = rooks_queens & !pin_hv;
        let pinned_rooks = rooks_queens & pin_hv & checkmask;
        for from in unpinned_rooks {
            let legal = lookup::rook_attack(from, occupied) & moveable;
            for to in legal {
                let mv = Move::new(from, to, quiet_or_capture(other_occupancy, to));
                unsafe { moves.push_unchecked(mv) };
            }
        }
        for from in pinned_rooks {
            let legal = lookup::rook_attack(from, occupied) & moveable & pin_hv;
            for to in legal {
                let mv = Move::new(from, to, quiet_or_capture(other_occupancy, to));
                unsafe { moves.push_unchecked(mv) };
            }
        }
    }

    // Generate legal king moves.
    {
        let king_moves = lookup::king_attack(king_square) & !(banned | self_occupancy);
        for to in king_moves {
            let mv = Move::new(king_square, to, quiet_or_capture(other_occupancy, to));
            unsafe { moves.push_unchecked(mv) };
        }

        if castling_rights.has_any(Color::KINGSIDE_CASTLE_FLAG | Color::QUEENSIDE_CASTLE_FLAG) {
            if (Color::SHOULD_UNOCCUPIED_KINGSIDE_DURING_CASTLING & occupied).is_empty()
                && (Color::SHOULD_NOT_ATTACKED_KINGSIDE_DURING_CASTLING & banned).is_empty()
                && castling_rights.has_any(Color::KINGSIDE_CASTLE_FLAG)
            {
                unsafe { moves.push_unchecked(Color::KINGSIDE_CASTLE_MOVE) };
            }

            if (Color::SHOULD_UNOCCUPIED_QUEENSIDE_DURING_CASTLING & occupied).is_empty()
                && (Color::SHOULD_NOT_ATTACKED_QUEENSIDE_DURING_CASTLING & banned).is_empty()
                && castling_rights.has_any(Color::QUEENSIDE_CASTLE_FLAG)
            {
                unsafe { moves.push_unchecked(Color::QUEENSIDE_CASTLE_MOVE) };
            }
        }
    }
}

/// Generates all legal moves from the provided arguments.
///
/// ## Safety
///
/// The provided arguments must detail a fully valid chess position.
#[inline(always)]
pub unsafe fn generate_all_legal_moves(
    position: &Position,
    color: Color,
    castling_rights: CastlingRights,
    en_passant_square: Option<Square>,
) -> MoveList {
    // This function is marked `#[inline(always)]` while the implementation functions are marked
    // `#[inline(never)]` so that the outer handling code gets to work with the top-level `match`
    // on `color`. This can be optimized or hoisted by LLVM.
    //
    // These are the before/after changes for certain perfts in million nodes per second:
    //     Start 7: 336 -> 366
    //  Kiwipete 6: 390 -> 417
    //    Tricky 8: 248 -> 280
    //   Complex 6: 381 -> 409
    //     Buggy 6: 350 -> 385
    let mut moves = MoveList::new();

    // SAFETY: The helper function has the same safety guarantee.
    unsafe {
        match color {
            Color::White => generate_all_legal_moves_impl::<White>(
                &mut moves,
                position,
                castling_rights,
                en_passant_square,
            ),
            Color::Black => generate_all_legal_moves_impl::<Black>(
                &mut moves,
                position,
                castling_rights,
                en_passant_square,
            ),
        }
    }

    moves
}

/// Determines all attackers of a given square, where the given color is the one attacking.
///
/// ## Safety
///
/// The provided arguments must detail a fully valid chess position.
pub unsafe fn square_attackers(position: &Position, color: Color, square: Square) -> Bitboard {
    let mask = Bitboard::from_square(square);
    let occupied = position.occupied();

    let pawns = position.piece_bitboard(Piece::Pawn);
    let knights = position.piece_bitboard(Piece::Knight);
    let bishops = position.piece_bitboard(Piece::Bishop);
    let rooks = position.piece_bitboard(Piece::Rook);
    let queens = position.piece_bitboard(Piece::Queen);
    let kings = position.piece_bitboard(Piece::King);

    let white_pawn_attack = White::left_pawn_attack(mask) | White::right_pawn_attack(mask);
    let black_pawn_attack = Black::left_pawn_attack(mask) | Black::right_pawn_attack(mask);
    let pawn_attack = std::hint::select_unpredictable(
        color == Color::White,
        black_pawn_attack,
        white_pawn_attack,
    );

    position.color_bitboard(color)
        & ((pawn_attack & pawns)
            | (lookup::king_attack(square) & kings)
            | (lookup::knight_attack(square) & knights)
            | (lookup::bishop_attack(square, occupied) & (bishops | queens))
            | (lookup::rook_attack(square, occupied) & (rooks | queens)))
}
