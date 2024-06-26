use std::{
    collections::BTreeSet,
    fs::File,
    io::{Read, Write},
    str::FromStr,
};

use anyhow::Result;
use bdk_esplora::EsploraExt;
use bdk_wallet::{wallet::Wallet, KeychainKind, SignOptions};
use bitcoin::{Address, Amount, Network};
use clap::Parser;

mod cli;
use cli::{Commands, Wallet as WellWallet};

fn main() -> Result<()> {
    let wallet = WellWallet::parse();
    wallet.dispatch_command()?;
    Ok(())
}

/// Implementation of the `WellWallet` struct.
impl WellWallet {
    /// Default store path for the wallet.
    #[allow(dead_code)]
    pub const DEFAULT_STORE_PATH: &'static str = "/tmp/well/";
    /// Default wallet descriptor.
    pub const DEFAULT_WALLET_DESCRIPTOR: &'static str = "wallet.descriptor";

    /// Dispatches the command based on the value of `self.commands`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the command is successfully dispatched, otherwise returns an error.
    pub fn dispatch_command(&self) -> Result<()> {
        match &self.commands {
            Commands::NewWallet => {
                println!("New wallet \r\n");
                self.new_wallet()?;
            }
            Commands::InitWallet => {
                println!("Init wallet \r\n");
                self.init_wallet()?;
            }
            Commands::CreateAddress => {
                println!("Create address \r\n");
                self.create_address()?;
            }
            Commands::GetBalance => {
                println!("Get balance \r\n");
                self.get_balance()?;
            }
            Commands::ListTransactions => {
                println!("List transaction \r\n");
                self.list_transactions()?;
            }
            Commands::Pay { receiver, amount } => {
                println!("Send transaction \r\n");
                self.pay(receiver, *amount)?;
            }
        };
        Ok(())
    }

    /// Creates a new wallet and stores the descriptor in the store path.
    fn new_wallet(&self) -> Result<()> {
        // first generate a new private key
        // TODO: Subsequent use: bitcoin::bip32::Xpriv

        let ext_private_key = bitcoin::key::PrivateKey::generate(Network::Regtest);
        let int_private_key = bitcoin::key::PrivateKey::generate(Network::Regtest);

        let int_descriptor = format!("wpkh({})", int_private_key.to_string());
        let ext_descriptor = format!("wpkh({})", ext_private_key.to_string());
        // let (ext_descriptor, ext_key_map, ext_networks) =
        //     bdk_wallet::descriptor!(wpkh(ext_private_key))?;
        // TODO: expanded version of the above macro
        // FIXME: why `bdk_wallet::descriptor!` only has public key?
        // let (ext_descriptor, ext_key_map, ext_networks) = {
        //     use ::bdk_wallet::miniscript::descriptor::{Descriptor, DescriptorPublicKey};
        //     {
        //         use ::bdk_wallet::miniscript::descriptor::Wpkh;
        //         #[allow(unused_imports)]
        //         use ::bdk_wallet::keys::{DescriptorKey, IntoDescriptorKey};
        //         let secp = ::bdk_wallet::bitcoin::secp256k1::Secp256k1::new();
        //         ext_private_key
        //             .into_descriptor_key()
        //             .and_then(|key: DescriptorKey<::bdk_wallet::miniscript::Segwitv0>| {
        //                 key.extract(&secp)
        //             })
        //             .map_err(::bdk_wallet::descriptor::DescriptorError::Key)
        //             .map(|(pk, key_map, valid_networks)| (Wpkh::new(pk), key_map, valid_networks))
        //     }
        //     .and_then(|(a, b, c)| Ok((a.map_err(|e| miniscript::Error::from(e))?, b, c)))
        //     .map(|(a, b, c)| (Descriptor::<DescriptorPublicKey>::Wpkh(a), b, c))
        // }?;
        // let (int_descriptor, int_key_map, int_networks) =
        //     bdk_wallet::descriptor!(wpkh(int_private_key))?;

        println!("Generated descriptor: internal: {} \r\n", int_descriptor);
        println!("Generated descriptor: external: {} \r\n", ext_descriptor);

        let store_path = std::path::Path::new(self.store_path.as_str());
        if !store_path.exists() {
            std::fs::create_dir_all(store_path)?;
        }

        let wallet_descriptor_path = store_path.join(Self::DEFAULT_WALLET_DESCRIPTOR);
        let mut wallet_descriptor = File::create(wallet_descriptor_path)?;
        wallet_descriptor.write_all(
            serde_json::json!(
                {
                    "internal": int_descriptor,
                    "external": ext_descriptor
                }
            )
            .to_string()
            .as_bytes(),
        )?;
        wallet_descriptor.sync_all()?;

        Ok(())
    }

    /// Initializes the wallet from the stored descriptor or the provided descriptor.
    fn init_wallet(&self) -> Result<()> {
        self.wallet().map(|_| ())
    }

