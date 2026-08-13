use crate::game::*;
use crate::utils::*;
use std::cmp::Ordering;
use std::ops::IndexMut;
use std::slice::SliceIndex;

pub struct Shared;
pub struct Exclusive;

pub trait Mutability {
    type Borrow<'a, T: 'a>;

    // These functions were NOT written by a clanker
    // I needed these because I wanted to abstract over mutability
    // took me an extremely long 30 minutes of effort :(
    // that's a lot of YouTube shorts
    fn as_shared<'a: 'b, 'b, T>(borrow: &'b Self::Borrow<'a, T>) -> &'b T;
    fn as_reborrowed<'a: 'b, 'b, T>(borrow: &'b mut Self::Borrow<'a, T>) -> Self::Borrow<'b, T>;
}

impl Mutability for Shared {
    type Borrow<'a, T: 'a> = &'a T;

    fn as_shared<'a: 'b, 'b, T>(borrow: &'b Self::Borrow<'a, T>) -> &'b T {
        borrow
    }

    fn as_reborrowed<'a: 'b, 'b, T>(borrow: &'b mut Self::Borrow<'a, T>) -> Self::Borrow<'b, T> {
        borrow
    }
}

impl Mutability for Exclusive {
    type Borrow<'a, T: 'a> = &'a mut T;

    fn as_shared<'a: 'b, 'b, T>(borrow: &'b Self::Borrow<'a, T>) -> &'b T {
        borrow
    }

    fn as_reborrowed<'a: 'b, 'b, T>(borrow: &'b mut Self::Borrow<'a, T>) -> Self::Borrow<'b, T> {
        borrow
    }
}

impl Game {
    /// Constructs a [`SearchZero`] that borrows this immutably.
    #[inline]
    #[must_use]
    pub fn search(&self) -> SearchZero<'_, Shared> {
        SearchZero { game: self }
    }

    /// Constructs a [`SearchZero`] that borrows this mutably.
    #[inline]
    #[must_use]
    pub fn search_mut(&mut self) -> SearchZero<'_, Exclusive> {
        SearchZero { game: self }
    }

    /// Construct a [`SearchOne`] borrowing this immutably for a move if it is legal.
    #[inline]
    #[must_use]
    pub fn get_legal_move(&self, mv: Move) -> Option<SearchOne<'_, Shared>> {
        Self::get_legal_move_impl(self, mv)
    }

    /// Construct a [`SearchOne`] borrowing this mutably for a move if it is legal.
    #[inline]
    #[must_use]
    pub fn get_legal_move_mut(&mut self, mv: Move) -> Option<SearchOne<'_, Exclusive>> {
        Self::get_legal_move_impl(self, mv)
    }

    /// Construct a [`SearchMany`] borrowing this immutably that contains all legal moves.
    #[inline]
    #[must_use]
    pub fn legal_moves(&self) -> SearchMany<'_, MoveList, Shared> {
        Self::legal_moves_impl(self)
    }

    /// Construct a [`SearchMany`] borrowing this mutably that contains all legal moves.
    #[inline]
    #[must_use]
    pub fn legal_moves_mut(&mut self) -> SearchMany<'_, MoveList, Exclusive> {
        Self::legal_moves_impl(self)
    }

    fn get_legal_move_impl<'a, M>(game: M::Borrow<'a, Game>, mv: Move) -> Option<SearchOne<'a, M>>
    where
        M: Mutability,
    {
        if M::as_shared(&game).is_legal_move_raw(mv) {
            Some(SearchOne { game, mv })
        } else {
            None
        }
    }

    fn legal_moves_impl<'a, M>(game: M::Borrow<'a, Game>) -> SearchMany<'a, MoveList, M>
    where
        M: Mutability,
    {
        SearchMany {
            moves: M::as_shared(&game).legal_moves_raw(),
            game,
        }
    }
}

/// Search object with no stored moves.
/// 
/// This can be promoted to [`SearchOne`] or [`SearchMany`], which contain moves absolutely
/// guaranteed legal for the current position. This allows operations to be done on the moves
/// safely without double-checking their legality.
pub struct SearchZero<'game, M>
where
    M: Mutability,
{
    game: M::Borrow<'game, Game>,
}

impl<'game, M> SearchZero<'game, M>
where
    M: Mutability,
{
    /// Returns an immutable reference to the game.
    #[inline]
    #[must_use]
    pub fn game(&self) -> &Game {
        M::as_shared(&self.game)
    }

    /// Constructs a [`SearchOne`] with the provided move if it is legal.
    #[inline]
    #[must_use]
    pub fn get_legal_move(&mut self, mv: Move) -> Option<SearchOne<'_, M>> {
        Game::get_legal_move_impl(M::as_reborrowed(&mut self.game), mv)
    }

    /// Constructs a [`SearchMany`] with all legal moves for the current position.
    #[inline]
    #[must_use]
    pub fn legal_moves(&mut self) -> SearchMany<'_, MoveList, M> {
        Game::legal_moves_impl(M::as_reborrowed(&mut self.game))
    }
}

/// Search object with one stored move.
/// 
/// This guarantees that the stored move is entirely legal for the current position, which allows
/// operations to be done on the move safely without double-checking its legality.
/// 
/// Additionally, if the [`Mutability`] for this is set to [`Exclusive`] (which you can get
/// indirectly from [`Game::search_mut`]), you can use the [`Self::test`] method to test a move.
/// After calling that, this object will still be accessbile for more operations.
pub struct SearchOne<'game, M>
where
    M: Mutability,
{
    game: M::Borrow<'game, Game>,
    mv: Move,
}

