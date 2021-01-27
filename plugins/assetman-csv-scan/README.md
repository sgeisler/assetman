# CSV Scan Plugin

* Name: `csv_scan`
* Arguments: comma-separated list of
  * CSV file path
  * index of the search column
  * search term to find in the search column
  * index of cell to extract (it also replaces commas with dots, because German fractions … should probably be optional)
* Example: `csv_scan(/home/user/depot.csv,3,MY_TICKER,12)`