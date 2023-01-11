// @generated automatically by Diesel CLI.

diesel::table! {
    comments (id) {
        id -> Nullable<Integer>,
        uid -> Text,
        text -> Text,
    }
}
