// @generated automatically by Diesel CLI.

diesel::table! {
    expense_records (uuid) {
        datetime_created -> TimestamptzSqlite,
        uuid -> Binary,
        amount -> Integer,
        datetime -> TimestamptzSqlite,
        description -> Nullable<Text>,
        description_container -> Binary,
        tags -> Text,
        origin -> Text,
        data -> Binary,
    }
}

diesel::table! {
    links (uuid) {
        uuid -> Binary,
        negative -> Binary,
        positive -> Binary,
    }
}

diesel::table! {
    possible_links (uuid) {
        uuid -> Binary,
        negative -> Binary,
        positive -> Binary,
        probability -> Double,
    }
}

diesel::table! {
    profiles (uuid) {
        uuid -> Binary,
        name -> Text,
        origin_name -> Text,
        data -> Binary,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    expense_records,
    links,
    possible_links,
    profiles,
);
