#[derive(Debug, Copy, Clone)]
pub enum Side {
    BUY,
    SELL,
}

// assume fixed-point precision (1e-8)
pub type Decimal64 = i64;