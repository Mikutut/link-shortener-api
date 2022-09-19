// @generated automatically by Diesel CLI.

diesel::table! {
    links (link_id) {
        link_id -> Varchar,
        target -> Text,
        control_key -> Varchar,
        added_at -> Timestamp,
        visit_count -> Integer,
    }
}
