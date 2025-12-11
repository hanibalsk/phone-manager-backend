//! Data Subject Request repository for database operations.
//!
//! AP-11.4-6: GDPR/CCPA Data Subject Request management

use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::entities::{
    DataSubjectRequestEntity, DataSubjectRequestStatusDb, DataSubjectRequestTypeDb,
    DataSubjectRequestWithProcessorEntity,
};

/// Default due date is 30 days from creation (GDPR requirement).
pub const DEFAULT_DUE_DAYS: i64 = 30;

/// Input for creating a new data subject request.
#[derive(Debug, Clone)]
pub struct CreateDataSubjectRequestInput {
    pub organization_id: Uuid,
    pub request_type: DataSubjectRequestTypeDb,
    pub subject_email: String,
    pub subject_name: Option<String>,
    pub subject_user_id: Option<Uuid>,
    pub description: Option<String>,
    pub due_days: Option<i64>,
}

/// Input for processing a data subject request.
#[derive(Debug, Clone)]
pub struct ProcessDataSubjectRequestInput {
    pub status: DataSubjectRequestStatusDb,
    pub processed_by: Uuid,
    pub rejection_reason: Option<String>,
    pub result_data: Option<serde_json::Value>,
    pub result_file_url: Option<String>,
    pub result_expires_days: Option<i64>,
}

/// Query parameters for listing data subject requests.
#[derive(Debug, Clone, Default)]
pub struct ListDataSubjectRequestsQuery {
    pub status: Option<DataSubjectRequestStatusDb>,
    pub request_type: Option<DataSubjectRequestTypeDb>,
    pub subject_email: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

/// Summary counts for compliance dashboard.
#[derive(Debug, Clone, Default)]
pub struct DataSubjectRequestCounts {
    pub total: i64,
    pub pending: i64,
    pub in_progress: i64,
    pub completed: i64,
    pub rejected: i64,
    pub overdue: i64,
}

/// Repository for data subject request database operations.
#[derive(Clone)]
pub struct DataSubjectRequestRepository {
    pool: PgPool,
}

impl DataSubjectRequestRepository {
    /// Create a new repository instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new data subject request.
    pub async fn create(
        &self,
        input: CreateDataSubjectRequestInput,
    ) -> Result<DataSubjectRequestEntity, sqlx::Error> {
        let due_days = input.due_days.unwrap_or(DEFAULT_DUE_DAYS);
        let due_date = Utc::now() + Duration::days(due_days);

        let entity = sqlx::query_as::<_, DataSubjectRequestEntity>(
            r#"
            INSERT INTO data_subject_requests (
                organization_id, request_type, subject_email, subject_name,
                subject_user_id, description, due_date
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(input.organization_id)
        .bind(input.request_type)
        .bind(&input.subject_email)
        .bind(&input.subject_name)
        .bind(input.subject_user_id)
        .bind(&input.description)
        .bind(due_date)
        .fetch_one(&self.pool)
        .await?;

        Ok(entity)
    }

    /// Find a data subject request by ID.
    pub async fn find_by_id(
        &self,
        org_id: Uuid,
        id: Uuid,
    ) -> Result<Option<DataSubjectRequestWithProcessorEntity>, sqlx::Error> {
        let entity = sqlx::query_as::<_, DataSubjectRequestWithProcessorEntity>(
            r#"
            SELECT
                dsr.*,
                u.email as processor_email
            FROM data_subject_requests dsr
            LEFT JOIN users u ON u.id = dsr.processed_by
            WHERE dsr.id = $1 AND dsr.organization_id = $2
            "#,
        )
        .bind(id)
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity)
    }

    /// List data subject requests with pagination and filtering.
    pub async fn list(
        &self,
        org_id: Uuid,
        query: &ListDataSubjectRequestsQuery,
    ) -> Result<(Vec<DataSubjectRequestWithProcessorEntity>, i64), sqlx::Error> {
        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(50).clamp(1, 100);
        let offset = ((page - 1) * per_page) as i64;

        // Build dynamic WHERE clause
        let mut conditions = vec!["dsr.organization_id = $1".to_string()];
        let mut param_count = 1;

        if query.status.is_some() {
            param_count += 1;
            conditions.push(format!("dsr.status = ${}", param_count));
        }

        if query.request_type.is_some() {
            param_count += 1;
            conditions.push(format!("dsr.request_type = ${}", param_count));
        }

        if query.subject_email.is_some() {
            param_count += 1;
            conditions.push(format!(
                "LOWER(dsr.subject_email) LIKE LOWER(${})",
                param_count
            ));
        }

        if query.from.is_some() {
            param_count += 1;
            conditions.push(format!("dsr.created_at >= ${}", param_count));
        }

        if query.to.is_some() {
            param_count += 1;
            conditions.push(format!("dsr.created_at <= ${}", param_count));
        }

        let where_clause = conditions.join(" AND ");

        // Count query
        let count_sql = format!(
            "SELECT COUNT(*) FROM data_subject_requests dsr WHERE {}",
            where_clause
        );
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql).bind(org_id);

        if let Some(ref status) = query.status {
            count_query = count_query.bind(status);
        }
        if let Some(ref request_type) = query.request_type {
            count_query = count_query.bind(request_type);
        }
        if let Some(ref email) = query.subject_email {
            count_query = count_query.bind(format!("%{}%", email));
        }
        if let Some(ref from) = query.from {
            count_query = count_query.bind(from);
        }
        if let Some(ref to) = query.to {
            count_query = count_query.bind(to);
        }

        let total = count_query.fetch_one(&self.pool).await?;

        // List query
        let list_sql = format!(
            r#"
            SELECT
                dsr.*,
                u.email as processor_email
            FROM data_subject_requests dsr
            LEFT JOIN users u ON u.id = dsr.processed_by
            WHERE {}
            ORDER BY dsr.created_at DESC
            LIMIT ${} OFFSET ${}
            "#,
            where_clause,
            param_count + 1,
            param_count + 2
        );

        let mut list_query =
            sqlx::query_as::<_, DataSubjectRequestWithProcessorEntity>(&list_sql).bind(org_id);

        if let Some(ref status) = query.status {
            list_query = list_query.bind(status);
        }
        if let Some(ref request_type) = query.request_type {
            list_query = list_query.bind(request_type);
        }
        if let Some(ref email) = query.subject_email {
            list_query = list_query.bind(format!("%{}%", email));
        }
        if let Some(ref from) = query.from {
            list_query = list_query.bind(from);
        }
        if let Some(ref to) = query.to {
            list_query = list_query.bind(to);
        }

        let entities = list_query
            .bind(per_page as i64)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        Ok((entities, total))
    }

