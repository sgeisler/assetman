CREATE TABLE assets (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  price_query TEXT NOT NULL,
  holdings_query TEXT NOT NULL,
  category TEXT DEFAULT 'default'
);

CREATE TABLE updates (
  id INTEGER PRIMARY KEY,
  timestamp INTEGER DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE holdings (
  update_id INTEGER REFERENCES updates (id) NOT NULL,
  asset_id INTEGER REFERENCES assets (id) NOT NULL,
  amount REAL NOT NULL,
  PRIMARY KEY (asset_id, update_id)
);

CREATE TABLE prices (
  update_id INTEGER REFERENCES updates (id) NOT NULL,
  asset_id INTEGER REFERENCES assets (id) NOT NULL,
  price REAL NOT NULL,
  PRIMARY KEY (asset_id, update_id)
);
