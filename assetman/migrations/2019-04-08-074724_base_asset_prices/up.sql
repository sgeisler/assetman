CREATE TABLE assets (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT,
  quandl_database TEXT,
  quandl_dataset TEXT,
  quandl_price_idx INTEGER,
  category TEXT default 'default',
  -- Either a quandl price source is defined or not (in case of base currency)
  CONSTRAINT quandl_all_or_nothing CHECK (
    ((quandl_database IS NULL) = (quandl_dataset IS NULL)) AND
    ((quandl_dataset IS NULL) = (quandl_price_idx IS NULL))
  )
);

CREATE TABLE updates (
  asset_id INTEGER REFERENCES assets (id) NOT NULL,
  timestamp INTEGER DEFAULT CURRENT_TIMESTAMP NOT NULL ,
  holdings REAL NOT NULL,
  PRIMARY KEY (asset_id, timestamp)
);

CREATE TABLE prices (
  asset_id INTEGER REFERENCES assets (id) NOT NULL,
  timestamp INTEGER DEFAULT CURRENT_TIMESTAMP NOT NULL,
  price REAL NOT NULL,
  PRIMARY KEY (asset_id, timestamp)
);