    /// Process a data subject request (approve, reject, complete).
    pub async fn process(
        &self,
        org_id: Uuid,
        id: Uuid,
        input: ProcessDataSubjectRequestInput,
    ) -> Result<Option<DataSubjectRequestEntity>, sqlx::Error> {
        let result_expires_at = input.result_expires_days.map(|days| Utc::now() + Duration::days(days));

        let entity = sqlx::query_as::<_, DataSubjectRequestEntity>(
            r#"
            UPDATE data_subject_requests
            SET
                status = $1,
                processed_by = $2,
                processed_at = NOW(),
                rejection_reason = $3,
                result_data = $4,
                result_file_url = $5,
                result_expires_at = $6,
                updated_at = NOW()
            WHERE id = $7 AND organization_id = $8
            RETURNING *
            "#,
        )
        .bind(input.status)
        .bind(input.processed_by)
        .bind(&input.rejection_reason)
        .bind(&input.result_data)
        .bind(&input.result_file_url)
        .bind(result_expires_at)
        .bind(id)
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(entity)
    }

    /// Get counts for compliance dashboard.
    pub async fn get_counts(&self, org_id: Uuid) -> Result<DataSubjectRequestCounts, sqlx::Error> {
        let row = sqlx::query_as::<_, (i64, i64, i64, i64, i64, i64)>(
            r#"
            SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'pending') as pending,
                COUNT(*) FILTER (WHERE status = 'in_progress') as in_progress,
                COUNT(*) FILTER (WHERE status = 'completed') as completed,
                COUNT(*) FILTER (WHERE status = 'rejected') as rejected,
                COUNT(*) FILTER (WHERE status IN ('pending', 'in_progress') AND due_date < NOW()) as overdue
            FROM data_subject_requests
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(DataSubjectRequestCounts {
            total: row.0,
            pending: row.1,
            in_progress: row.2,
            completed: row.3,
            rejected: row.4,
            overdue: row.5,
        })
    }

