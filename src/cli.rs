// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use senseicore::hex_utils;
use std::{
    fs::File,
    io::{self, Read},
};

use clap::{Arg, Command};
use sensei::GetBalanceRequest;
use sensei::{admin_client::AdminClient, node_client::NodeClient};
use tonic::{metadata::MetadataValue, transport::Channel, Request};

use crate::sensei::{
    CloseChannelRequest, ConnectPeerRequest, CreateAdminRequest, CreateInvoiceRequest,
    CreateNodeRequest, GetUnusedAddressRequest, InfoRequest, KeysendRequest, ListChannelsRequest,
    ListNodesRequest, ListPaymentsRequest, ListPeersRequest, ListUnspentRequest,
    NetworkGraphInfoRequest, OpenChannelRequest, OpenChannelsRequest, PayInvoiceRequest,
    SignMessageRequest, StartNodeRequest,
};

pub mod sensei {
    tonic::include_proto!("sensei");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("senseicli")
        .version("1.0")
        .author("John Cantrell <john@l2.technology>")
        .about("Control your sensei node from a cli")
        .arg(
            Arg::new("datadir")
                .short('d')
                .long("datadir")
                .value_name("DATADIR")
                .help("Sets a custom Sensei data directory")
                .takes_value(true),
        )
        .arg(
            Arg::new("node")
                .short('n')
                .long("node")
                .value_name("NODE")
                .help("Sets the node to issue commands to")
                .takes_value(true),
        )
        .subcommand(
            Command::new("init")
                .about("initialize your Sensei node")
                .arg(
                    Arg::new("username")
                        .required(true)
                        .index(1)
                        .help("username for the root lightning node"),
                )
                .arg(
                    Arg::new("alias")
                        .required(true)
                        .index(2)
                        .help("alias used for the root lightning node"),
                ),
        )
        .subcommand(Command::new("start").about("unlock and start your sensei node"))
        .subcommand(Command::new("listnodes").about("list all the lightning nodes"))
        .subcommand(
            Command::new("createnode")
                .about("create a new child node")
                .arg(
                    Arg::new("username")
                        .required(true)
                        .index(1)
                        .help("username to use for this lightning node"),
                )
                .arg(
                    Arg::new("alias")
                        .required(true)
                        .index(2)
                        .help("alias to use for this lightning node"),
                ),
        )
        .subcommand(Command::new("startnode").about("start a child lightning node"))
        .subcommand(Command::new("getbalance").about("gets wallet's balance"))
        .subcommand(Command::new("getaddress").about("get wallet's next unused address"))
        .subcommand(
            Command::new("createinvoice")
                .about("create an invoice for an amount in msats")
                .arg(
                    Arg::new("amt_msat")
                        .required(true)
                        .index(1)
                        .help("amount in msats"),
                ),
        )
        .subcommand(
            Command::new("openchannel")
                .about("open a channel with another node")
                .arg(
                    Arg::new("node_connection_string")
                        .required(true)
                        .index(1)
                        .help("connection string formatted pubkey@host:port"),
                )
                .arg(
                    Arg::new("amt_satoshis")
                        .required(true)
                        .index(2)
                        .help("how many satoshis to put into this channel"),
                )
                .arg(
                    Arg::new("public")
                        .index(3)
                        .takes_value(true)
                        .long("public")
                        .possible_values(&["true", "false"])
                        .required(true)
                        .help("announce this channel to the network?"),
                ),
        )
        .subcommand(
            Command::new("closechannel")
                .about("close a channel")
                .arg(
                    Arg::new("channel_id")
                        .required(true)
                        .index(1)
                        .help("the id of the channel"),
                )
                .arg(
                    Arg::new("force")
                        .index(2)
                        .takes_value(true)
                        .long("force")
                        .possible_values(&["true", "false"])
                        .required(true)
                        .help("force close this channel?"),
                ),
        )
        .subcommand(
            Command::new("payinvoice").about("pay an invoice").arg(
                Arg::new("invoice")
                    .required(true)
                    .index(1)
                    .help("bolt11 invoice"),
            ),
        )
        .subcommand(
            Command::new("keysend")
                .about("send a payment to a public key")
                .arg(
                    Arg::new("dest_pubkey")
                        .required(true)
                        .index(1)
                        .help("destination public key to send payment to"),
                )
                .arg(
                    Arg::new("amt_msat")
                        .required(true)
                        .index(2)
                        .help("amount of millisatoshis to pay"),
                ),
        )
        .subcommand(
            Command::new("connectpeer")
                .about("connect to a peer on the lightning network")
                .arg(
                    Arg::new("node_connection_string")
                        .required(true)
                        .index(1)
                        .help("peer's connection string formatted pubkey@host:port"),
                ),
        )
        .subcommand(
            Command::new("signmessage")
                .about("sign a message with your nodes key")
                .arg(
                    Arg::new("message")
                        .required(true)
                        .index(1)
                        .help("the message to be signed"),
                ),
        )
        .subcommand(Command::new("listchannels").about("list channels"))
        .subcommand(Command::new("listpayments").about("list payments"))
        .subcommand(Command::new("listpeers").about("list peers"))
        .subcommand(Command::new("nodeinfo").about("see information about your node"))
        .get_matches();

