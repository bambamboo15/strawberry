use crate::game::*;
use crate::utils::*;

pub struct PieceDisplay(Piece);
pub struct ColoredPieceDisplay(Piece, Color);

impl Piece {
    /// Returns an object that implements [Display] using UCI rules (always lowercase).
    ///
    /// [Display]: std::fmt::Display
    #[inline]
    #[must_use]
    pub fn display(self) -> PieceDisplay {
        PieceDisplay(self)
    }

    /// Returns an object that implements [Display] using FEN/SAN rules
    /// (uppercase for white, lowercase for black).
    ///
    /// [Display]: std::fmt::Display
    #[inline]
    #[must_use]
    pub fn display_colored(self, color: Color) -> ColoredPieceDisplay {
        ColoredPieceDisplay(self, color)
    }
}

impl std::fmt::Display for PieceDisplay {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let char = match self.0 {
            Piece::Pawn => 'p',
            Piece::Knight => 'n',
            Piece::Bishop => 'b',
            Piece::Rook => 'r',
            Piece::Queen => 'q',
            Piece::King => 'k',
        };
        write!(f, "{}", char)
    }
}

impl std::fmt::Display for ColoredPieceDisplay {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let char = match (self.0, self.1) {
            (Piece::Pawn, Color::White) => 'P',
            (Piece::Knight, Color::White) => 'N',
            (Piece::Bishop, Color::White) => 'B',
            (Piece::Rook, Color::White) => 'R',
            (Piece::Queen, Color::White) => 'Q',
            (Piece::King, Color::White) => 'K',
            (Piece::Pawn, Color::Black) => 'p',
            (Piece::Knight, Color::Black) => 'n',
            (Piece::Bishop, Color::Black) => 'b',
            (Piece::Rook, Color::Black) => 'r',
            (Piece::Queen, Color::Black) => 'q',
            (Piece::King, Color::Black) => 'k',
        };
        write!(f, "{}", char)
    }
}

pub struct LowercaseSquareDisplay(Square);
pub struct UppercaseSquareDisplay(Square);

impl Square {
    /// Produces a [`Square`] from a lowercase string (example: `b7`).
    #[inline]
    #[must_use]
    pub fn from_lowercase_str(str: &str) -> Option<Self> {
        match str.as_bytes() {
            [letter @ b'a'..=b'h', number @ b'1'..=b'8'] => {
                Self::from_file_and_rank((letter - b'a') as usize, (number - b'1') as usize)
            }
            _ => None,
        }
    }

    /// Produces a [`Square`] from an uppercase string (example: `B7`).
    #[inline]
    #[must_use]
    pub fn from_uppercase_str(str: &str) -> Option<Self> {
        match str.as_bytes() {
            [letter @ b'A'..=b'h', number @ b'1'..=b'8'] => {
                Self::from_file_and_rank((letter - b'A') as usize, (number - b'1') as usize)
            }
            _ => None,
        }
    }

    /// Returns an object that implements [Display] for lowercase (example: `a5`).
    ///
    /// [Display]: std::fmt::Display
    #[inline]
    #[must_use]
    pub fn display_lowercase(self) -> LowercaseSquareDisplay {
        LowercaseSquareDisplay(self)
    }

    /// Returns an object that implements [Display] for uppercase (example: `A5`).
    ///
    /// [Display]: std::fmt::Display
    #[inline]
    #[must_use]
    pub fn display_uppercase(self) -> UppercaseSquareDisplay {
        UppercaseSquareDisplay(self)
    }
}

impl std::fmt::Display for LowercaseSquareDisplay {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            (self.0.file() as u8 + b'a') as char,
            (self.0.rank() as u8 + b'1') as char
        )
    }
}

impl std::fmt::Display for UppercaseSquareDisplay {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            (self.0.file() as u8 + b'A') as char,
            (self.0.rank() as u8 + b'1') as char
        )
    }
}

pub struct BitboardDebugDisplay(Bitboard);

impl Bitboard {
    /// Returns an object that implements [Display] that shows a debug ANSI representation of set
    /// bits.
    ///
    /// [Display]: std::fmt::Display
    #[inline]
    #[must_use]
    pub fn display_debug(self) -> BitboardDebugDisplay {
        BitboardDebugDisplay(self)
    }
}