    /// Get counts by request type for compliance report.
    pub async fn get_counts_by_type(
        &self,
        org_id: Uuid,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> Result<Vec<(DataSubjectRequestTypeDb, i64)>, sqlx::Error> {
        let mut conditions = vec!["organization_id = $1".to_string()];
        let mut param_count = 1;

        if from.is_some() {
            param_count += 1;
            conditions.push(format!("created_at >= ${}", param_count));
        }
        if to.is_some() {
            param_count += 1;
            conditions.push(format!("created_at <= ${}", param_count));
        }

        let sql = format!(
            r#"
            SELECT request_type, COUNT(*) as count
            FROM data_subject_requests
            WHERE {}
            GROUP BY request_type
            "#,
            conditions.join(" AND ")
        );

        let mut query = sqlx::query_as::<_, (DataSubjectRequestTypeDb, i64)>(&sql).bind(org_id);

        if let Some(ref f) = from {
            query = query.bind(f);
        }
        if let Some(ref t) = to {
            query = query.bind(t);
        }

        let rows = query.fetch_all(&self.pool).await?;
        Ok(rows)
    }

    /// Get average processing time in days.
    pub async fn get_average_processing_time(
        &self,
        org_id: Uuid,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> Result<Option<f64>, sqlx::Error> {
        let mut conditions = vec![
            "organization_id = $1".to_string(),
            "status IN ('completed', 'rejected')".to_string(),
            "processed_at IS NOT NULL".to_string(),
        ];
        let mut param_count = 1;

        if from.is_some() {
            param_count += 1;
            conditions.push(format!("created_at >= ${}", param_count));
        }
        if to.is_some() {
            param_count += 1;
            conditions.push(format!("created_at <= ${}", param_count));
        }

        let sql = format!(
            r#"
            SELECT AVG(EXTRACT(EPOCH FROM (processed_at - created_at)) / 86400.0)
            FROM data_subject_requests
            WHERE {}
            "#,
            conditions.join(" AND ")
        );

        let mut query = sqlx::query_scalar::<_, Option<f64>>(&sql).bind(org_id);

        if let Some(ref f) = from {
            query = query.bind(f);
        }
        if let Some(ref t) = to {
            query = query.bind(t);
        }

        let avg = query.fetch_one(&self.pool).await?;
        Ok(avg)
    }

    /// Get compliance rate (completed on time / total completed).
    pub async fn get_compliance_rate(
        &self,
        org_id: Uuid,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> Result<Option<f64>, sqlx::Error> {
        let mut conditions = vec![
            "organization_id = $1".to_string(),
            "status = 'completed'".to_string(),
        ];
        let mut param_count = 1;

        if from.is_some() {
            param_count += 1;
            conditions.push(format!("created_at >= ${}", param_count));
        }
        if to.is_some() {
            param_count += 1;
            conditions.push(format!("created_at <= ${}", param_count));
        }

        let sql = format!(
            r#"
            SELECT
                CASE
                    WHEN COUNT(*) = 0 THEN NULL
                    ELSE (COUNT(*) FILTER (WHERE processed_at <= due_date)::FLOAT / COUNT(*)::FLOAT) * 100
                END
            FROM data_subject_requests
            WHERE {}
            "#,
            conditions.join(" AND ")
        );

        let mut query = sqlx::query_scalar::<_, Option<f64>>(&sql).bind(org_id);

        if let Some(ref f) = from {
            query = query.bind(f);
        }
        if let Some(ref t) = to {
            query = query.bind(t);
        }

        let rate = query.fetch_one(&self.pool).await?;
        Ok(rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_due_days() {
        assert_eq!(DEFAULT_DUE_DAYS, 30);
    }

    #[test]
    fn test_counts_default() {
        let counts = DataSubjectRequestCounts::default();
        assert_eq!(counts.total, 0);
        assert_eq!(counts.pending, 0);
        assert_eq!(counts.overdue, 0);
    }
}
