table! {
    assets {
        id -> Integer,
        name -> Text,
        description -> Nullable<Text>,
        query -> Text,
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