    let (command, command_args) = matches.subcommand().unwrap();

    if command == "init" {
        let channel = Channel::from_static("http://0.0.0.0:3000")
            .connect()
            .await?;
        let mut admin_client = AdminClient::new(channel);

        let username = command_args.value_of("username").unwrap();

        let mut passphrase = String::new();
        print!("set a passphrase: ");
        io::stdin().read_line(&mut passphrase)?;

        let request = tonic::Request::new(CreateAdminRequest {
            username: username.to_string(),
            passphrase,
        });
        let response = admin_client.create_admin(request).await?;
        println!("{:?}", response.into_inner());
    } else {
        let data_dir = matches.value_of("datadir").unwrap_or("./.sensei");
        let node = matches.value_of("node").unwrap_or("admin");
        let macaroon_path = format!("{}/{}/.ldk/admin.macaroon", data_dir, node);

        println!("macaroon path: {:?}", macaroon_path);

        let mut macaroon_file = File::open(macaroon_path)?;
        let mut macaroon_raw = Vec::new();
        let _bytes = macaroon_file.read_to_end(&mut macaroon_raw)?;
        let macaroon_hex_str = hex_utils::hex_str(&macaroon_raw);

        let channel = Channel::from_static("http://0.0.0.0:3000")
            .connect()
            .await?;
        let macaroon = MetadataValue::from_str(&macaroon_hex_str)?;
        let admin_macaroon = macaroon.clone();

        let mut client = NodeClient::with_interceptor(channel, move |mut req: Request<()>| {
            req.metadata_mut().insert("macaroon", macaroon.clone());
            Ok(req)
        });

        let admin_channel = Channel::from_static("http://0.0.0.0:3000")
            .connect()
            .await?;

        let mut admin_client =
            AdminClient::with_interceptor(admin_channel, move |mut req: Request<()>| {
                req.metadata_mut()
                    .insert("macaroon", admin_macaroon.clone());
                Ok(req)
            });

        match command {
            "listnodes" => {
                let request = tonic::Request::new(ListNodesRequest { pagination: None });
                let response = admin_client.list_nodes(request).await?;
                println!("{:?}", response.into_inner());
            }
            "createnode" => {
                let username = command_args.value_of("username").unwrap();
                let alias = command_args.value_of("alias").unwrap();

                let mut passphrase = String::new();
                println!("set a passphrase: ");
                io::stdin().read_line(&mut passphrase)?;

                let request = tonic::Request::new(CreateNodeRequest {
                    username: username.to_string(),
                    alias: alias.to_string(),
                    passphrase,
                    start: false,
                    entropy: None,
                    cross_node_entropy: None,
                });
                let response = admin_client.create_node(request).await?;
                println!("{:?}", response.into_inner());
            }
            "startnode" => {
                let mut passphrase = String::new();
                println!("enter your passphrase: ");
                io::stdin().read_line(&mut passphrase)?;

                let request = tonic::Request::new(StartNodeRequest { passphrase });
                let response = client.start_node(request).await?;
                println!("{:?}", response.into_inner());
            }
            "getbalance" => {
                let request = tonic::Request::new(GetBalanceRequest {});
                let response = client.get_balance(request).await?;
                println!("{:?}", response.into_inner());
            }
            "getaddress" => {
                let request = tonic::Request::new(GetUnusedAddressRequest {});
                let response = client.get_unused_address(request).await?;
                println!("{:?}", response.into_inner());
            }
            "createinvoice" => {
                let amt_msat: Option<Result<u64, _>> = command_args
                    .value_of("amt_msat")
                    .map(|str_amt| str_amt.parse());
                match amt_msat {
                    Some(amt_msat) => {
                        if let Ok(amt_msat) = amt_msat {
                            let request = tonic::Request::new(CreateInvoiceRequest {
                                amt_msat,
                                description: String::from(""),
                            });
                            let response = client.create_invoice(request).await?;
                            println!("{:?}", response.into_inner());
                        } else {
                            println!("invalid amount, please specify in msats");
                        }
                    }
                    None => {
                        println!("amt_msat is required to create an invoice");
                    }
                }
            }
            "openchannel" => {
                let args = command_args;
                let amt_satoshis: u64 = args
                    .value_of("amt_satoshis")
                    .expect("amt_satoshis is required field")
                    .parse()
                    .expect("amount must be in satoshis");

                let node_connection_string = args
                    .value_of("node_connection_string")
                    .expect("node_connection_string required");

                let public: bool = args
                    .value_of("public")
                    .expect("public field required")
                    .parse()
                    .expect("public must be true or false");

                let mut split = node_connection_string.split('@');
                let pubkey = split.next().expect("you must provide pubkey@host:port");
                let host_and_port = split.next().unwrap();
                let request = tonic::Request::new(OpenChannelsRequest {
                    requests: vec![OpenChannelRequest {
                        counterparty_pubkey: pubkey.to_string(),
                        amount_sats: amt_satoshis,
                        public,
                        scid_alias: None,
                        push_amount_msats: None,
                        custom_id: None,
                        counterparty_host_port: Some(host_and_port.to_string()),
                        forwarding_fee_proportional_millionths: None,
                        forwarding_fee_base_msat: None,
                        cltv_expiry_delta: None,
                        max_dust_htlc_exposure_msat: None,
                        force_close_avoidance_max_fee_satoshis: None,
                    }],
                });

                let response = client.open_channels(request).await?;
                println!("{:?}", response.into_inner());
            }
            "closechannel" => {
                let args = command_args;

                let channel_id = args.value_of("channel_id").expect("channel_id required");

                let force: bool = args
                    .value_of("force")
                    .expect("force field required")
                    .parse()
                    .expect("force must be true or false");

                let request = tonic::Request::new(CloseChannelRequest {
                    channel_id: channel_id.to_string(),
                    force,
                });

                let response = client.close_channel(request).await?;
                println!("{:?}", response.into_inner());
            }
            "payinvoice" => {
                let args = command_args;

                let invoice = args.value_of("invoice").expect("invoice required");

                let request = tonic::Request::new(PayInvoiceRequest {
                    invoice: invoice.to_string(),
                });

                let response = client.pay_invoice(request).await?;
                println!("{:?}", response.into_inner());
            }
            "keysend" => {
                let args = command_args;

                let dest_pubkey = args.value_of("dest_pubkey").expect("dest_pubkey required");

                let amt_msat: u64 = args
                    .value_of("amt_msat")
                    .expect("amt_msat is required field")
                    .parse()
                    .expect("amount must be in millisatoshis");

                let request = tonic::Request::new(KeysendRequest {
                    dest_pubkey: dest_pubkey.to_string(),
                    amt_msat,
                });

                let response = client.keysend(request).await?;
                println!("{:?}", response.into_inner());
            }
            "connectpeer" => {
                let args = command_args;

                let node_connection_string = args
                    .value_of("node_connection_string")
                    .expect("node_connection_string required");

                let request = tonic::Request::new(ConnectPeerRequest {
                    node_connection_string: node_connection_string.to_string(),
                });

                let response = client.connect_peer(request).await?;
                println!("{:?}", response.into_inner());
            }
            "signmessage" => {
                let args = command_args;

                let message = args.value_of("message").expect("message required");

                let request = tonic::Request::new(SignMessageRequest {
                    message: message.to_string(),
                });

                let response = client.sign_message(request).await?;
                println!("{:?}", response.into_inner());
            }
            "listchannels" => {
                let request = tonic::Request::new(ListChannelsRequest { pagination: None });
                let response = client.list_channels(request).await?;
                println!("{:?}", response.into_inner());
            }
            "listpayments" => {
                let request = tonic::Request::new(ListPaymentsRequest {
                    pagination: None,
                    filter: None,
                });
                let response = client.list_payments(request).await?;
                println!("{:?}", response.into_inner());
            }
            "listpeers" => {
                let request = tonic::Request::new(ListPeersRequest {});
                let response = client.list_peers(request).await?;
                println!("{:?}", response.into_inner());
            }
            "listunspent" => {
                let request = tonic::Request::new(ListUnspentRequest {});
                let response = client.list_unspent(request).await?;
                println!("{:?}", response.into_inner());
            }
            "networkgraphinfo" => {
                let request = tonic::Request::new(NetworkGraphInfoRequest {});
                let response = client.network_graph_info(request).await?;
                println!("{:?}", response.into_inner());
            }
            "nodeinfo" => {
                let request = tonic::Request::new(InfoRequest {});
                let response = client.info(request).await?;
                println!("{:?}", response.into_inner());
            }
            _ => {
                println!("invalid command. use senseicli --help to see usage instructions.")
            }
        }
    }

    Ok(())
}
