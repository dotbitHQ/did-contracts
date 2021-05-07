pub const ACCOUNT_CELL_BASIC_CAPACITY: u64 = 20_000_000_000;

pub fn calc_account_storage_capacity(account_name_storage: u64) -> u64 {
    ACCOUNT_CELL_BASIC_CAPACITY + (account_name_storage * 100_000_000)
}

pub fn calc_yearly_capacity(yearly_price: u64, quote: u64, discount: u32) -> u64 {
    let total;
    if yearly_price < quote {
        total = yearly_price * 100_000_000 / quote;
        println!(
            "Because of price < quote: price_in_CKB({}) = price({}) * 100_000_000 / quote({})",
            total, yearly_price, quote
        );
    } else {
        total = yearly_price / quote * 100_000_000;
        println!(
            "Because of price >= quote: price_in_CKB({}) = price({}) / quote({}) * 100_000_000",
            total, yearly_price, quote
        );
    }

    let ret = total - (total * discount as u64 / 10000);
    println!(
        "price_in_CKB_discounted({}) = price_in_CKB({}) - (price_in_CKB({}) * discount({}) / 10000)",
        ret, total, total, discount
    );

    ret
}

pub fn calc_duration_from_paid(paid: u64, yearly_price: u64, quote: u64, discount: u32) -> u64 {
    let yearly_capacity = calc_yearly_capacity(yearly_price, quote, discount);

    // Original formula: duration = (paid / yearly_capacity) * 365 * 86400
    // But CKB VM can only handle uint, so we put division to later for higher precision.
    let ret = paid * 365 / yearly_capacity * 86400;
    println!(
        "duration({}) = profit({}) * 365 / price({}) * 86400",
        ret, paid, yearly_price
    );

    ret
}
