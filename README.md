# FrontEraning Solana Program
---

This program implements **delayed settlement into instant discounts** using Perena's USD* basket and Numeraire AMM.

* *Buyer* pays with USD*/USDC/USDT, gets an immediate discount.
* *Seller* waits an agreed period (default 1 year) and receives the principal + half of the USD* yield.
* Borrower supplies the small "gap" capital(like 2 USD) locks for the same period, and later withdraws their principal + the other half or more of the yield.

## Overview


## Instructions

1. init_config

Owner bootstrap - set per-token discount BPS and default periods. `usdc_discount`, `usdt_discount`, `usd_star_discount`. (u16, in bps)

2. update_config

Owner runtime update of the same fields + `settle_wait_secs`.

3. init_payment

Seller lists a product. Seeds = `[b"payment", seller, price]`. Args = `price` (u64, 6 decimals).
Initial status = `Initialized`.

4. execute_payment

Buyer pays `amount`. Transfers discounted amount to vault. If not already USD*, Numeraire `swap_exact_in` CPI call swaps to USD*. And records `paid_amount`, `paid_mint`, `start_ts`.
After payment, status will be set to `Funded`.

5. invest_gap

Borrower locks any USD*/USDC/USDT gap capital. PDA seeds [b"investment", borrower].
Status => Locked.

6. settle

Callable by seller after `settle_wait_secs`. Status => Settled.

7. withdraw_investment

Borrower withdraws after `invest_lock_secs` (default 1 year). Status => Withdrawn.

## Flow

## Build & Test

1. Before build

```sh
brew install rustup
rustup default stable
npm i -g @coral-xyz/anchor-cli
```

2. Build

```sh
anchor build
```

3. Local test

```sh
anchor test
```

4. Devnet deploy

```sh
anchor deploy --provider.cluster devnet --program-name front-earning-program
```

5. Verify

```sh
solana address -k target/deploy/front-earning-program-keypair.json
solana account <program-id>
```