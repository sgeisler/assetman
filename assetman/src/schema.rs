table! {
    assets {
        id -> Integer,
        name -> Text,
        price_query -> Text,
        holdings_query -> Text,
        category -> Text,
    }
}

table! {
    updates {
        id -> Integer,
        timestamp -> Timestamp,
    }
}

table! {
    holdings (update_id, asset_id) {
        update_id -> Integer,
        asset_id -> Integer,
        amount -> Double,
    }
}

table! {
    prices (update_id, asset_id) {
        update_id -> Integer,
        asset_id -> Integer,
        price -> Double,
    }
}

joinable!(prices -> updates (update_id));
joinable!(prices -> assets (asset_id));
joinable!(holdings -> updates (update_id));
joinable!(holdings -> assets (asset_id));
allow_tables_to_appear_in_same_query!(assets, prices, holdings);
