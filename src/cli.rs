// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

mod hex_utils;
use std::{
    io::{self}
};

use std::{fs};

use serde::{Deserialize, Serialize};

use clap::{App, Arg};
use sensei::GetBalanceRequest;
use sensei::{admin_client::AdminClient, node_client::NodeClient};
use tonic::{metadata::MetadataValue, transport::Channel, Request};


use crate::sensei::{
    CloseChannelRequest, ConnectPeerRequest, CreateAdminRequest, CreateInvoiceRequest,
    CreateNodeRequest, GetUnusedAddressRequest, InfoRequest, KeysendRequest, ListChannelsRequest,
    ListNodesRequest, ListPaymentsRequest, ListPeersRequest, OpenChannelRequest, PayInvoiceRequest,
    SignMessageRequest, StartAdminRequest, StartNodeRequest, PaginationRequest
};

pub mod sensei {
    tonic::include_proto!("sensei");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("senseicli")
        .version("1.0")
        .author("John Cantrell <john@l2.technology>")
        .about("Control your sensei node from a cli")
        .arg(
            Arg::new("node")
                .short('n')
                .long("node")
                .value_name("NODE")
                .help("Sets the node to issue commands to")
                .takes_value(true),
        )
        .arg(
            Arg::new("token")
                .short('t')
                .long("token")
                .value_name("TOKEN")
                .help("Sets the admin token that will be used for admin requests")
                .takes_value(true),
        )
        .arg(
            Arg::new("host")
                .short('h')
                .long("host")
                .value_name("HOST")
                .help("Sets the host of the senseid instance")
                .takes_value(true),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("Sets the port of the senseid instance")
                .takes_value(true),
        )
        .arg(
            Arg::new("macaroon")
                .long("macaroon")
                .value_name("MACAROON")
                .help("The macaroon for use communicating with your instance")
                .takes_value(true),
        )
        .arg(
            Arg::new("dev")
                .long("dev")
                .value_name("DEV")
                .help("Whether to connect in development mode")
                .takes_value(true),
        )
        .arg(
            Arg::new("passphrase")
                .long("passphrase")
                .value_name("PASSPHRASE")
                .help("Passphrase (for testing, this is pretty insecure)")
                .takes_value(true),
        )
        .subcommand(
            App::new("init")
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
        .subcommand(App::new("start").about("unlock and start your sensei node"))
        .subcommand(App::new("listnodes").about("list all the lightning nodes"))
        .subcommand(
            App::new("createnode")
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
                )
                .arg(
                    Arg::new("start")
                        .required(true)
                        .index(3)
                        .possible_values(&["true", "false"])
                        .help("Whether or not to start the node"),
                ),
        )
        .subcommand(App::new("startnode").about("start a child lightning node"))
        .subcommand(App::new("getbalance").about("gets wallet's balance"))
        .subcommand(App::new("getaddress").about("get wallet's next unused address"))
        .subcommand(
            App::new("createinvoice")
                .about("create an invoice for an amount in msats")
                .arg(
                    Arg::new("amt_msat")
                        .required(true)
                        .index(1)
                        .help("amount in msats"),
                ),
        )
        .subcommand(
            App::new("openchannel")
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
            App::new("closechannel")
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
            App::new("payinvoice").about("pay an invoice").arg(
                Arg::new("invoice")
                    .required(true)
                    .index(1)
                    .help("bolt11 invoice"),
            ),
        )
        .subcommand(
            App::new("keysend")
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
            App::new("connectpeer")
                .about("connect to a peer on the lightning network")
                .arg(
                    Arg::new("node_connection_string")
                        .required(true)
                        .index(1)
                        .help("peer's connection string formatted pubkey@host:port"),
                ),
        )
        .subcommand(
            App::new("signmessage")
                .about("sign a message with your nodes key")
                .arg(
                    Arg::new("message")
                        .required(true)
                        .index(1)
                        .help("the message to be signed"),
                ),
        )
        .subcommand(App::new("listchannels").about("list channels"))
        .subcommand(App::new("listpayments").about("list payments"))
        .subcommand(App::new("listpeers").about("list peers"))
        .subcommand(App::new("nodeinfo").about("see information about your node"))
        .get_matches();

    let endpoint = if matches.is_present("dev") {
        "http://0.0.0.0:5401"
    } else {
        "http://0.0.0.0:3000"
    };

    let (command, command_args) = matches.subcommand().unwrap();

    let host = matches.value_of("host").unwrap_or("http://127.0.0.1");
    let port = matches.value_of("port").unwrap_or("5401");
    let endpoint = format!("{}:{}", host, port);

    let channel = Channel::from_shared(endpoint.to_string())?
            .connect()
            .await?;

