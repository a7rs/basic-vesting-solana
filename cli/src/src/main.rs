use solana_program::{
    msg,
    program_pack::Pack,
    pubkey::Pubkey,
    system_program,
};

use solana_clap_utils::{
    input_parsers::{pubkey_of, value_of},
    input_validators::{is_amount, is_keypair, is_parsable, is_pubkey, is_url},
    keypair::{DefaultSigner, SignerFromPathConfig},
};

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    self,
    signature::Signer,
    transaction::Transaction,
};

use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};
use spl_token;

use std::{
    env,
    fs:: File,
    io,
    path::Path,
    str::FromStr,
};

use chrono::{Datelike, DateTime, NaiveDateTime, TimeZone, Utc};

use clap::{App, AppSettings, Arg, crate_description, crate_name, crate_version, SubCommand};

use serde_derive::{Deserialize, Serialize};
use dirs_next;

use synchrony_vc::{
    instruction::{create, init, unlock},
    state::{unpack_releases, VestingHeader, VestingInfo},
};

const D: i64 = 86400;
const M: f64 = 12.0;
const Y: i64 = 31556952;

#[derive(Serialize, Deserialize)]
struct Config {
    program_id: String,
    mint: String,
    mint_decimals: u8,
    execution_date: String,
    team_vesting_period: f64,
    preseed_vesting_period: f64,
    preseed_price: f64,
    seed_vesting_period: f64,
    seed_price: f64,
    p1_vesting_period: f64,
    p1_price: f64,
    p2_vesting_period: f64,
    p2_price: f64,
}

impl Config {
    pub fn load(config_file: &str) -> Result<Self, io::Error> {
        load_config_file(config_file)
    }
}

fn load_config_file<T, P>(config_file: P) -> Result<T, io::Error>
where
    T: serde::de::DeserializeOwned,
    P: AsRef<Path>,
{
    let file = File::open(config_file)?;
    let config = serde_yaml::from_reader(file)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("{:?}", err)))?;
    Ok(config)
}

struct SolConfig {
    payer: Box<dyn Signer>,
    rpc_client: RpcClient,
}

struct TierInfo {
    group: Group,
    release_periods: f64,
    amount: f64,
}

#[derive(Debug, Eq, PartialEq)]
enum Group {
    Team,
    Private,
}

fn is_leap(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

//Ensure that the release is at the same time on the say day everything month
fn parse_increment(timestamp: i64) -> i64 {
    let date = DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp(timestamp, 0),
        Utc,
    );

    D * {
        match date.month() {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => if is_leap(date.year()) {29} else {28},
            _ => unreachable!(),
        }
    }
}

// Calculate the quantity of tokens unlocked per release
fn parse_releases(config: &Config, tier: &TierInfo) -> Vec<VestingInfo> {
    let mut release_timestamp = DateTime::<Utc>::from_utc(
        NaiveDateTime::from_str(&config.execution_date).unwrap(),
        Utc,
    ).timestamp();

    let release_periods = tier.release_periods as usize;
    let mut release_increment = if tier.group == Group::Team {Y} else {0};

    let mut releases: Vec<VestingInfo> = Vec::with_capacity(release_periods);

    let parse_quantity = |i: usize, adj: f64, pc: f64| -> f64 {
        if (..release_periods - 1).contains(&i) {
            ((tier.amount * pc) / (tier.release_periods - adj)).floor()
        } else {
            let qty = (tier.amount * pc) / (tier.release_periods - adj);
            (qty - qty.floor()) * (tier.release_periods - (1f64 + adj)) + qty
        }
    };

    // Rounding errors occur if calculation is performed as one would expect.
    // Solution is to round down on every release and on the last release
    // find qty - qty.floor() and multiply by the number of releases - 2 then add to qty.
    // This will always return the exact amount.
    for i in 0..release_periods {
        let release_quantity = spl_token::ui_amount_to_amount(
            if tier.group == Group::Team {
                parse_quantity(i, 0f64, 1.0)
            } else {
                match i {
                    0 => tier.amount * 0.1,
                    _ => parse_quantity(i, 1f64, 0.9),
                }
            }, config.mint_decimals,
        );

        release_timestamp += release_increment;
        release_increment = parse_increment(release_timestamp);
        releases.push(VestingInfo {
            timestamp: release_timestamp as u32,
            quantity: release_quantity as u64,
        });
    }
    releases
}

