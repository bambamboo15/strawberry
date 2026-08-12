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
    #[inline]
    #[must_use]
    pub fn search(&self) -> SearchZero<'_, Shared> {
        SearchZero { game: self }
    }

    #[inline]
    #[must_use]
    pub fn search_mut(&mut self) -> SearchZero<'_, Exclusive> {
        SearchZero { game: self }
    }

    #[inline]
    #[must_use]
    pub fn get_legal_move(&self, mv: Move) -> Option<SearchOne<'_, Shared>> {
        Self::get_legal_move_impl(self, mv)
    }

    #[inline]
    #[must_use]
    pub fn get_legal_move_mut(&mut self, mv: Move) -> Option<SearchOne<'_, Exclusive>> {
        Self::get_legal_move_impl(self, mv)
    }

    #[inline]
    #[must_use]
    pub fn legal_moves(&self) -> SearchMany<'_, MoveList, Shared> {
        Self::legal_moves_impl(self)
    }

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
    #[inline]
    #[must_use]
    pub fn game(&self) -> &Game {
        M::as_shared(&self.game)
    }

    #[inline]
    #[must_use]
    pub fn get_legal_move(&mut self, mv: Move) -> Option<SearchOne<'_, M>> {
        Game::get_legal_move_impl(M::as_reborrowed(&mut self.game), mv)
    }

    #[inline]
    #[must_use]
    pub fn legal_moves(&mut self) -> SearchMany<'_, MoveList, M> {
        Game::legal_moves_impl(M::as_reborrowed(&mut self.game))
    }
}

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
    #[inline]
    #[must_use]
    pub fn game(&self) -> &Game {
        M::as_shared(&self.game)
    }

    #[inline]
    #[must_use]
    pub fn mv(&self) -> Move {
        self.mv
    }

    #[inline]
    #[must_use]
    pub fn moved_piece(&self) -> Piece {
        let position = self.game().position();
        let square = self.mv.from();

        // SAFETY: The invariant guarantees this is a valid move for the current position.
        // All chess moves have a piece on the starting square.
        unsafe { position.piece_at_unchecked(square) }
    }

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

pub struct SearchMany<'game, Moves, M>
where
    M: Mutability,
{
    pub game: M::Borrow<'game, Game>,
    pub moves: Moves,
}

impl<'game, Moves, M> SearchMany<'game, Moves, M>
where
    Moves: AsRef<[Move]>,
    M: Mutability,
{
    #[inline]
    #[must_use]
    pub fn game(&self) -> &Game {
        M::as_shared(&self.game)
    }

    #[inline]
    #[must_use]
    pub fn moves(&self) -> &Moves {
        &self.moves
    }

    #[inline]
    #[must_use]
    pub fn into_moves(self) -> Moves {
        self.moves
    }

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
    /// ## Panic
    ///
    /// This panics when the indexing fails.
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

    #[inline]
    pub fn swap(&mut self, a: usize, b: usize)
    where
        Moves: AsMut<[Move]>,
    {
        self.moves.as_mut().swap(a, b);
    }
}
