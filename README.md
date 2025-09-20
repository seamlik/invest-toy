# Invest Toy: Toolkit for playing with my investment portfolio

The main feature right now is to rank all my stocks with a simple (and probably stupid) algorithm.
It also generates a table that help me decide which stocks to invest in once I get my salary.

This project consists of 2 parts:

- A web browser extension (tested on [Microsoft Edge](https://microsoft.com/edge))
  that extracts stock metrics from [Yahoo Finance](https://finance.yahoo.com)
- A CLI program that consumes the stock metrics and generates investment advice.

## Build Instructions

The main entry point is [Ninja](https://ninja-build.org).
For build dependencies, consult [Dockerfile](./Dockerfile).

### Web Extension

Run:

```shell
ninja web-extension
```

The output is stored at directory `web-extension/build/`.
Now, follow the [instructions](https://learn.microsoft.com/microsoft-edge/extensions-chromium/getting-started/extension-sideloading) to sideload the extension into your browser.

### CLI Program

Run:

```shell
ninja install-cli
```

The CLI program `stock-ranker` is now usable.

## Usage

1. Create a Yahoo account
2. Add the stocks you like to a portfolio
3. Click on the "Invest Toy" extension button
4. Wait for it to finish and download the stock metrics as a JSON file
5. Run `cat stock-metrics-XXX.json | stock-ranker`

Now you know which stock to invest at what percentage of your available cash.

## Parameters

This CLI program takes parameters from environment variables:

- PLAYWRIGHT_BROWSER: The [browser channel](https://playwright.dev/docs/api/class-browsertype#browser-type-launch-option-channel) used to scrap stock metrics.
- STOCK_RANKER_INVEST_COUNT: How many stocks to invest in.