fn command_create_vc(
    config: &Config,
    sol: &SolConfig,
    beneficiary: Pubkey,
    tier: TierInfo,
) {
    let releases = parse_releases(&config, &tier);

    let program_id = Pubkey::from_str(&config.program_id).unwrap();
    let mint = Pubkey::from_str(&config.mint).unwrap();
    let vested_token = get_associated_token_address(&sol.payer.pubkey(), &mint);

    let mut not_found = true;
    let mut seed: [u8; 32] = [0; 32];
    let mut vesting_pubkey = Pubkey::new_unique();
    println!("Creating vesting contract {}", vesting_pubkey);
    while not_found {
        seed = Pubkey::new_unique().to_bytes();
        let bump = Pubkey::find_program_address(&[&seed[..31]], &program_id);
        vesting_pubkey = bump.0;
        seed[31] = bump.1;
        not_found = match &sol.rpc_client.get_account(&vesting_pubkey) {
            Ok(_) => true,
            Err(_) => false,
        };
    }

    let vesting_vault = get_associated_token_address(&vesting_pubkey, &mint);

    let instructions = [
        init(
            &program_id,
            &sol.payer.pubkey(),
            &vesting_pubkey,
            &system_program::id(),
            seed,
            tier.release_periods as u32,
        ).unwrap(),
        create_associated_token_account(
            &sol.payer.pubkey(),
            &vesting_pubkey,
            &mint,
        ),
        create(
            &program_id,
            &sol.payer.pubkey(),
            &vested_token,
            &vesting_pubkey,
            &vesting_vault,
            &spl_token::id(),
            seed,
            mint,
            beneficiary,
            releases,
        ).unwrap(),
    ];

    let mut transaction = Transaction::new_with_payer(&instructions, Some(&sol.payer.pubkey()));

    let recent_blockhash = sol.rpc_client.get_recent_blockhash().unwrap().0;
    transaction.sign(&[&*sol.payer], recent_blockhash);

    let signature = sol.rpc_client.send_and_confirm_transaction_with_spinner(&transaction).unwrap();
    println!("  Tokens vested: {}\n  Recipient: {}", tier.amount, beneficiary);
    println!("Signature: {}", signature);
    println!(
        "\nContract seed: {:?}",
        Pubkey::new_from_array(seed)
    );
}

fn command_unlock_tokens(
    config: &Config,
    sol: &SolConfig,
    seed: [u8; 32],
) {
    let program_id = Pubkey::from_str(&config.program_id).unwrap();
    let mint = Pubkey::from_str(&config.mint).unwrap();

    let (vesting_pda, _) = Pubkey::find_program_address(&[&seed[..31]], &program_id);
    let state = sol.rpc_client.get_account_data(&vesting_pda).unwrap();
    let header = VestingHeader::unpack(&state[..VestingHeader::LEN]).unwrap();
    let beneficiary = header.beneficiary;
    println!("Unlocking tokens for {}", beneficiary);

    let vesting_vault_pubkey = get_associated_token_address(&vesting_pda, &header.mint);
    let beneficiary_token_pubkey = get_associated_token_address(&beneficiary, &mint);

    let unlock_token_ix = unlock(
        &program_id,
        &vesting_pda,
        &vesting_vault_pubkey,
        &beneficiary,
        &beneficiary_token_pubkey,
        &spl_token::id(),
        seed,
    ).unwrap();

    let quantity_unlocked = {
        let releases = unpack_releases(&state[VestingHeader::LEN..]).unwrap();
        let mut quantity = 0;
        for release in releases {
            if release.timestamp <= Utc::now().timestamp() as u32 {
                quantity += release.quantity;
            }
        }
        spl_token::amount_to_ui_amount(quantity, config.mint_decimals)
    };

    let mut transaction = Transaction::new_with_payer(&[unlock_token_ix], Some(&sol.payer.pubkey()));

    let recent_blockhash = sol.rpc_client.get_recent_blockhash().unwrap().0;
    transaction.sign(&[&*sol.payer], recent_blockhash);

    let signature = sol.rpc_client.send_and_confirm_transaction_with_spinner(&transaction).unwrap();
    println!("  Tokens unlocked: {}\n  Recipient: {}", quantity_unlocked, beneficiary);
    println!("Signature: {}", signature);
}

