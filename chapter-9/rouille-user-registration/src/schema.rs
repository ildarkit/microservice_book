// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Text,
        email -> Text,
        password -> Text,
    }
}