    /// Creates a new address for receiving bitcoin.
    fn create_address(&self) -> Result<()> {
        let mut wallet = self.wallet()?;
        let address = wallet.next_unused_address(KeychainKind::External);
        println!("Generated Address: {}", address);
        Ok(())
    }

    /// Gets the wallet balance.
    fn get_balance(&self) -> Result<()> {
        let wallet = self.wallet()?;

        let balance = wallet.balance();
        println!(
            " \r\n Wallet balance after syncing: \r\n {:?} \r\n",
            balance
        );

        Ok(())
    }

    /// Lists all transactions.
    fn list_transactions(&self) -> Result<()> {
        let wallet = self.wallet()?;

        for tx in wallet.tx_graph().full_txs() {
            println!("{:?} \r\n", tx.txid);
        }
        Ok(())
    }

    /// Sends bitcoin to the receiver address.
    fn pay(&self, receiver: &String, amount: u64) -> Result<()> {
        let mut wallet = self.wallet()?;
        let balance = wallet.balance();
        let amount = Amount::from_sat(amount);
        if balance.total() < amount {
            println!("balance: {}, amount: {}", balance.total(), amount);
            anyhow::bail!("Insufficient balance");
        }

        let faucet_address = Address::from_str(receiver)?.require_network(Network::Regtest)?;

        let mut tx_builder = wallet.build_tx();
        tx_builder
            .add_recipient(faucet_address.script_pubkey(), amount)
            .enable_rbf();

        // TODO: what is psbt?
        let mut psbt = tx_builder.finish()?;

        // FIXED (by descriptor):
        // Could not satisfy a script (fragment) because of a missing signature
        // MissingSig(bitcoin::PublicKey),
        let finalized = wallet.sign(&mut psbt, SignOptions::default())?;
        if !finalized {
            anyhow::bail!("Transaction not finalized");
        }

        let tx = psbt.extract_tx()?;

        // broadcast the transaction
        self.client()?.broadcast(&tx)?;
        println!("\r\n Transaction sent: {:?} \r\n", tx.compute_txid());
        Ok(())
    }

    fn client(&self) -> Result<bdk_esplora::esplora_client::BlockingClient> {
        let esplora_address = self.esplora_address.as_str();
        let client = bdk_esplora::esplora_client::Builder::new(esplora_address).build_blocking();
        Ok(client)
    }

    fn wallet(&self) -> Result<Wallet> {
        let (descriptor, change_descriptor) = self.descriptor()?;
        let mut wallet = Wallet::new(&descriptor, &change_descriptor, Network::Regtest)?;

        print!("Syncing... \r\n");
        let client =
            bdk_esplora::esplora_client::Builder::new(&self.esplora_address).build_blocking();

        let request = wallet.start_full_scan().inspect_spks_for_all_keychains({
            let mut once = BTreeSet::<KeychainKind>::new();
            move |keychain, spk_i, s| {
                match once.insert(keychain) {
                    true => print!(
                        "\n Scanning keychain [{:?}], spk_i: {}, script: {}",
                        keychain, spk_i, s
                    ),
                    false => print!(" {:<3} \r\n", spk_i),
                };
                std::io::stdout().flush().expect("must flush")
            }
        });

        let mut update = client.full_scan(request, 20, 2)?;
        let now = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs();
        let _ = update.graph_update.update_last_seen_unconfirmed(now);

        wallet.apply_update(update)?;

        // For teminal clean
        println!("\r\n");

        Ok(wallet)
    }

    fn descriptor(&self) -> Result<(String, String)> {
        let mut desicrptor: &str = self.desciptor.as_str();
        let mut change_descriptor: &str = self.change_descriptor.as_str();

        let store_path = std::path::Path::new(self.store_path.as_str());
        let wallet_desicrptor_path = store_path.join(Self::DEFAULT_WALLET_DESCRIPTOR);

        let mut desicrptor_store = File::open(&wallet_desicrptor_path)?;
        let mut buf: String = String::new();
        desicrptor_store.read_to_string(&mut buf)?;

        // deserialize the descriptor
        let persist_desicrptor: serde_json::Value = serde_json::from_str(&buf)?;
        let internal = persist_desicrptor["internal"].as_str().unwrap_or("");
        let external = persist_desicrptor["external"].as_str().unwrap_or("");

        if desicrptor.is_empty() {
            if internal.is_empty() {
                anyhow::bail!("Wallet descriptor is empty");
            }

            desicrptor = internal;
        }

        if change_descriptor.is_empty() {
            if external.is_empty() {
                anyhow::bail!("Change descriptor is empty");
            }

            change_descriptor = external;
        }

        if desicrptor != internal || change_descriptor != external {
            // store new desicrptor by truncating the file
            let mut desicrptor_store = File::create(wallet_desicrptor_path)?;
            desicrptor_store.write_all(
                serde_json::json!(
                    {
                        "internal": desicrptor,
                        "external": change_descriptor
                    }
                )
                .to_string()
                .as_bytes(),
            )?;

            desicrptor_store.sync_all()?;
        }

        Ok((desicrptor.to_string(), change_descriptor.to_string()))
    }
}
