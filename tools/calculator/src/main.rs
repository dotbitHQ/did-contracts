use clap::Clap;

mod util;

#[derive(Clap, Debug)]
#[clap(version = "1.0", author = "Link Xie. <xieaolin@gmail.com>")]
struct Options {
    #[clap(subcommand)]
    action: SubCommand,
}

#[derive(Clap, Debug)]
enum SubCommand {
    New(NewAction),
    Renew(RenewAction),
}

#[derive(Clap, Debug)]
struct NewAction {
    #[clap(
        long = "capacity",
        about = "The capacity of PreAccountCell. Required if --profit is not provided."
    )]
    capacity: Option<u64>,
    #[clap(
        long = "account-name-storage",
        about = "The length of account, must count its suffix. Required if --profit is not provided."
    )]
    account_name_storage: Option<u64>,
    #[clap(long = "profit", about = "The profit of proposal confirmation.")]
    profit: Option<u64>,
    #[clap(
        long = "price",
        required = true,
        about = "The register fee of account for one year."
    )]
    price: u64,
    #[clap(
        long = "quote",
        required = true,
        about = "The quote of CKB to USD, AKA USD/CKB."
    )]
    quote: u64,
    #[clap(
        long = "discount",
        default_value = "0",
        about = "The discount of register fee."
    )]
    discount: u32,
    #[clap(long = "current", about = "The current timestamp, can be omitted.")]
    current: Option<u64>,
}

#[derive(Clap, Debug)]
struct RenewAction {
    #[clap(
        long = "profit",
        required = true,
        about = "The total profit which DAS get."
    )]
    profit: u64,
    #[clap(
        long = "price",
        required = true,
        about = "The renew fee of account for one year."
    )]
    price: u64,
    #[clap(
        long = "quote",
        required = true,
        about = "The quote of CKB to USD, AKA USD/CKB."
    )]
    quote: u64,
}

fn main() {
    // Parse options
    let options: Options = Options::parse();
    // println!("{:?}", options);

    match options.action {
        SubCommand::New(options) => {
            let profit;
            if options.profit.is_none() {
                if options.account_name_storage.is_none() {
                    panic!(
                        "Params --account-name-storage is required when --profit is not provided."
                    );
                }
                if options.capacity.is_none() {
                    panic!("Params --capacity is required when --profit is not provided.");
                }

                let storage_capacity =
                    util::calc_account_storage_capacity(options.account_name_storage.unwrap());
                println!(
                    "storage_capacity({}) = ACCOUNT_CELL_BASIC_CAPACITY({}) + (account_name_storage({}) * 100_000_000)",
                    storage_capacity,
                    util::ACCOUNT_CELL_BASIC_CAPACITY,
                    options.account_name_storage.unwrap()
                );

                profit = options.capacity.unwrap() - storage_capacity;
                println!(
                    "total_profit({}) = capacity({}) - storage_capacity({})",
                    profit,
                    options.capacity.unwrap(),
                    storage_capacity
                );
            } else {
                profit = options.profit.unwrap();
            }

            let duration = util::calc_duration_from_paid(
                profit,
                options.price,
                options.quote,
                options.discount,
            );

            if let Some(current) = options.current {
                let expired_at = current + duration;
                println!(
                    "expired_at({}) = current({}) - duration({})",
                    expired_at, current, duration
                );
            }
        }
        SubCommand::Renew(options) => {
            let duration =
                util::calc_duration_from_paid(options.profit, options.price, options.quote, 0);
        }
    }
}
