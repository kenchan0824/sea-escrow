# Seahorse Escrow

Most DeFi protocols need to take custody of users fund. Our Escrow program demonstrates how to achieve that with **Seahorse**.

This project is motivated by an imaginary E-commerce scenario, where the seller lists his his products or services with the dedicated price. 
- If a buyer takes the deal, the seller would issue an escrow order to him with an assigned referee. 
- The buyer then commits the deal by depositing funds to the order. 
- When the product is fulfilled, he can instruct the order to release the funds to the seller. 
- Otherwise, he can dispute to the referee and ask for a refund. 

![escrow](./escrow.jpg)

> Refer to the [Counter program](https://github.com/kenchan0824/sea-counter) for the Seahorse basics. Also, our test module uses the **Simple-Web3** module, you can read the docs [here](https://github.com/kenchan0824/simple-web3).

## Pre-requisites
- Solana 1.18.12
- Anchor 0.29.0
- Seahorse 0.2.0

## Steps to run
[1] Clone the repository to your local machines.
```
git clone https://github.com/kenchan0824/sea-escrow
```

[2] Go to the project folder and install all packages.
```
npm i
```

[3] Pre-build Anchor
```
anchor build
```

[4] Run tests
```
npm test
```

[5] You may do some experiments with the codes inside `programs_py/sea_escrow.py` and `tests/sea_escrow.ts`