# Assetman

Assetman is a tool to track prices and holdings of owned assets to calculate a (rough) total net worth. It uses a plugin system
to be able to support every conceivable data source. It already comes with a few plugins described below but can also be
easily extended.

The code isn't very good, mostly because it's the third forced rewrite due to a data provider not being sufficient
anymore. Any efforts to clean up the code are welcome. But since it mostly works for me I'll probably only rewrite parts
I need to touch anyway be it because of annoying bugs or to add features. Don't rely on it for anything critical. It
uses floating point arithmetic, which is a no-go for most financial software, but ok for me as I only wanted to know a
rough estimate anyway and ain't that rich (even if, who cares if they own 1ct more or less).

## Configuration
Currently all configuration happens via environment variables, either set directly or via an `.env` file. Eventually a
real config file would be nice.

```bash
# Absolute database path, the database is created on first run
AM_DATABASE="/path/to/database.db"
# Colon-sparated list of plugins, either their absolute path or just the name if they are in $PATH
AM_PLUGINS="assetman-static:assetman-bitcoin-holdings:assetman-bitstamp-price:assetman-csv-scan:assetman-metal-price"
# Plugin-specific config, e.g. the electrum server to use for the bitcoin holdings plugin
AM_ELECTRUM_SERVER="ssl://my.electrum.server:50002"
```  

## Commands
Assetman has three main functions:

* `add`: Adds a new asset/account to be tracked it requires a *name* for the account, a *price query* and a *holdings
query* referencing a certain plugin to fetch price data or holding amounts from and lastly a *category* which allows
grouping of the accounts. An added asset will only show up after a successful fetch.
* `fetch`: Fetches the current price and holding amount for each asset/account. If anything fails during that the
operation is aborted and no data is written to the database. So it is safe to just re-run fetch till it works (some
plugins are buggy or depend on external services that might error but work the next time).
* `list` Lists all assets/accounts with their prices and total value. There are two options: `-c` group by category and
`-v` sort by value. If used together the categories aren't explicitly sorted again currently.

## Plugins

The plugin API is quite primitive and too stringly typed for my taste, but I needed the flexibility. I hope to refactor
it to be more rustic eventually.

The currently provided plugins are (see their respective `README` for docs):

* [bitcoin-holdings](plugins/assetman-bitcoin-holdings)
* [bitstamp-price](plugins/assetman-bitstamp-price)
* [csv-scan](plugins/assetman-csv-scan)
* [metal-price](plugins/assetman-metal-price)
* [static](plugins/assetman-static)

## Contributing
Patches are welcome, feature demands not so much (leave feature ideas as issues if you like, but don't expect me to work on them
except if you pay me for it). Some ideas for future expansion:

* Make the system aware of its base currency to avoid errors
* Make the plugin API more semantic (precursor for the next idea imo) 
* Make price queries chainable, so e.g. `XAU/USD -> USD/EUR` will effectively become `XAU/EUR` so that not-directly-supported
currencies can be used as base currency

There's no code of conduct, just don't be a jerk or you'll be shown the door.