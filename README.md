![Sensei Logo](./web-admin/public/images/sensei-logo.svg)


### **WARNING: This software is in beta.  Do not use it on mainnet until this warning is removed.  Expect breaking changes to the api and database schema until the 0.1.0 release.**

<br/>

Sensei is a new lightning node implementation with a focus on easing the onboarding experience for users new to Bitcoin. It is built using the [bitcoin development kit](https://bitcoindevkit.org) and the [lightning development kit](https://lightningdevkit.org).

## Running from source

To run from source you will need to take the following steps:

1. Clone the repo: `git clone git@github.com:L2-Technology/sensei.git`
2. Build the web-admin: `cd sensei/web-admin && npm install && npm run build && cd ..`
3. Run senseid on regtest: `cargo run --bin senseid -- --network=regtest`
4. Open the admin at `http://localhost:5401/admin/nodes`


## Developing the web-admin

In order to see your changes live you will need to:

1. Run the web-admin dev server: `cd sensei/web-admin && npm run start`
2. Visit the admin using port 3000: `http://localhost:3000/admin/nodes`

## Documentation

Please visit the [documentation website](https://docs.l2.technology) for installation and getting started instructions.  

## Community

Please join [our discord community](https://discord.gg/bneS492Tqu) to discuss anything related to this project.