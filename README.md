# Inventory Management System

This project is a decentralized platform built on the Internet Computer for managing inventory, suppliers, and orders. It allows users to create, update, retrieve, and delete inventory items, suppliers, and orders, ensuring robust and efficient management of inventory data.

## Key Features

1. **Item Management**
   - **Create Item:** Allows users to create inventory items.
   - **Get All Items:** Retrieve a list of all inventory items.
   - **Get Item by ID:** Retrieve a specific item by its ID.
   - **Update Item:** Update the details of an existing inventory item.
   - **Delete Item:** Remove an inventory item from the system.
   - **Search Items by Name:** Search for items by their name.
   - **Count Items:** Get the total number of items.

2. **Supplier Management**
   - **Create Supplier:** Allows users to create supplier profiles.
   - **Get All Suppliers:** Retrieve a list of all supplier profiles.
   - **Get Supplier by ID:** Retrieve a specific supplier by its ID.
   - **Update Supplier:** Update the details of an existing supplier.
   - **Delete Supplier:** Remove a supplier from the system.
   - **Count Suppliers:** Get the total number of suppliers.

3. **Order Management**
   - **Create Order:** Allows users to create orders for inventory items.
   - **Get All Orders:** Retrieve a list of all orders.
   - **Update Order:** Update the details of an existing order.
   - **Delete Order:** Remove an order from the system.
   - **Filter Orders by Supplier:** Retrieve orders filtered by supplier ID.
   - **Count Orders:** Get the total number of orders.

4. **Error Handling**
   - **Invalid Payload:** Returns an error if the input payload is invalid.
   - **Not Found:** Returns an error if a requested item, supplier, or order is not found.

## Requirements
* rustc 1.64 or higher
```bash
$ curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
$ source "$HOME/.cargo/env"
```
* rust wasm32-unknown-unknown target
```bash
$ rustup target add wasm32-unknown-unknown
```
* candid-extractor
```bash
$ cargo install candid-extractor
```
* install `dfx`
```bash
$ DFX_VERSION=0.15.0 sh -ci "$(curl -fsSL https://sdk.dfinity.org/install.sh)"
$ echo 'export PATH="$PATH:$HOME/bin"' >> "$HOME/.bashrc"
$ source ~/.bashrc
$ dfx start --background
```

If you want to start working on your project right away, you might want to try the following commands:

```bash
$ cd icp_rust_boilerplate/
$ dfx help
$ dfx canister --help
```

## Update dependencies

update the `dependencies` block in `/src/{canister_name}/Cargo.toml`:
```
[dependencies]
candid = "0.9.9"
ic-cdk = "0.11.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1.0"
ic-stable-structures = { git = "https://github.com/lwshang/stable-structures.git", branch = "lwshang/update_cdk"}
```

## did autogenerate

Add this script to the root directory of the project:
```
https://github.com/buildwithjuno/juno/blob/main/scripts/did.sh
```

Update line 16 with the name of your canister:
```
https://github.com/buildwithjuno/juno/blob/main/scripts/did.sh#L16
```

After this run this script to generate Candid.
Important note!

You should run this script each time you modify/add/remove exported functions of the canister.
Otherwise, you'll have to modify the candid file manually.

Also, you can add package json with this content:
```
{
    "scripts": {
        "generate": "./did.sh && dfx generate",
        "gen-deploy": "./did.sh && dfx generate && dfx deploy -y"
      }
}
```

and use commands `npm run generate` to generate candid or `npm run gen-deploy` to generate candid and to deploy a canister.

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
$ dfx start --background

# Deploys your canisters to the replica and generates your candid interface
$ dfx deploy
```