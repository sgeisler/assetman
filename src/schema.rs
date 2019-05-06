table! {
    assets {
        id -> Integer,
        name -> Text,
        description -> Nullable<Text>,
        quandl_database -> Nullable<Text>,
        quandl_dataset -> Nullable<Text>,
        quandl_price_idx -> Nullable<Integer>,
        category -> Text,
    }
}

table! {
    updates (asset_id, timestamp) {
        asset_id -> Integer,
        timestamp -> Timestamp,
        holdings -> Float,
    }
}

table! {
    prices (asset_id, timestamp) {
        asset_id -> Integer,
        timestamp -> Timestamp,
        price -> Float,
    }
}

joinable!(updates -> assets (asset_id));
joinable!(prices -> assets (asset_id));
allow_tables_to_appear_in_same_query!(assets, prices, updates);

