use crate as hitori;
/// Sequence of 🍄🫑🧀🥚, where each item may or may not be present,
/// capturing the last item in the list
pub struct ShoppingList;

#[hitori::impl_expr]
impl Expr<usize, char> for ShoppingList {
    const PATTERN: _ = (
        #[hitori::repeat(le = 1)]
        (
            #[hitori::capture(last_item)]
            (|ch| ch == '🍄',),
        ),
        #[hitori::repeat(le = 1)]
        (
            #[hitori::capture(last_item)]
            (|ch| ch == '🫑',),
        ),
        #[hitori::repeat(le = 1)]
        (
            #[hitori::capture(last_item)]
            (|ch| ch == '🧀',),
        ),
        #[hitori::repeat(le = 1)]
        (
            #[hitori::capture(last_item)]
            (|ch| ch == '🥚',),
        ),
    );
}
