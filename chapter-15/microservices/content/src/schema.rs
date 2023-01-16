// @generated automatically by Diesel CLI.

diesel::table! {
    comments (id) {
        id -> Integer,
        uid -> Text,
        text -> Text,
    }
}