impl std::fmt::Display for BitboardDebugDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const RESET: &str = "\x1b[0m";
        const LIGHT_BROWN: &str = "\x1b[48;2;240;217;181m";
        const DARK_BROWN: &str = "\x1b[48;2;181;136;99m";
        const RED_PIECE: &str = "\x1b[48;2;230;50;50m";

        for rank in (0..8).rev() {
            for file in 0..8 {
                let square = rank * 8 + file;
                let is_set = (self.0.0 >> square) & 1 == 1;

                let bg_color = if is_set {
                    RED_PIECE
                } else if (rank + file) % 2 == 1 {
                    LIGHT_BROWN
                } else {
                    DARK_BROWN
                };

                write!(f, "{}  {}", bg_color, RESET)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

/// Simple readable move data, similar to how a user moves a piece in online chess.
pub struct MoveIntent {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<Piece>,
}

impl MoveIntent {
    /// Constructs a [`MoveIntent`] from the source and destination squares (no promotion).
    #[inline]
    #[must_use]
    pub fn from_squares(from: Square, to: Square) -> Self {
        MoveIntent {
            from,
            to,
            promotion: None,
        }
    }

    /// Constructs a [`MoveIntent`] from the start and destination squares and a promotion piece.
    #[inline]
    #[must_use]
    pub fn from_squares_and_promotion(from: Square, to: Square, promotion: Piece) -> Self {
        MoveIntent {
            from,
            to,
            promotion: Some(promotion),
        }
    }
}

impl std::str::FromStr for MoveIntent {
    type Err = ();

    /// Converts a UCI-style move string (example: `d7d8q`) to a [`MoveIntent`].
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn inner(s: &str) -> Option<MoveIntent> {
            Some(MoveIntent {
                from: Square::from_lowercase_str(s.get(0..2)?)?,
                to: Square::from_lowercase_str(s.get(2..4)?)?,
                promotion: match s.get(4..)? {
                    "" => None,
                    "p" => Some(Piece::Pawn),
                    "n" => Some(Piece::Knight),
                    "b" => Some(Piece::Bishop),
                    "r" => Some(Piece::Rook),
                    "q" => Some(Piece::Queen),
                    "k" => Some(Piece::King),
                    _ => return None,
                },
            })
        }
        inner(s).ok_or(())
    }
}

impl std::fmt::Display for MoveIntent {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            self.from.display_lowercase(),
            self.to.display_lowercase(),
        )?;
        if let Some(promotion_piece) = self.promotion {
            write!(f, "{}", promotion_piece.display())?;
        }
        Ok(())
    }
}

/// Standard Algebraic Notation (SAN) form of a chess move.
pub enum SanMove {
    Normal {
        piece: Piece,
        to: Square,
        capture: bool,
        from_file: Option<u8>,
        from_rank: Option<u8>,
    },
    CastleKingSide,
    CastleQueenSide,
}

impl std::fmt::Display for SanMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal {
                piece,
                to,
                capture,
                from_file,
                from_rank,
            } => {
                struct OptionalChar(Option<char>);
                impl std::fmt::Display for OptionalChar {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        match self.0 {
                            Some(char) => write!(f, "{}", char),
                            None => Ok(()),
                        }
                    }
                }

                let piece = OptionalChar(match piece {
                    Piece::Pawn => None,
                    Piece::Knight => Some('N'),
                    Piece::Bishop => Some('B'),
                    Piece::Rook => Some('R'),
                    Piece::Queen => Some('Q'),
                    Piece::King => Some('K'),
                });
                let from_file = OptionalChar(match from_file {
                    Some(file) => Some((file + b'a') as char),
                    None => None,
                });
                let from_rank = OptionalChar(match from_rank {
                    Some(rank) => Some((rank + b'1') as char),
                    None => None,
                });
                let capture = OptionalChar(match capture {
                    true => Some('x'),
                    false => None,
                });
                let to = to.display_lowercase();

                write!(f, "{piece}{from_file}{from_rank}{capture}{to}")
            }
            Self::CastleKingSide => write!(f, "O-O"),
            Self::CastleQueenSide => write!(f, "O-O-O"),
        }
    }
}

