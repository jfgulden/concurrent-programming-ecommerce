[![Review Assignment Due Date](https://classroom.github.com/assets/deadline-readme-button-24ddc0f5d75046c5622901739e7c5dd533143b0c8e959d652212380cedb1ea36.svg)](https://classroom.github.com/a/AdDZ0HGe)
[![Open in Visual Studio Code](https://classroom.github.com/assets/open-in-vscode-718a45dd9cf7e7f842a935f5ebbe5719a5e09af4491e668f4dbf3b35d5cca122.svg)](https://classroom.github.com/online_ide?assignment_repo_id=12589822&assignment_repo_type=AssignmentRepo)

# 23C2-Rust-eze

Repo for Concurrentes FIUBA

## Files Creation

Before compiling and running the program, we must create some files:

### Orders
- an pedidos/[ecom_orders_filename].txt for each ecom, which will contains all the online orders, with the following format:

```
<ecom_id>
----------------------
<order1_product_name>,<quantity>,<purchase_zone>
<order2_product_name>,<quantity>,<purchase_zone>
...
<orderN_product_name>,<quantity>,<purchase_zone>
```
An example of it is shown at pedidos/ecom1.txt

- A pedidos/[local_orders_file].txt for each shop, which will contains all the local orders, with the following format:

```
<order1_product_name>,<quantity>
<order2_product_name>,<quantity>
...
<orderN_product_name>,<quantity>
```
An example of it is shown at pedidos/tienda1.txt

### Shops

- A tiendas/[local_shop_filename].txt for each shop, which will contains all the local stock, with the following format:

```
<shop_zone_name>,<shop_address:port>,<shop_zone_id>
----------------------
<stock1_product_name>,<quantity>
<stock2_product_name>,<quantity>
...
<stockN_product_name>,<quantity>
```
An example of it is shown at tiendas/tienda1.txt

## Compile and run

First, we should run the shop binary:

```
cargo run --bin shop [shop_filename]
```

Then, we should run the ecom binary:

```
cargo run --bin [ecom_orders_filename]
```

If we do so, the shop will start listening for online orders while it processes local orders, and the ecom will be sending those online orders to the shop.
If we run the ecom but we don't run any shops, the ecom will try to send the orders and it won't be able, so all of them will be rejected.

## Ecom shortcuts

We can disconnect shops from the ecom by pressing 's[shop_zone_id]', and pressing the enter key.
We can try to reconnect to a shop from the ecom by pressing 'r[shop_zone_id]', and pressing the enter key.

For example:
    
    ``` s1 ```
    Disconnects shop 1 from the ecom
    ``` r1 ```
    Tries to (re)connect shop 1 to the ecom

With these shortcuts we can play with the execution orders of the ecom and the shops, and see how they behave.
