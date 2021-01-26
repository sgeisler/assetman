#!/usr/bin/env bash

set -e

cargo install $@ --path assetman
cargo install $@ --path plugins/assetman-bitcoin-holdings
cargo install $@ --path plugins/assetman-static
cargo install $@ --path plugins/assetman-bitstamp-price
cargo install $@ --path plugins/assetman-csv-scan
cargo install $@ --path plugins/assetman-metal-price