fn command_info(
    config: &Config,
    sol: &SolConfig,
    seed: [u8; 32],
) {
    let program_id = Pubkey::from_str(&config.program_id).unwrap();
    msg!("Program ID: {:?}", &config.program_id);
    msg!("Seed: {:?}", Pubkey::new_from_array(seed));

    let (vesting_pda, _) = Pubkey::find_program_address(&[&seed[..31]], &program_id);
    msg!("Vesting Account Address: {:?}", &vesting_pda);

    let state = sol.rpc_client.get_account_data(&vesting_pda).unwrap().clone();
    let header = VestingHeader::unpack(&state[..VestingHeader::LEN]).unwrap();
    let vesting_vault = get_associated_token_address(&vesting_pda, &header.mint);
    msg!("Vesting Vault Address: {:?}", &vesting_vault);
    msg!("Initialized: {:?}", &header.is_initialized);
    msg!("Mint Address: {:?}", &header.mint);
    msg!("Beneficary Address: {:?}", &header.beneficiary);
    let beneficiary_token_address = get_associated_token_address(&header.beneficiary, &header.mint);
    msg!("Beneficiary Token Address: {:?}", &beneficiary_token_address);

    let releases = unpack_releases(&state[VestingHeader::LEN..]).unwrap();
    let mut total_tokens: u64 = 0;

    for i in 0..releases.len() {
        let release_quantity = spl_token::amount_to_ui_amount(releases[i].quantity, config.mint_decimals);
        total_tokens += release_quantity as u64;
        let j = i + 1;
        let date = Utc.timestamp(releases[i].timestamp.clone() as i64, 0).to_string();
        msg!("\nRelease {:?}", j);
        msg!("Release Date: {:?}", date);
        msg!("Quantity: {:?}", release_quantity);
    }
    msg!("Total Tokens Remaining: {:?}", total_tokens);
}

