/// String of either one 🚃, or five 🚃, capturing first and last 🚃
pub struct TrainCars;

#[hitori::impl_expr]
impl Expr<usize, char> for TrainCars {
    const PATTERN: _ = [
        #[hitori::capture(first_car, last_car)]
        (
            #[hitori::position(first, last)]
            (|ch| ch == '🚃',),
        ),
        (
            #[hitori::capture(first_car)]
            (
                #[hitori::position(first)]
                (|ch| ch == '🚃',),
            ),
            #[hitori::repeat(eq = 3)]
            (|ch| ch == '🚃',),
            #[hitori::capture(last_car)]
            (
                #[hitori::position(last)]
                (|ch| ch == '🚃',),
            ),
        ),
    ];
}
