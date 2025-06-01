use std::borrow::BorrowMut;
use tracing::field::{Field, Visit};
use tracing::Level;
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

#[cfg(feature = "color-sql")]
use super::sql_highlighter::SqlHighlighter;
#[cfg(feature = "color-sql")]
use std::sync::LazyLock;
#[cfg(feature = "color-sql")]
static SQL_H: LazyLock<SqlHighlighter> = LazyLock::new(SqlHighlighter::new);

pub struct SqlxLayer;

impl<S> Layer<S> for SqlxLayer
where
    S: tracing::Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        if event.metadata().target() != "sqlx::query" {
            return;
        }

        let mut fields = SqlxQueryFields::default();

        let mut visitor = SqlxQueryVisitor {
            fields: &mut fields,
        };
        event.record(&mut visitor);

        if cfg!(feature = "color-sql") {
            fields.sql = SQL_H.highlight_sql(&fields.sql);
        };

        tracing::event!(
            target: "sqlx::formatted_query",
            Level::DEBUG, // or whatever level you want queries logged at
            sql = %fields.sql,
            rows_affected = fields.rows_affected,
            rows_returned = fields.rows_returned,
            elapsed = fields.elapsed,
            elapsed_secs = fields.elapsed_secs,
            unknown = fields.unknown,
        );
    }
}

#[derive(Debug, Default)]
struct SqlxQueryFields {
    sql: String,
    rows_affected: Option<usize>,
    rows_returned: Option<usize>,
    elapsed: Option<String>,
    elapsed_secs: Option<f64>,
    unknown: Option<String>,
}

struct SqlxQueryVisitor<'a> {
    fields: &'a mut SqlxQueryFields,
}

impl<'a> Visit for SqlxQueryVisitor<'a> {
    fn record_str(&mut self, field: &Field, value: &str) {
        let fields = self.fields.borrow_mut();
        match field.name() {
            "summary" => {}
            "db.statement" => fields.sql = value.to_string(),
            "rows_affected" => fields.rows_affected = value.parse().ok(),
            "rows_returned" => fields.rows_returned = value.parse().ok(),
            "elapsed" => fields.elapsed = Some(value.to_string()),
            "elapsed_secs" => fields.elapsed_secs = value.parse().ok(),
            _ => fields
                .unknown
                .get_or_insert(String::new())
                .push_str(&format!("\n{}={}", field.name(), value)),
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let fields = self.fields.borrow_mut();
        let value = format!("{value:?}");
        match field.name() {
            "summary" => {}
            "db.statement" => fields.sql = value,
            "rows_affected" => fields.rows_affected = value.parse().ok(),
            "rows_returned" => fields.rows_returned = value.parse().ok(),
            "elapsed" => fields.elapsed = Some(value),
            "elapsed_secs" => fields.elapsed_secs = value.parse().ok(),
            _ => fields
                .unknown
                .get_or_insert(String::new())
                .push_str(&format!("\n{}={:?}", field.name(), value)),
        }
    }
}
