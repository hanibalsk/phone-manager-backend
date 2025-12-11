//! Domain models for Phone Manager.

pub mod admin_geofence;
pub mod admin_group;
pub mod admin_user;
pub mod analytics;
pub mod api_key;
pub mod app_usage;
pub mod audit_log;
pub mod bulk_import;
pub mod compliance;
pub mod dashboard;
pub mod data_subject_request;
pub mod device;
pub mod device_policy;
pub mod device_token;
pub mod enrollment;
pub mod enrollment_token;
pub mod fleet;
pub mod geofence;
pub mod geofence_event;
pub mod group;
pub mod invite;
pub mod location;
pub mod movement_event;
pub mod org_member_invite;
pub mod org_user;
pub mod org_webhook;
pub mod organization;
pub mod organization_role;
pub mod organization_settings;
pub mod permission;
pub mod proximity_alert;
pub mod setting;
pub mod system_config;
pub mod system_role;
pub mod trip;
pub mod trip_path_correction;
pub mod unlock_request;
pub mod usage_warning;
pub mod user;
pub mod webhook;

pub use admin_geofence::{
    AdminAllDeviceLocationsResponse, AdminDeviceLocation, AdminDeviceLocationResponse,
    AdminGeofenceEventInfo, AdminGeofenceEventsQuery, AdminGeofenceEventsResponse,
    AdminGeofenceInfo, AdminGeofenceListResponse, AdminGeofencePagination, AdminGeofenceQuery,
    AdminLocationAnalyticsResponse, AdminLocationHistoryQuery, AdminLocationHistoryResponse,
    CreateAdminGeofenceRequest, CreateAdminGeofenceResponse, DeleteAdminGeofenceResponse,
    GeofenceVisitCount, LocationAnalyticsSummary, UpdateAdminGeofenceRequest,
    UpdateAdminGeofenceResponse,
};
pub use admin_group::{
    AddGroupMemberRequest, AddGroupMemberResponse, AdminGroupDetailResponse, AdminGroupItem,
    AdminGroupListResponse, AdminGroupPagination, AdminGroupProfile, AdminGroupQuery,
    AdminGroupSortField, AdminGroupSummary, CreateGroupInvitationRequest,
    CreateGroupInvitationResponse, DeactivateGroupResponse, GroupDeviceInfo, GroupInvitationInfo,
    GroupMemberInfo, GroupMembersPagination, GroupOwnerInfo, ListGroupInvitationsResponse,
    ListGroupMembersQuery, ListGroupMembersResponse, RemoveGroupMemberResponse,
    UpdateAdminGroupRequest, UpdateAdminGroupResponse,
};
pub use admin_user::{
    AdminUserDetailResponse, AdminUserItem, AdminUserListResponse, AdminUserPagination,
    AdminUserProfile, AdminUserQuery, AdminUserSortField, AdminUserSummary, RecentAction,
    RemoveUserResponse, SortOrder as AdminSortOrder, UpdateAdminUserRequest,
    UpdateAdminUserResponse, UserActivitySummary, UserDeviceInfo, UserGroupInfo,
};
pub use analytics::{
    AnalyticsGroupBy, AnalyticsPeriod, ApiUsageAnalyticsQuery, ApiUsageAnalyticsResponse,
    ApiUsageSummary, ApiUsageTrend, DeviceActivityTrend, DeviceAnalyticsQuery,
    DeviceAnalyticsResponse, DeviceAnalyticsSummary,
    DeviceStatusBreakdown as AnalyticsDeviceStatusBreakdown, EndpointUsage, GenerateReportRequest,
    ReportDownloadResponse, ReportFormat, ReportJobResponse, ReportStatus, UserActivityTrend,
    UserAnalyticsQuery, UserAnalyticsResponse, UserAnalyticsSummary, UserRoleBreakdown,
};
pub use api_key::{
    ApiKeyPagination, ApiKeyResponse, CreateApiKeyRequest, CreateApiKeyResponse, ListApiKeysQuery,
    ListApiKeysResponse, UpdateApiKeyRequest, MAX_API_KEYS_PER_ORG,
};
pub use audit_log::{
    ActorType, AsyncExportResponse, AuditAction, AuditActor, AuditLog, AuditLogPagination,
    AuditMetadata, AuditResource, CreateAuditLogInput, ExportAuditLogsQuery, ExportFormat,
    ExportJobResponse, ExportJobStatus, FieldChange, ListAuditLogsQuery, ListAuditLogsResponse,
    ResourceType, SyncExportResponse, EXPORT_JOB_EXPIRY_HOURS, MAX_EXPORT_RECORDS,
    MAX_SYNC_EXPORT_RECORDS,
};
pub use bulk_import::{
    BulkDeviceImportRequest, BulkDeviceImportResponse, BulkDeviceInput, BulkDeviceItem,
    BulkImportError, BulkImportJobStatus, BulkImportOptions, BulkImportResult,
    MAX_BULK_IMPORT_DEVICES, MAX_METADATA_SIZE,
};
pub use compliance::{
    ActionCount, AuditActivitySummary, AuditLogStats, ComplianceAssessment,
    ComplianceDashboardResponse, ComplianceFinding, ComplianceReportFormat, ComplianceReportQuery,
    ComplianceReportResponse, ComplianceStatus, DataRetentionStatus, DataSubjectRequestReportSummary,
    DataSubjectRequestStats, FindingSeverity, OrganizationReportSummary, RequestStatusCounts,
    RequestTypeCount,
};
pub use dashboard::{
    ActivityPeriod, ActivitySummary, DashboardMetrics, DeviceMetrics, DeviceStatusBreakdown,
    EnrollmentMetrics, GroupMetrics, PolicyMetrics, RoleBreakdown, TrendData, Trends, UserMetrics,
};
pub use data_subject_request::{
    CreateDataSubjectRequestRequest, DataSubjectRequestAction, DataSubjectRequestPagination,
    DataSubjectRequestResponse, DataSubjectRequestStatus, DataSubjectRequestType,
    ListDataSubjectRequestsQuery, ListDataSubjectRequestsResponse, ProcessDataSubjectRequestRequest,
    ProcessorInfo,
};
pub use device::Device;
pub use device_policy::{
    AppliedToCount, ApplyPolicyRequest, ApplyPolicyResponse, CreateDevicePolicyRequest,
    DevicePolicy, DevicePolicyPagination, DevicePolicyResponse, ListDevicePoliciesQuery,
    ListDevicePoliciesResponse, PolicyTarget, PolicyTargetType, UnapplyPolicyRequest,
    UnapplyPolicyResponse, UpdateDevicePolicyRequest,
};
pub use device_token::{
    calculate_device_token_expiry, extract_device_token_prefix, generate_device_token, DeviceToken,
    EnrollmentStatus, DEFAULT_TOKEN_EXPIRY_DAYS, DEVICE_TOKEN_PREFIX,
};
pub use enrollment::{
    DeviceInfo, EnrollDeviceRequest, EnrollDeviceResponse, EnrolledDevice, EnrollmentGroupInfo,
    EnrollmentPolicyInfo,
};
pub use enrollment_token::{
    calculate_expiry, extract_prefix, generate_token, CreateEnrollmentTokenRequest,
    EnrollmentToken, EnrollmentTokenPagination, EnrollmentTokenResponse, ListEnrollmentTokensQuery,
    ListEnrollmentTokensResponse, QrCodeResponse,
};
pub use fleet::{
    AssignDeviceRequest, AssignDeviceResponse, AssignedUserInfo, BulkDeviceUpdate,
    BulkDeviceUpdateResult, BulkUpdateDevicesRequest, BulkUpdateDevicesResponse, DeviceCommand,
    DeviceCommandHistoryItem, DeviceCommandHistoryPagination, DeviceCommandHistoryQuery,
    DeviceCommandHistoryResponse, DeviceCommandStatus, DeviceCommandType,
    DeviceStatusChangeResponse, FleetDeviceItem, FleetDeviceListResponse, FleetDeviceQuery,
    FleetGroupInfo, FleetLastLocation, FleetPagination, FleetPolicyInfo, FleetSortField,
    FleetSummary, IssueCommandRequest, IssueCommandResponse, SortOrder, UnassignDeviceResponse,
    MAX_BULK_UPDATE_DEVICES,
};
pub use geofence::Geofence;
pub use geofence_event::{
    CreateGeofenceEventRequest, GeofenceEvent, GeofenceEventResponse, GeofenceTransitionType,
    ListGeofenceEventsQuery, ListGeofenceEventsResponse,
};
pub use group::{Group, GroupMembership, GroupRole};
pub use invite::GroupInvite;
pub use location::Location;
pub use movement_event::MovementEvent;
pub use org_member_invite::{
    AcceptInvitationRequest, AcceptInvitationResponse, AcceptedOrgInfo, AcceptedUserInfo,
    CreateInvitationRequest, CreateInvitationResponse, InvitationPagination, InvitationResponse,
    InvitationStatus, InvitationSummary, InvitedByInfo, ListInvitationsQuery,
    ListInvitationsResponse, DEFAULT_EXPIRATION_DAYS, MAX_EXPIRATION_DAYS, MAX_INVITATIONS_PER_ORG,
    MIN_EXPIRATION_DAYS,
};
pub use org_user::{
    validate_permissions, AddOrgUserRequest, ForceMfaResponse, ListOrgUsersQuery,
    ListOrgUsersResponse, ListUserSessionsResponse, MfaMethod, MfaStatusResponse, OrgUser,
    OrgUserInfo, OrgUserPagination, OrgUserResponse, OrgUserRole, OrgUserWithDetails,
    ReactivateOrgUserResponse, ResetMfaResponse, RevokeAllSessionsResponse, RevokeSessionResponse,
    SuspendOrgUserRequest, SuspendOrgUserResponse, TriggerPasswordResetResponse,
    UpdateOrgUserRequest, UserSessionInfo, PERMISSIONS,
};
pub use org_webhook::{
    CreateOrgWebhookRequest, ListOrgWebhooksResponse, ListWebhookDeliveriesQuery,
    ListWebhookDeliveriesResponse, OrgWebhookResponse, RetryDeliveryResponse,
    TestOrgWebhookRequest, TestOrgWebhookResponse, UpdateOrgWebhookRequest,
    WebhookDeliveryResponse, WebhookPagination, WebhookStatsResponse, MAX_WEBHOOKS_PER_ORG,
    SUPPORTED_EVENT_TYPES,
};
pub use organization::{
    CreateOrganizationRequest, CreateOrganizationResponse, DeviceStatusCounts, DeviceUsageMetric,
    ListOrganizationsQuery, ListOrganizationsResponse, Organization, OrganizationPagination,
    OrganizationUsageResponse, OrganizationWithUsage, PlanType, ReactivateOrganizationResponse,
    SuspendOrganizationRequest, SuspendOrganizationResponse, UpdateOrganizationRequest,
    UsageMetric, SLUG_REGEX,
};
pub use organization_role::{
    is_system_role_name, CreateOrganizationRoleRequest, DeleteOrganizationRoleResponse,
    ListOrganizationRolesQuery, ListOrganizationRolesResponse, OrganizationRole,
    OrganizationRoleResponse, SYSTEM_ROLE_NAMES,
};
pub use organization_settings::{
    OrganizationSettings, OrganizationSettingsResponse, UpdateOrganizationSettingsRequest,
    VerifyPinRequest, VerifyPinResponse,
};
pub use permission::{
    get_all_permissions, get_permissions_by_category, get_permissions_by_category_filter,
    ListPermissionsQuery, ListPermissionsResponse, Permission, PermissionCategory,
    PermissionsByCategory,
};
pub use proximity_alert::ProximityAlert;
pub use setting::{
    DeviceSetting, GetSettingsResponse, SettingCategory, SettingDataType, SettingDefinition,
    SettingValue,
};
pub use system_config::{
    AuthTogglesInfo, DatabaseSettingsInfo, EmailSettingsInfo, EmailTemplate, EmailTemplatesResponse,
    FcmSettingsInfo, FeatureFlagResponse, FeatureFlagsInfo, FeatureFlagsResponse,
    FrontendSettingsInfo, LimitsSettingsInfo, LoggingSettingsInfo, MaintenanceModeResponse,
    MapMatchingSettingsInfo, NotificationTemplate, NotificationTemplatesResponse, RateLimitConfigItem,
    RateLimitsResponse, SecuritySettingsInfo, ServerSettingsInfo, SystemSettingItem,
    SystemSettingsResponse, ToggleMaintenanceModeRequest, UpdateEmailTemplateRequest,
    UpdateFeatureFlagRequest, UpdateNotificationTemplateRequest, UpdateRateLimitsRequest,
    UpdateSystemSettingsRequest, UpdateSystemSettingsResponse,
};
pub use system_role::{
    AddSystemRoleRequest, AddSystemRoleResponse, AdminOrgAssignment, AssignOrgRequest,
    AssignOrgResponse, ListSystemRolesResponse, OrgAssignmentDetail, RemoveOrgAssignmentResponse,
    RemoveSystemRoleResponse, SystemRole, SystemRoleInfo, UserOrgAssignmentsResponse,
    UserSystemRole, UserSystemRoleDetail, UserSystemRolesResponse, SYSTEM_PERMISSIONS,
};
pub use trip::Trip;
pub use trip_path_correction::TripPathCorrection;
pub use unlock_request::{
    AdminListUnlockRequestsQuery, AdminListUnlockRequestsResponse, AdminUnlockPagination,
    AdminUnlockRequestActionResponse, AdminUnlockRequestItem, AdminUserBrief,
    ApproveUnlockRequestRequest, BulkProcessUnlockRequestsRequest,
    BulkProcessUnlockRequestsResponse, CreateUnlockRequestRequest, CreateUnlockRequestResponse,
    DenyUnlockRequestRequest, ListUnlockRequestsQuery, ListUnlockRequestsResponse,
    RespondToUnlockRequestRequest, RespondToUnlockRequestResponse, UnlockRequestStatus,
};
pub use app_usage::{
    AnalyticsSummary, AnalyticsTrendPoint, AppUsageAnalyticsQuery, AppUsageAnalyticsResponse,
    AppUsageHistoryEntry, AppUsageHistoryQuery, AppUsageHistoryResponse, AppUsageItem,
    AppUsagePagination, AppUsagePeriod, AppUsageSummary, AppUsageSummaryQuery,
    AppUsageSummaryResponse, CategoryUsageItem, TopAppItem,
};
pub use usage_warning::{check_usage_warning, ResponseWithWarnings, UsageWarning};
pub use user::{OAuthAccount, OAuthProvider, User, UserSession};
pub use webhook::{
    CreateWebhookRequest, ListWebhooksQuery, ListWebhooksResponse, UpdateWebhookRequest, Webhook,
    WebhookResponse,
};