    if command == "init" {
        // senseicli init username <node-alias> <start?>
        let mut admin_client = AdminClient::new(channel);

        let username = command_args.value_of("username").unwrap();
        let alias = command_args.value_of("alias").unwrap();

        let passphrase = set_passphrase(&matches);
        
        let request = tonic::Request::new(CreateAdminRequest {
            username: username.to_string(),
            alias: alias.to_string(),
            passphrase,
            start: false,
        });
        
        let result = admin_client.create_admin(request).await?;
        let message = result.into_inner();
        let response = messages::CreateAdminResponse {
            pubkey: message.pubkey,
            token: message.token,
            role: message.role,
            macaroon: message.macaroon,
            // external_id: message.external_id
        };
        println!("{}", serde_json::to_string(&response).unwrap());
    } else {
        let macaroon_hex_str = matches.value_of("macaroon").unwrap_or("");

        let token_str = matches.value_of("token").unwrap_or("");

        let macaroon = MetadataValue::from_str(&macaroon_hex_str)?;
        let admin_macaroon = macaroon.clone();
        
        let token = MetadataValue::from_str(&token_str)?;

        let mut client = NodeClient::with_interceptor(channel.clone(), move |mut req: Request<()>| {
            req.metadata_mut().insert("macaroon", macaroon.clone());
            Ok(req)
        });

        // This is only needed for a couple of queries.  Disabling pagination effectively by large get
        let pagination = PaginationRequest {
            page: 0,
            take: 1000,
            query: None,
        };

        let mut admin_client =
            AdminClient::with_interceptor(channel.clone(), move |mut req: Request<()>| {
                req.metadata_mut()
                    .insert("macaroon", admin_macaroon.clone());
                req.metadata_mut()
                    .insert("token", token.clone());
                Ok(req)
            });

        match command {
            // senseicli --token <> start
            "start" => {
                let passphrase = read_passphrase(&matches);

                let request = tonic::Request::new(StartAdminRequest { passphrase });
                let response = admin_client.start_admin(request).await?;
                println!("{:?}", response.into_inner());
            }
            
            // senseicli --token <> listnodes 
            "listnodes" => {
                let request = tonic::Request::new(ListNodesRequest { pagination: pagination.into() });
                let response = admin_client.list_nodes(request).await?;
                println!("{:?}", response.into_inner());
            }
            
            // senseicli --token <> createnode <username> <alias>
            "createnode" => {
                let username = command_args.value_of("username").unwrap();
                let alias = command_args.value_of("alias").unwrap();

                let start: bool = command_args
                    .value_of("start")
                    .expect("start field required")
                    .parse()
                    .expect("start must be true or false");


                let passphrase = set_passphrase(&matches);
                                
                let request = tonic::Request::new(CreateNodeRequest {
                    username: username.to_string(),
                    alias: alias.to_string(),
                    passphrase,
                    start: start,
                });
                let response = admin_client.create_node(request).await?;
                
                let message = response.into_inner();
                
                let mut config = NodeConfig {
                    data_dir: ".".into(),
                    pubkey: message.pubkey.clone(),
                    macaroon: message.macaroon.clone(),
                    token: "".into(),
                    role: Role::User.to_integer(),
                    // external_id: message.external_id.clone(),
                };
                config.save();

                println!("{}", serde_json::to_string(&config).unwrap());
            }
            

            // End Admin Commands

            "startnode" => {
                let passphrase = read_passphrase(&matches);

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
                let result = client.get_unused_address(request).await?;
                let response = messages::GetAddressResponse { address: result.into_inner().address.to_string() };

                println!("{}", serde_json::to_string(&response).unwrap());
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
                            let result = client.create_invoice(request).await?;
                            
                            let response = messages::CreateInvoiceResponse {invoice: result.into_inner().invoice.to_string()};
                            println!("{}", serde_json::to_string(&response).unwrap());
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

                let request = tonic::Request::new(OpenChannelRequest {
                    node_connection_string: node_connection_string.to_string(),
                    amt_satoshis,
                    public,
                });

                let response = client.open_channel(request).await?;
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
                let request = tonic::Request::new(ListChannelsRequest { pagination: pagination.into() });
                let response = client.list_channels(request).await?;
                println!("{:?}", response.into_inner());
            }
            "listpayments" => {
                let request = tonic::Request::new(ListPaymentsRequest {
                    pagination: pagination.into(),
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

fn read_passphrase(matches: &clap::ArgMatches) -> String {
    let passphrase_arg = matches.value_of("passphrase");
    match passphrase_arg {
        Some(p) => p.to_string(),
        None => {
            println!("Input the passphrase: ");
            let mut read = String::new();
            io::stdin().read_line(&mut read).expect("Reading passphrase failed");
            read
        },
    }
}

fn set_passphrase(matches: &clap::ArgMatches) -> String {
    let passphrase_arg = matches.value_of("passphrase");
    match passphrase_arg {
        Some(p) => p.to_string(),
        None => {
            println!("Set a passphrase: ");
            let mut read = String::new();
            io::stdin().read_line(&mut read).expect("Reading passphrase failed");
            read
        },
    }
}


#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct NodeConfig {
    pub data_dir: String,
    pub pubkey: String,
    pub macaroon: String,
    pub role: u8,
    pub token: String,
}

// impl Default for NodeConfig {
//     fn default() -> Self {
        
//         NodeConfig {
//             data_dir: ".".into(),
//             pubkey: "".into(),
//             macaroon: "".into(),
//             role: Role::Admin.to_integer(),
//             token: "satoshi".into(),
//         }
//     }
// }

impl NodeConfig {
    pub fn path(&self) -> String {
        format!("{}/data/{}", self.data_dir, self.pubkey)
    }

    pub fn save(&mut self) {
        fs::write(
            self.path().clone(),
            serde_json::to_string(&self).expect("failed to serialize config"),
        )
        .expect("failed to write config");
    }
}

// I could not figure out how to improt this from the database/admin
pub enum Role {
    Admin,
    User,
}

impl Role {
    pub fn to_integer(&self) -> u8 {
        match self {
            Role::Admin => 0,
            Role::User => 1,
        }
    }
}

// Messages
mod messages {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Serialize, Deserialize, Debug)]
    pub struct CreateAdminResponse {
        pub pubkey: String,
        pub macaroon: String,
        pub role: u32,
        pub token: String,
    }

    #[derive(Serialize)]
    pub struct CreateInvoiceResponse {
        pub invoice: String
    }

    #[derive(Serialize)]
    pub struct GetAddressResponse {
        pub address: String
    }

}