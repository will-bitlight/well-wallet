use clap::Subcommand;
#[derive(clap::Parser, Debug, Clone)]
#[clap(
    version = "1.0",
    author = "Bitlightlabs Well",
    after_help = r#"Examples

## Initialize wallet
$ ./target/debug/well-wallet -s "/tmp/well" new-wallet 

## Create address
$ ./target/debug/well-wallet -s "/tmp/well" create-address

## Get balance
$ ./target/debug/well-wallet -s "/tmp/well" get-balance

## List transactions
$ ./target/debug/well-wallet -s "/tmp/well" list-transactions

## Send transaction
$ ./target/debug/well-wallet -s "/tmp/well" pay -r bcrt1qwwf3ckm89aqxzpxhp62ee65s75kn7fnuk0y82g -a 10000

"#
)]
#[clap(version = "1.0", author = "AtlasGraph Authors")]
pub struct Wallet {
    #[clap(
        short,
        long,
        default_value = "http://127.0.0.1:3002",
        value_name = "HOST:PORT",
        help = "Bitcoin esplora server address"
    )]
    pub esplora_address: String,
    #[clap(short, long, help = "Wallet descriptor", default_value = "")]
    pub desciptor: String,
    #[clap(short, long, help = "Change descriptor", default_value = "")]
    pub change_descriptor: String,
    #[clap(short, long, help = "Wallet store path", default_value = "/tmp/well")]
    pub store_path: String,
    #[command(subcommand)]
    pub commands: Commands,
}

// Bitcoin wallet supported subcommands
// 0. New wallet, auto generate descriptor and init wallet
// 1. Initialize wallet, create wallet from descriptor
// 2. Create address, generate new address for receiving
// 3. Get balance, get wallet balance
// 4. List transaction, list all transactions
// 5. Send transaction, send bitcoin to receiver address
// defalut subcommand is InitWallet
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    NewWallet,
    InitWallet,
    CreateAddress,
    GetBalance,
    ListTransactions,
    Pay {
        #[clap(short, long, help = "Receiver address")]
        receiver: String,
        #[clap(short, long, help = "Amount to send")]
        amount: u64,
    },
}
