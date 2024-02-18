// @generated automatically by Diesel CLI.

diesel::table! {
    dir (id) {
        id -> BigInt,
        name -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    file (id) {
        id -> BigInt,
        dir_id -> BigInt,
        name -> Text,
        file_type -> SmallInt,
        created_at -> Timestamp,
    }
}

diesel::table! {
    file_version (id) {
        id -> BigInt,
        file_id -> BigInt,
        size -> BigInt,
        version -> Text,
        state -> SmallInt,
        created_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    file_version_tag (id) {
        id -> BigInt,
        file_version_id -> BigInt,
        name -> Text,
        created_at -> Timestamp,
        activated_at -> Timestamp,
    }
}

diesel::joinable!(file -> dir (dir_id));
diesel::joinable!(file_version -> file (file_id));
diesel::joinable!(file_version_tag -> file_version (file_version_id));

diesel::allow_tables_to_appear_in_same_query!(
    dir,
    file,
    file_version,
    file_version_tag,
);