fn main() {
    let app_matches = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg({
            let arg = Arg::with_name("config_file")
                .short("c")
                .long("config")
                .value_name("PATH")
                .takes_value(true)
                .global(true)
                .help("Configuration file to use");
            if let Some(ref config_file) = *solana_cli_config::CONFIG_FILE {
                arg.default_value(&config_file)
            } else {
                arg
            }
        })
        .arg(
            Arg::with_name("rpc_url")
            .short("u")
            .long("url")
            .validator(is_url)
            .takes_value(true)
            .global(true)
            .help("Specify target Solana cluster")
        )
        .arg(
            Arg::with_name("payer")
            .long("payer")
            .value_name("KEYPAIR")
            .validator(is_keypair)
            .takes_value(true)
            .help(
                "Specify payer. \
                Defaults to client keypair."
            ),
        )
        .subcommand(SubCommand::with_name("create").about("Create new vesting contract")
            .arg(
                Arg::with_name("beneficiary_address")
                .long("beneficiary")
                .value_name("ADDRESS")
                .validator(is_pubkey)
                .takes_value(true)
                .index(1)
                .required(true)
                .help("Specify the address for the beneficiary.")
            )
            .arg(
                Arg::with_name("tier")
                .long("tier")
                .takes_value(true)
                .index(2)
                .required(true)
                .help(
                    "Investor tier. \
                    0 = Team & advisors \
                    1 = Preseed \
                    2 = Seed \
                    3 = Private 1 \
                    4 = Private 2"
                )
            )
            .arg(
                Arg::with_name("amount")
                .long("amount")
                .value_name("AMOUNT")
                .validator(is_amount)
                .takes_value(true)
                .allow_hyphen_values(true)
                .index(3)
                .required(true)
                .help("Investment size in USD")
            )
            .arg(
                Arg::with_name("payer")
                .long("payer")
                .value_name("KEYPAIR")
                .validator(is_keypair)
                .takes_value(true)
                .help(
                    "Specify the transaction fee payer account address. \
                        Defaults to the client keypair.",
                ),
            )
        )
        .subcommand(SubCommand::with_name("unlock").about("unlock vested tokens")
            .arg(
                Arg::with_name("seed")
                .long("seed")
                .value_name("SEED")
                .validator(is_parsable::<String>)
                .takes_value(true)
                .index(1)
                .required(true)
                .help(
                    "Specify the seed for the vesting contract.",
                ),
            )
        )
        .subcommand(SubCommand::with_name("info").about("Print vesting information")
            .arg(
                Arg::with_name("seed")
                .long("seed")
                .value_name("SEED")
                .validator(is_parsable::<String>)
                .takes_value(true)
                .index(1)
                .required(true)
                .help(
                    "Specify the seed for the vesting contract.",
                ),
            )
        ).get_matches();

    let mut wallet_manager = None;
    let (sub_command, sub_matches) = app_matches.subcommand();
    let matches = sub_matches.unwrap();

    //Retrieves payer keypair and target RPC from the config file.
    let sol_config = {
        let cli_config = if let Some(config_file) = matches.value_of("config_file") {
            solana_cli_config::Config::load(config_file).unwrap_or_default()
        } else {
            solana_cli_config::Config::default()
        };

        let rpc_url = &cli_config.json_rpc_url;

        let default_signer_arg_name = "owner".to_string();
        let default_signer_path = cli_config.keypair_path.clone();
        let default_signer = DefaultSigner {
            path: default_signer_path,
            arg_name: default_signer_arg_name,
        };

        let payer = {
            let config = SignerFromPathConfig {
                allow_null_signer: true,
            };
            let payer = default_signer
                .signer_from_path_with_config(&matches, &mut wallet_manager, &config)
                .unwrap_or_else(|e| {
                    eprintln!("error: {}", e);
                    std::process::exit(1);
                });
            payer
        };

        SolConfig {
            rpc_client: RpcClient::new(rpc_url.clone()),
            payer,
        }
    };

    //Unpacks config.yml into a Config struct
    let config_file = {
        let mut config_path = dirs_next::home_dir().unwrap();
        config_path.extend(&["synchrony", "synchrony-vc", "cli", "config.yml"]);
        config_path.to_str().unwrap().to_string()
    };

    let config: Config = Config::load(&config_file).unwrap();

    let _ = match (sub_command, sub_matches) {
        ("create", Some(args)) => {
            let beneficiary_pubkey = pubkey_of(args, "beneficiary_address").unwrap();
            let tier = value_of(args, "tier").unwrap();
            let amount = value_of(args, "amount").unwrap();

            let tier = match tier {
                0 => TierInfo {
                    group: Group::Team,
                    release_periods: (config.team_vesting_period * M) - M,
                    amount,
                },
                1 => TierInfo {
                    group: Group::Private,
                    release_periods: config.preseed_vesting_period * M,
                    amount: amount / config.preseed_price,
                },
                2 => TierInfo {
                    group: Group::Private,
                    release_periods: config.seed_vesting_period * M,
                    amount: amount / config.seed_price,
                },
                3 => TierInfo {
                    group: Group::Private,
                    release_periods: config.p1_vesting_period * M,
                    amount: amount / config.p1_price,
                },
                4 => TierInfo {
                    group: Group::Private,
                    release_periods: config.p2_vesting_period * M,
                    amount: amount / config.p2_price,
                },
                _ => std::process::exit(1),
            };

            command_create_vc(
                &config,
                &sol_config,
                beneficiary_pubkey,
                tier,
            )
        },
        ("unlock", Some(args)) => {
            let seed = pubkey_of(args, "seed").unwrap().to_bytes();
            command_unlock_tokens(
                &config,
                &sol_config,
                seed,
            )
        },
        ("info", Some(args)) => {
            let seed = pubkey_of(args, "seed").unwrap().to_bytes();
            command_info(
                &config,
                &sol_config,
                seed,
            )
        },
        _ => unreachable!(),
    };
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_vested_qty() {
        let config_file = {
            let mut config_path = dirs_next::home_dir().unwrap();
            config_path.extend(&["synchrony", "synchrony-vc", "cli", "config.yml"]);
            config_path.to_str().unwrap().to_string()
        };
        let config = Config::load(&config_file).unwrap();

        let tier = TierInfo {
            group: Group::Private,
            release_periods: 24.0,
            amount: 5000000.0,
        };
        let releases = parse_releases(&config, &tier);

        let total = releases.iter().fold(0, |a, b| a + b.quantity);

        assert_eq!(spl_token::amount_to_ui_amount(total, config.mint_decimals), tier.amount);
    }


    #[test]
    fn test_vested_p1_even() {
        let config_file = {
            let mut config_path = dirs_next::home_dir().unwrap();
            config_path.extend(&["synchrony", "synchrony-vc", "cli", "config.yml"]);
            config_path.to_str().unwrap().to_string()
        };
        let config = Config::load(&config_file).unwrap();

        let tier = TierInfo {
            group: Group::Private,
            release_periods: 21.0,
            amount: 123456.0,
        };
        let releases = parse_releases(&config, &tier);

        let total = releases.iter().fold(0, |a, b| a + b.quantity);

        assert_eq!(spl_token::amount_to_ui_amount(total, config.mint_decimals), tier.amount);
    }

    #[test]
    fn test_vested_p1_odd() {
        let config_file = {
            let mut config_path = dirs_next::home_dir().unwrap();
            config_path.extend(&["synchrony", "synchrony-vc", "cli", "config.yml"]);
            config_path.to_str().unwrap().to_string()
        };
        let config = Config::load(&config_file).unwrap();

        let tier = TierInfo {
            group: Group::Private,
            release_periods: 21.0,
            amount: 98765.0,
        };
        let releases = parse_releases(&config, &tier);

        let total = releases.iter().fold(0, |a, b| a + b.quantity);

        assert_eq!(spl_token::amount_to_ui_amount(total, config.mint_decimals), tier.amount);
    }

    #[test]
    fn test_vested_p2_even() {
        let config_file = {
            let mut config_path = dirs_next::home_dir().unwrap();
            config_path.extend(&["synchrony", "synchrony-vc", "cli", "config.yml"]);
            config_path.to_str().unwrap().to_string()
        };
        let config = Config::load(&config_file).unwrap();

        let tier = TierInfo {
            group: Group::Private,
            release_periods: 18.0,
            amount: 212328.0,
        };
        let releases = parse_releases(&config, &tier);

        let total = releases.iter().fold(0, |a, b| a + b.quantity);

        assert_eq!(spl_token::amount_to_ui_amount(total, config.mint_decimals), tier.amount);
    }

    #[test]
    fn test_vested_p2_odd() {
        let config_file = {
            let mut config_path = dirs_next::home_dir().unwrap();
            config_path.extend(&["synchrony", "synchrony-vc", "cli", "config.yml"]);
            config_path.to_str().unwrap().to_string()
        };
        let config = Config::load(&config_file).unwrap();

        let tier = TierInfo {
            group: Group::Private,
            release_periods: 18.0,
            amount: 299999.0,
        };
        let releases = parse_releases(&config, &tier);

        let total = releases.iter().fold(0, |a, b| a + b.quantity);

        assert_eq!(spl_token::amount_to_ui_amount(total, config.mint_decimals), tier.amount);
    }

    #[test]
    fn test_team_vesting_qty() {
        let config_file = {
            let mut config_path = dirs_next::home_dir().unwrap();
            config_path.extend(&["synchrony", "synchrony-vc", "cli", "config.yml"]);
            config_path.to_str().unwrap().to_string()
        };
        let config = Config::load(&config_file).unwrap();

        let tier = TierInfo {
            group: Group::Team,
            release_periods: 24.0,
            amount: 100000.0,
        };
        let releases = parse_releases(&config, &tier);

        let total = releases.iter().fold(0, |a, b| a + b.quantity);

        assert_eq!(spl_token::amount_to_ui_amount(total, config.mint_decimals), tier.amount);
    }

    #[test]
    fn team_vesting_duration() {
        let config_file = {
            let mut config_path = dirs_next::home_dir().unwrap();
            config_path.extend(&["synchrony", "synchrony-vc", "cli", "config.yml"]);
            config_path.to_str().unwrap().to_string()
        };
        let config = Config::load(&config_file).unwrap();

        let tier = TierInfo {
            group: Group::Team,
            release_periods: 24.0,
            amount: 100000.0,
        };
        let releases = parse_releases(&config, &tier);
        let timestamp = DateTime::<Utc>::from_utc(
            NaiveDateTime::from_str(&config.execution_date).unwrap(),
            Utc,
        ).timestamp();
        assert_eq!(releases[0].timestamp, (timestamp + Y) as u32)
    }
}
