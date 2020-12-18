table! {
    experiments (id) {
        id -> Int4,
        user_id -> Int4,
        name -> Varchar,
        status -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    roles (id) {
        id -> Int4,
        name -> Varchar,
    }
}

table! {
    users (id) {
        id -> Int4,
        first_name -> Varchar,
        last_name -> Varchar,
        email -> Varchar,
        password -> Varchar,
        status -> Varchar,
        role_id -> Int4,
    }
}

joinable!(experiments -> users (user_id));
joinable!(users -> roles (role_id));

allow_tables_to_appear_in_same_query!(
    experiments,
    roles,
    users,
);
