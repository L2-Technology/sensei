![Sensei Logo](./web-admin/public/images/sensei-logo.svg)

### **WARNING: This software is in beta. Please do not use it with a lot of funds.

<br/>

Sensei is a new lightning node implementation with a focus on easing the onboarding experience for users new to Bitcoin. It is built using the [bitcoin development kit](https://bitcoindevkit.org) and the [lightning development kit](https://lightningdevkit.org).

## Dependencies

You will need a bitcoind instance to use Sensei.

I recommend using [Nigiri](https://nigiri.vulpem.com/) to get everything running locally.

## Building and running from source

To run from source you will need to take the following steps:

1. Clone the repo: `git clone git@github.com:L2-Technology/sensei.git`
2. Build the web-admin: `cd sensei/web-admin && npm install && npm run build && cd ..`
3. Run senseid on regtest: `cargo run --bin senseid -- --network=regtest --bitcoind-rpc-host=localhost --bitcoind-rpc-port=18443 --bitcoind-rpc-username=admin1 --bitcoind-rpc-password=123 --database-url=sensei.db`
4. Open the admin at `http://localhost:5401/admin/nodes`

## Database Backends

Sensei supports `sqlite`, `mysql`, and `postgres` databases.  You can configure what database to use by specifying a `DATABASE_URL` environment varilable or `--database-url` command line argument.

### Sqlite

For sqlite you just specify the filename to use for the database.  It will be saved in the Sensei data directory.

Example: `--database-url=sensei.db`

### Postgres & MySQL

Sensei includes a `docker-compose.yml` file that can automatically run these databases for you locally.  Feel free to use these or just substitute the credentials to whatever database you already have.

Starting docker based databases: `docker-compose up -d`

Postgres Example: `--database-url=postgres://sensei:sensei@localhost:5432/sensei`
MySQL Example: `--database-url=mysql://sensei:sensei@localhost:3306/sensei`


## Developing the web-admin

In order to see your changes live you will need to:

1. Allow requests from local web-admin: `cargo run --bin senseid -- --network=regtest --bitcoind-rpc-host=localhost --bitcoind-rpc-port=18443 --bitcoind-rpc-username=admin1 --bitcoind-rpc-password=123 --allow-origins=http://localhost:3001`
2. Run the web-admin dev server: `cd sensei/web-admin && npm install && npm run start`
3. Visit the admin using port 3001: `http://localhost:3001/admin/nodes`

## Using with Nigiri

[Nigiri](https://nigiri.vulpem.com/) is a great tool for running local docker images of bitcoind, electrum, esplora, and much more. Once it's running you can use the bitcoind instance it provides when starting up your Sensei node.

Once your node is setup you can:

1. Visit the 'Fund Node' page in the Sensei admin to get an unused receive address.
2. Send 100M sats to your Sensei node via: `nigiri faucet <sensei_fund_address>`
3. After you open a channel you can mine blocks using nigiri by:
   - Getting an address to mine to `nigiri rpc getnewaddress "" "bech32"`
   - Mine some blocks to that address `nigiri rpc generatetoaddress 10 "<address_from_previous_command>"`

## Data directory

You can pass a custom data directory using --data_dir flag but the default will be a `.sensei` directory in your operating systems home directory.

Home directory is retrieved using the [dirs crate](https://github.com/dirs-dev/dirs-rs).

| Platform | Value                | Example                |
| -------- | -------------------- | ---------------------- |
| Linux    | `$HOME`              | /home/alice/.sensei    |
| macOS    | `$HOME`              | /Users/Alice/.sensei   |
| Windows  | `{FOLDERID_Profile}` | C:\Users\Alice\.sensei |

### Linux and macOS:

- Use `$HOME` if it is set and not empty.
- If `$HOME` is not set or empty, then the function `getpwuid_r` is used to determine
  the home directory of the current user.
- If `getpwuid_r` lacks an entry for the current user id or the home directory field is empty,
  then the function returns `None`.

### Windows:

This function retrieves the user profile folder using `SHGetKnownFolderPath`.

## Configuration Files

Sensei will create a root `config.json` file inside the data directory. These are configurations that will be applied across all networks.

Sensei will also create subdirectories for each network (e.g. mainnet, testnet, regtest) that you instantiate the daemon with. Each network subdirectory will have it's own `config.json` file.

Sensei will merge the network specific configuration into the root configuration to create the final configuration.

This means any configuration set at the network level will override configuration at the root level.

## Command Line Args & Environment Variables

Some of the configuration options can be set using command line arguments or environment variables.

These will have the highest precedence and overwrite the network specific configuration.

instance > network > root

## Documentation

Please visit the [documentation website](https://docs.l2.technology) for installation and getting started instructions.

## Community

Please join [our discord community](https://discord.gg/bneS492Tqu) to discuss anything related to this project.
