# Seahorse Escrow

Most DeFi protocols need to take custody of users fund. Our Escrow program demonstrates how to achieve that with **Seahorse**.

In an imaginary E-commerce scenario, the seller lists his his products or services with the dedicated price. If a buyer takes the deal, the seller would issue an escrow order to him with an assigned referee. The buyer then commits the deal by depositing funds to the order. When the product is fulfilled, he can instruct the order to release the funds to the seller. Otherwise, he can dispute to the referee and ask for a refund. 

Refer to the [Counter program](https://github.com/kenchan0824/sea-counter) for the Seahorse basics. Also, our test module uses the **Simple-Web3** module, you can read the docs [here](https://github.com/kenchan0824/simple-web3).

## Pre-requisites
- Solana 1.18.12
- Anchor 0.29.0
- Seahorse 0.2.0

## Setup
Download the sources to your local machines.
```
git clone https://github.com/kenchan0824/sea-escrow
```

Change to the project folder and install all packages.
```
npm i
```

## Play Around
1. Run the tests with `npm t`.

2. Inspect the sources in `programs_py/sea_escrow.py` and `tests/sea_escrow.ts`