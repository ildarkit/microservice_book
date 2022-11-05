// @generated automatically by Diesel CLI.

diesel::table! {
    channels (id) {
        id -> Int4,
        user_id -> Int4,
        title -> Text,
        is_public -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    memberships (id) {
        id -> Int4,
        channel_id -> Int4,
        user_id -> Int4,
    }
}

diesel::table! {
    messages (id) {
        id -> Int4,
        timestamp -> Timestamp,
        channel_id -> Int4,
        user_id -> Int4,
        text -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        email -> Text,
    }
}

diesel::joinable!(channels -> users (user_id));
diesel::joinable!(memberships -> channels (channel_id));
diesel::joinable!(memberships -> users (user_id));
diesel::joinable!(messages -> channels (channel_id));
diesel::joinable!(messages -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    channels,
    memberships,
    messages,
    users,
);