impl<'game, M> SearchOne<'game, M>
where
    M: Mutability,
{
    /// Returns an immutable reference to the game.
    #[inline]
    #[must_use]
    pub fn game(&self) -> &Game {
        M::as_shared(&self.game)
    }

    /// Returns the stored move.
    #[inline]
    #[must_use]
    pub fn mv(&self) -> Move {
        self.mv
    }

    /// Returns the piece that is moved.
    #[inline]
    #[must_use]
    pub fn moved_piece(&self) -> Piece {
        let position = self.game().position();
        let square = self.mv.from();

        // SAFETY: The invariant guarantees this is a valid move for the current position.
        // All chess moves have a piece on the starting square.
        unsafe { position.piece_at_unchecked(square) }
    }

    /// Returns the piece that is captured, if any.
    #[inline]
    #[must_use]
    pub fn captured_piece(&self) -> Option<Piece> {
        if self.mv.flags().is_capture() {
            let piece = self.game().position().piece_at(self.mv.to());
            Some(piece.unwrap_or(Piece::Pawn))
        } else {
            None
        }
    }
}

impl<'game> SearchOne<'game, Exclusive> {
    /// Tests the stored move by playing it, executing a closure, and undoing it. This closure
    /// takes a [`SearchZero`] ready for more search operations. This play/undo sequence is done
    /// entirely without checking due to the guarantee that the stored move is valid.
    #[inline]
    pub fn test<R>(&mut self, closure: impl FnOnce(SearchZero<Exclusive>) -> R) -> R {
        unsafe { self.game.play_unchecked(self.mv) };

        struct UndoGuard<'a>(&'a mut Game);
        impl Drop for UndoGuard<'_> {
            fn drop(&mut self) {
                unsafe { self.0.undo_unchecked() };
            }
        }

        let undo_guard = UndoGuard(self.game);
        closure(SearchZero { game: undo_guard.0 })
    }
}

/// Search object with a container of moves.
/// 
/// This guarantees that the stored moves are entirely legal for the current position, which allows
/// operations to be done on the moves safely without double-checking their legality.
/// 
/// You can use the [`Self::as_one`] method with an index to obtain a [`SearchOne`] for the move at
/// that index, which allows you to do operations specifically for that move, such as testing it.
/// 
/// There are also numerous safe helper functions that perform common operations while keeping the
/// legality invariant.
pub struct SearchMany<'game, Moves, M>
where
    M: Mutability,
{
    game: M::Borrow<'game, Game>,
    moves: Moves,
}

impl<'game, Moves, M> SearchMany<'game, Moves, M>
where
    Moves: AsRef<[Move]>,
    M: Mutability,
{
    /// Returns an immutable reference to the game.
    #[inline]
    #[must_use]
    pub fn game(&self) -> &Game {
        M::as_shared(&self.game)
    }

    /// Returns an immutable reference to the stored moves.
    #[inline]
    #[must_use]
    pub fn moves(&self) -> &Moves {
        &self.moves
    }

    /// Consumes this object into the stored moves.
    #[inline]
    #[must_use]
    pub fn into_moves(self) -> Moves {
        self.moves
    }

    /// Returns a [`SearchOne`] containing the move obtained by indexing the stored moves.
    /// 
    /// ## Panics
    /// 
    /// Panics when the indexing fails.
    #[inline]
    #[must_use]
    pub fn as_one(&mut self, index: usize) -> SearchOne<'_, M> {
        SearchOne {
            game: M::as_reborrowed(&mut self.game),
            mv: self.moves.as_ref()[index],
        }
    }

    /// Indexes the underlying `Moves` container to generate another [`SearchMany`].
    /// The container must implement [`AsMut`] for `[Move]`.
    ///
    /// ## Panics
    ///
    /// Panics when the indexing fails.
    #[inline]
    #[must_use]
    pub fn as_mut_sliced<I>(&mut self, index: I) -> SearchMany<'_, &mut I::Output, M>
    where
        Moves: AsMut<[Move]>,
        I: SliceIndex<[Move]>,
    {
        SearchMany {
            game: M::as_reborrowed(&mut self.game),
            moves: self.moves.as_mut().index_mut(index),
        }
    }

    /// Sorts the moves according to `[T]::sort_unstable_by`, but the comparison function takes a
    /// [`SearchMany`] containing two moves.
    #[inline]
    pub fn sort_unstable_by<K, F>(&mut self, mut f: F)
    where
        Moves: AsMut<[Move]>,
        F: FnMut(SearchMany<'_, [Move; 2], M>) -> Ordering,
        K: Ord,
    {
        self.moves.as_mut().sort_unstable_by(|&a, &b| {
            f(SearchMany {
                game: M::as_reborrowed(&mut self.game),
                moves: [a, b],
            })
        });
    }

    /// Sorts the moves according to `[T]::sort_unstable_by_key`, but the comparison function takes
    /// a [`SearchOne`] containing the relevant move.
    #[inline]
    pub fn sort_unstable_by_key<K, F>(&mut self, mut f: F)
    where
        Moves: AsMut<[Move]>,
        F: FnMut(SearchOne<'_, M>) -> K,
        K: Ord,
    {
        self.moves.as_mut().sort_unstable_by_key(|&mv| {
            f(SearchOne {
                game: M::as_reborrowed(&mut self.game),
                mv,
            })
        });
    }

    /// Swaps two moves at the provided indices.
    /// 
    /// ## Panics
    /// 
    /// Panics if `a` or `b` are out of bounds.
    #[inline]
    pub fn swap(&mut self, a: usize, b: usize)
    where
        Moves: AsMut<[Move]>,
    {
        self.moves.as_mut().swap(a, b);
    }
}
