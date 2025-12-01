//! Domain models for Phone Manager.

pub mod device;
pub mod device_policy;
pub mod enrollment_token;
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
    ListDevicePoliciesResponse, PolicyTarget, PolicyTargetType, UpdateDevicePolicyRequest,
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
