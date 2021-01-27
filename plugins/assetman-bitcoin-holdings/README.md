# Bitcoin Holdings Plugin

* Name: `bitcoin_h`
* Arguments: semicolon separated list of miniscript descriptors. If these contain xpubs the plugin will automatically
derive both internal and external scripts. It uses a fixed gap limit of 10, which should be made a config parameter.
* Example: `bitcoin_h(sh(wpkh(xpub…/*));wsh(sortedmulti(2,xpub…/*,xpub…/*,xpub…/*)))`