impl Game {
    /// Constructs a new [`Game`] from the starting position.
    #[inline]
    #[must_use]
    pub fn start_position() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    /// Constructs a new [`Game`] from a FEN string.
    ///
    /// ### Leniencies
    ///
    /// Any amount of whitespace between or around the fields is allowed.
    ///
    /// The last one, two, three, four, five, and six fields can be omitted, and will be replaced
    /// with defaults form the starting position FEN.
    pub fn from_fen(fen: &str) -> Option<Self> {
        let mut chunks = fen.split_whitespace();

        let default_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
        let position = Position::from_fen(chunks.next().unwrap_or(default_fen))?;
        let color = match chunks.next()? {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return None,
        };
        let castling_rights = CastlingRights::from_value(match chunks.next().unwrap_or("KQkq") {
            "-" => 0b0000,
            "K" => 0b0001,
            "Q" => 0b0010,
            "KQ" => 0b0011,
            "k" => 0b0100,
            "Kk" => 0b0101,
            "Qk" => 0b0110,
            "QKk" => 0b0111,
            "q" => 0b1000,
            "Kq" => 0b1001,
            "Qq" => 0b1010,
            "KQq" => 0b1011,
            "kq" => 0b1100,
            "Kkq" => 0b1101,
            "Qkq" => 0b1110,
            "KQkq" => 0b1111,
            _ => return None,
        })?;
        let en_passant_square = match chunks.next().unwrap_or("-") {
            "-" => None,
            str => Some(Square::from_lowercase_str(str)?),
        };

        // TODO: Move counts.
        let _ = chunks.next().unwrap_or("0");
        let _ = chunks.next().unwrap_or("1");
        if chunks.next().is_some() {
            return None;
        }

        Some(Game::from_raw(
            position,
            color,
            castling_rights,
            en_passant_square,
            0, // TODO
        )?)
    }

    /// Converts [`MoveIntent`] (simple move data) to a [`Move`] (encoded move data) if legal.
    pub fn intent_to_move(&self, intent: MoveIntent) -> Option<Move> {
        self.legal_moves_raw()
            .iter()
            .find(|mv| {
                (mv.from() == intent.from && mv.to() == intent.to)
                    && mv.flags().promotion_piece() == intent.promotion
            })
            .copied()
    }

    /// Converts [`Move`] (encoded move data) to [`MoveIntent`] (simple move data) if legal.
    pub fn move_to_intent(&self, mv: Move) -> Option<MoveIntent> {
        if self.is_legal_move_raw(mv) {
            Some(MoveIntent {
                from: mv.from(),
                to: mv.to(),
                promotion: mv.flags().promotion_piece(),
            })
        } else {
            None
        }
    }
}

impl Position {
    /// Constructs a new [`Position`] from the first part of a FEN string.
    ///
    /// This method will reject strings that contain any space character.
    pub fn from_fen(fen: &str) -> Option<Self> {
        let mut position = Position::from_raw([Bitboard::EMPTY; 6], [Bitboard::EMPTY; 2])?;

        let mut row = 7;
        let mut col = 0;
        for &byte in fen.as_bytes() {
            if col == 8 {
                if byte != b'/' || row == 0 {
                    return None;
                }
                col = 0;
                row -= 1;
                continue;
            }

            if matches!(byte, b'1'..=b'8') {
                col += (byte - b'0') as usize;
                if col > 8 {
                    return None;
                }
                continue;
            }

            let (piece, color) = match byte {
                b'P' => (Piece::Pawn, Color::White),
                b'N' => (Piece::Knight, Color::White),
                b'B' => (Piece::Bishop, Color::White),
                b'R' => (Piece::Rook, Color::White),
                b'Q' => (Piece::Queen, Color::White),
                b'K' => (Piece::King, Color::White),
                b'p' => (Piece::Pawn, Color::Black),
                b'n' => (Piece::Knight, Color::Black),
                b'b' => (Piece::Bishop, Color::Black),
                b'r' => (Piece::Rook, Color::Black),
                b'q' => (Piece::Queen, Color::Black),
                b'k' => (Piece::King, Color::Black),
                _ => return None,
            };

            let square = Square::from_file_and_rank(col, row)?;
            position.add_piece_at(square, piece, color);
            col += 1;
        }

        if row != 0 || col != 8 {
            return None;
        }

        Some(position)
    }
}
