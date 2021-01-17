use diesel::expression::NonAggregate;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_builder::*;
use diesel::query_dsl::methods::LoadQuery;
use diesel::sql_types::BigInt;
use serde::{Deserialize, Serialize};

pub trait Paginate: Sized {
    fn paginate(self, page: Option<i64>) -> Paginated<Self>;
}

impl<T> Paginate for T {
    fn paginate(self, page: Option<i64>) -> Paginated<Self> {
        let page = match page {
            Some(num) => num,
            None => 1
        };

        Paginated {
            query: self,
            per_page: DEFAULT_PER_PAGE,
            page,
        }
    }
}

const DEFAULT_PER_PAGE: i64 = 10;

#[derive(Debug, Clone, Copy, QueryId)]
pub struct Paginated<T> {
    pub query: T,
    page: i64,
    per_page: i64,
}

impl<T> Paginated<T> {
    pub fn per_page(self, per_page: Option<i64>) -> Self {
        let per_page = match per_page {
            Some(num) => num,
            None => DEFAULT_PER_PAGE,
        };

        Paginated { per_page, ..self }
    }

    pub fn load_and_count_pages<U>(self, conn: &PgConnection) -> QueryResult<Pagination<U>>
        where
            Self: LoadQuery<PgConnection, (U, i64)>,
            U: Serialize,
    {
        let per_page = self.per_page;
        let page = self.page;

        let results = self.load::<(U, i64)>(conn)?;
        let total = results.get(0).map(|x| x.1).unwrap_or(0);
        let items = results.into_iter().map(|x| x.0).collect();
        let total_pages = (total as f64 / per_page as f64).ceil() as i64;
        Ok(Pagination {
            per_page,
            current_page: page,
            total_items: total,
            total_pages,
            items,
        })
    }
}

impl<T: Query> Query for Paginated<T> {
    type SqlType = T::SqlType;
}

impl<T> RunQueryDsl<PgConnection> for Paginated<T> {}

impl<T> QueryFragment<Pg> for Paginated<T>
    where
        T: QueryFragment<Pg>,
{
    fn walk_ast(&self, mut out: AstPass<Pg>) -> QueryResult<()> {
        self.query.walk_ast(out.reborrow())?;
        out.push_sql(" LIMIT ");
        out.push_bind_param::<BigInt, _>(&self.per_page)?;
        out.push_sql(" OFFSET ");
        let offset = (self.page - 1) * self.per_page;
        out.push_bind_param::<BigInt, _>(&offset)?;

        Ok(())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pagination<T> {
    per_page: i64,
    current_page: i64,
    total_pages: i64,
    total_items: i64,
    items: Vec<T>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationRequest {
    pub per_page: Option<i64>,
    pub page: Option<i64>,
}

pub struct CountStarOver;

impl Expression for CountStarOver {
    type SqlType = BigInt;
}

impl QueryFragment<Pg> for CountStarOver {
    fn walk_ast(&self, mut out: AstPass<Pg>) -> QueryResult<()> {
        out.push_sql("COUNT(*) OVER()");
        Ok(())
    }
}

impl<QS> AppearsOnTable<QS> for CountStarOver {}

impl<QS> SelectableExpression<QS> for CountStarOver {}

impl NonAggregate for CountStarOver {}

impl QueryId for CountStarOver {
    type QueryId = <Self as Expression>::SqlType;
}
