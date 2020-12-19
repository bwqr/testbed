table! {
    experiments (id) {
        id -> Int4,
        user_id -> Int4,
        name -> Varchar,
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
    runners (id) {
        id -> Int4,
        access_key -> Varchar,
        created_at -> Timestamp,
    }
}

table! {
    runs (id) {
        id -> Int4,
        experiment_id -> Int4,
        status -> Varchar,
        created_at -> Timestamp,
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
joinable!(runs -> experiments (experiment_id));
joinable!(users -> roles (role_id));

allow_tables_to_appear_in_same_query!(
    experiments,
    roles,
    runners,
    runs,
    users,
);
