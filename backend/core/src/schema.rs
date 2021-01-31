table! {
    experiments (id) {
        id -> Int4,
        user_id -> Int4,
        name -> Varchar,
        code -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    jobs (id) {
        id -> Int4,
        experiment_id -> Int4,
        runner_id -> Int4,
        code -> Text,
        output -> Nullable<Text>,
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
    runners (id) {
        id -> Int4,
        name -> Varchar,
        access_key -> Varchar,
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
joinable!(jobs -> experiments (experiment_id));
joinable!(jobs -> runners (runner_id));
joinable!(users -> roles (role_id));

allow_tables_to_appear_in_same_query!(
    experiments,
    jobs,
    roles,
    runners,
    users,
);
