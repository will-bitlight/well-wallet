# well-wallet
well-wallet is primarily about learning to verify Bitcoin knowledge

```

## Initialize wallet
$ ./target/debug/well-wallet -s "/tmp/well" new-wallet 

## Create address
$ ./target/debug/well-wallet -s "/tmp/well" create-address

## Get balance
$ ./target/debug/well-wallet -s "/tmp/well" get-balance

## List transactions
$ ./target/debug/well-wallet -s "/tmp/well" list-transactions

## Send transaction
$ ./target/debug/well-wallet -s "/tmp/well" pay -r bcrt1qwwf3ckm89aqxzpxhp62ee65s75kn7fnuk0y82g -a 10000

```