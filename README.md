# Invest Toy: Toolkit for playing with my investment portfolio

The main feature right now is to rank all my stocks with a simple (and probably stupid) algorithm.
It also generates a table that help me decide which stocks to invest in once I get my salary.

This project consists of 2 parts:

- stock-metric-collector: A CLI program that extracts stock metrics from various sources
- stock-ranker: A CLI program that consumes the stock metrics and generates investment advice

## Build Instructions

The main entry point is [Ninja](https://ninja-build.org).
For build dependencies, consult [Dockerfile](./Dockerfile).

To build and install the programs, run:

```shell
ninja install
```

While `stock-ranker` is installed automatically by [Cargo](https://doc.rust-lang.org/stable/cargo),
`stock-metric-collector` is built at `./stock-metric-collector/build/` and must be installed manually.

## Usage

First, maintain a CSV containing your portfolio, named `portfolio.csv`, in the following format:

| Ticker | Type                                | iShares ID              | iShares Region              |
| ------ | ----------------------------------- | ----------------------- | --------------------------- |
|        | ("Stock" or "Exchange-Traded Fund") | (From product page URL) | ("United States" or "日本") |
| TSLA   | Stock                               |                         |                             |
| IVV    | Exchange-Traded Fund                | 239726                  | United States               |

Then, grab an API key from MarketStack and set it as an environment variable.

Then, run:

```shell
cat portfolio.csv | stock-metric-collector
```

This might need to run multiple times since Playwright is unpredictable.

Finally, run:

```shell
cat metrics.json | stock-ranker
```

Now you know which stocks to invest at what percentage of your available cash.

## Parameters

The CLI programs take these parameters from environment variables:

- MARKET_STACK_API_KEY: API key from [MarketStack](https://marketstack.com)
- PLAYWRIGHT_BROWSER: The [browser channel](https://playwright.dev/docs/api/class-browsertype#browser-type-launch-option-channel) used to scrap stock metrics.
- STOCK_METRIC_COLLECTOR_OUTPUT_DIRECTORY: Where to cache the scrapped result.
- STOCK_RANKER_INVEST_COUNT: How many stocks to invest in.
