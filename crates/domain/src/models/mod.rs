//! Domain models for Phone Manager.

pub mod admin_group;
pub mod admin_user;
pub mod audit_log;
pub mod bulk_import;
pub mod dashboard;
pub mod device;
pub mod device_policy;
pub mod device_token;
pub mod enrollment;
pub mod enrollment_token;
pub mod fleet;
pub mod geofence;
pub mod group;
pub mod invite;
pub mod location;
pub mod movement_event;
pub mod org_user;
pub mod organization;
pub mod proximity_alert;
pub mod setting;
pub mod trip;
pub mod trip_path_correction;
pub mod unlock_request;
pub mod user;

pub use device::Device;
pub use device_policy::{
    AppliedToCount, ApplyPolicyRequest, ApplyPolicyResponse, CreateDevicePolicyRequest,
    DevicePolicy, DevicePolicyPagination, DevicePolicyResponse, ListDevicePoliciesQuery,
    ListDevicePoliciesResponse, PolicyTarget, PolicyTargetType, UnapplyPolicyRequest,
    UnapplyPolicyResponse, UpdateDevicePolicyRequest,
};
pub use enrollment_token::{
    CreateEnrollmentTokenRequest, EnrollmentToken, EnrollmentTokenPagination,
    EnrollmentTokenResponse, ListEnrollmentTokensQuery, ListEnrollmentTokensResponse, QrCodeResponse,
    calculate_expiry, extract_prefix, generate_token,
};
pub use geofence::Geofence;
pub use group::{Group, GroupMembership, GroupRole};
pub use invite::GroupInvite;
pub use location::Location;
pub use movement_event::MovementEvent;
pub use org_user::{
    AddOrgUserRequest, ListOrgUsersQuery, ListOrgUsersResponse, OrgUser, OrgUserInfo,
    OrgUserPagination, OrgUserResponse, OrgUserRole, OrgUserWithDetails, UpdateOrgUserRequest,
    PERMISSIONS, validate_permissions,
};
pub use organization::{
    CreateOrganizationRequest, CreateOrganizationResponse, DeviceStatusCounts, DeviceUsageMetric,
    ListOrganizationsQuery, ListOrganizationsResponse, Organization, OrganizationPagination,
    OrganizationUsageResponse, OrganizationWithUsage, PlanType, UpdateOrganizationRequest, UsageMetric,
    SLUG_REGEX,
};
pub use proximity_alert::ProximityAlert;
pub use setting::{
    DeviceSetting, GetSettingsResponse, SettingCategory, SettingDataType, SettingDefinition,
    SettingValue,
};
pub use trip::Trip;
pub use trip_path_correction::TripPathCorrection;
pub use unlock_request::{
    CreateUnlockRequestRequest, CreateUnlockRequestResponse, ListUnlockRequestsQuery,
    ListUnlockRequestsResponse, RespondToUnlockRequestRequest, RespondToUnlockRequestResponse,
    UnlockRequestStatus,
};
pub use user::{OAuthAccount, OAuthProvider, User, UserSession};
pub use device_token::{
    DeviceToken, EnrollmentStatus, calculate_device_token_expiry, extract_device_token_prefix,
    generate_device_token, DEFAULT_TOKEN_EXPIRY_DAYS, DEVICE_TOKEN_PREFIX,
};
pub use enrollment::{
    DeviceInfo, EnrollDeviceRequest, EnrollDeviceResponse, EnrolledDevice, EnrollmentGroupInfo,
    EnrollmentPolicyInfo,
};
pub use fleet::{
    AssignDeviceRequest, AssignDeviceResponse, AssignedUserInfo, DeviceCommand, DeviceCommandStatus,
    DeviceCommandType, DeviceStatusChangeResponse, FleetDeviceItem, FleetDeviceListResponse,
    FleetDeviceQuery, FleetGroupInfo, FleetLastLocation, FleetPagination, FleetPolicyInfo,
    FleetSortField, FleetSummary, IssueCommandRequest, IssueCommandResponse, SortOrder,
    UnassignDeviceResponse,
};
pub use audit_log::{
    ActorType, AsyncExportResponse, AuditAction, AuditActor, AuditLog, AuditLogPagination, AuditMetadata,
    AuditResource, CreateAuditLogInput, ExportAuditLogsQuery, ExportFormat, ExportJobResponse,
    ExportJobStatus, FieldChange, ListAuditLogsQuery, ListAuditLogsResponse, ResourceType,
    SyncExportResponse, MAX_EXPORT_RECORDS, MAX_SYNC_EXPORT_RECORDS, EXPORT_JOB_EXPIRY_HOURS,
};
pub use bulk_import::{
    BulkDeviceImportRequest, BulkDeviceImportResponse, BulkDeviceInput, BulkDeviceItem,
    BulkImportError, BulkImportJobStatus, BulkImportOptions, BulkImportResult,
    MAX_BULK_IMPORT_DEVICES, MAX_METADATA_SIZE,
};
pub use dashboard::{
    ActivityPeriod, ActivitySummary, DashboardMetrics, DeviceMetrics, DeviceStatusBreakdown,
    EnrollmentMetrics, GroupMetrics, PolicyMetrics, RoleBreakdown, TrendData, Trends, UserMetrics,
};
pub use admin_user::{
    AdminUserDetailResponse, AdminUserItem, AdminUserListResponse, AdminUserPagination,
    AdminUserProfile, AdminUserQuery, AdminUserSortField, AdminUserSummary, RecentAction,
    RemoveUserResponse, SortOrder as AdminSortOrder, UpdateAdminUserRequest, UpdateAdminUserResponse,
    UserActivitySummary, UserDeviceInfo, UserGroupInfo,
};
pub use admin_group::{
    AdminGroupDetailResponse, AdminGroupItem, AdminGroupListResponse, AdminGroupPagination,
    AdminGroupProfile, AdminGroupQuery, AdminGroupSortField, AdminGroupSummary, DeactivateGroupResponse,
    GroupDeviceInfo, GroupMemberInfo, GroupOwnerInfo, UpdateAdminGroupRequest, UpdateAdminGroupResponse,
};
