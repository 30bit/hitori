use crate as hitori;
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
