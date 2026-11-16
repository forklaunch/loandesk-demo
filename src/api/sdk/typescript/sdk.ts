import type {
  ForkLaunchSdkRequest,
  ForkLaunchSdkResponse,
  ForkLaunchErrorResponses,
  ForkLaunchHeaders,
} from '@forklaunch/universal-sdk';

export type AccessReview = {
  generated_at: Date;
  last_review_at?: Date | null;
  last_review_by?: string | null;
  review_interval_days: number;
  review_overdue: boolean;
  stale_accounts: number;
  stale_after_days: number;
  truncated: boolean;
  users: Array<AccessReviewEntry>
};

export type AccessReviewEntry = {
  created_at: Date;
  days_since_last_use: number;
  disabled_at?: Date | null;
  disabled_reason?: string | null;
  email: string;
  full_name: string;
  id: string;
  last_login_at?: Date | null;
  mfa_enabled: boolean;
  open_sessions: number;
  password_changed_at?: Date | null;
  role: string;
  stale: boolean;
  status: string
};

export type AccessReviewRecorded = {
  accounts: number;
  disabled_accounts: number;
  reviewed_at: Date;
  stale_accounts: number
};

export type AccessReviewUsersItem = AccessReviewEntry;

export type ApiApiAccessReviewResponseUsersItem = AccessReviewUsersItem;

export type ApiApiCreateApplicationBodyOwnersItem = NewApplicationOwnersItem;

export type ApiApiExportApplicationResponseApplication = ApplicationExportApplication;

export type ApiApiExportApplicationResponseApplicationRisk = ApplicationExportApplicationRisk;

export type ApiApiExportApplicationResponseDocumentsItem = ApplicationExportDocumentsItem;

export type ApiApiExportApplicationResponseOwnersItem = ApplicationExportOwnersItem;

export type ApiApiGetApplicationResponseApplication = ApplicationDetailApplication;

export type ApiApiGetApplicationResponseApplicationRisk = ApplicationDetailApplicationRisk;

export type ApiApiGetApplicationResponseDocumentsItem = ApplicationDetailDocumentsItem;

export type ApiApiGetApplicationResponseOwnersItem = ApplicationDetailOwnersItem;

export type ApiApiGetApplicationResponsePermissions = ApplicationDetailPermissions;

export type ApiApiGetLivenessResponseChecksItem = HealthResponseChecksItem;

export type ApiApiGetReadinessResponseChecksItem = HealthResponseChecksItem;

export type ApiApiListApplicationsResponseItem = Application;

export type ApiApiListApplicationsResponseItemRisk = Risk;

export type ApiApiListAuditLogResponseItem = AuditEntry;

export type ApiApiListUsersResponseItem = User;

export type ApiApiSearchApplicationsResponseItem = Application;

export type ApiApiSearchApplicationsResponseItemRisk = Risk;

export type ApiHealthGetHealthResponseChecksItem = HealthResponseChecksItem;

export type Application = {
  address: string;
  created_at: Date;
  created_by: string;
  created_by_name: string;
  decided_at?: Date | null;
  decided_by_name?: string | null;
  decision_note?: string | null;
  ein: string;
  id: string;
  industry: string;
  legal_name: string;
  net_income?: number | null;
  purpose?: string | null;
  reference: string;
  requested_amount: number;
  revenue?: number | null;
  risk: Risk;
  status: string;
  total_debt?: number | null
};

export type ApplicationDetail = {
  application: Application;
  documents: Array<Document>;
  owners: Array<Owner>;
  permissions: Permissions
};

export type ApplicationDetailApplication = Application;

export type ApplicationDetailApplicationRisk = Risk;

export type ApplicationDetailDocumentsItem = Document;

export type ApplicationDetailOwnersItem = Owner;

export type ApplicationDetailPermissions = Permissions;

export type ApplicationExport = {
  application: Application;
  documents: Array<Document>;
  exported_at: Date;
  owners: Array<Owner>
};

export type ApplicationExportApplication = Application;

export type ApplicationExportApplicationRisk = Risk;

export type ApplicationExportDocumentsItem = Document;

export type ApplicationExportOwnersItem = Owner;

export type AuditEntry = {
  action: string;
  at: Date;
  detail?: string | null;
  entity_id?: string | null;
  id: number;
  request_id?: string | null;
  user_id?: string | null
};

export type BrowserErrorReport = {
  class: string;
  kind: string;
  release: string;
  screen: string;
  stack: string
};

export type ChangePassword = {
  current_password: string;
  new_password: string
};

export type ClientSettingsResponse = {
  documents_through_api: boolean;
  hosted_demo: boolean;
  max_upload_bytes: number
};

export type Created = {
  id: string;
  reference: string
};

export type Decided = {
  status: string
};

export type Decision = {
  decision: string;
  note: string
};

export type DependencyCheck = {
  name: string;
  status: string
};

export type Document = {
  doc_type: string;
  error?: string | null;
  filename: string;
  id: string;
  net_income?: number | null;
  revenue?: number | null;
  size_bytes: number;
  status: string;
  total_debt?: number | null;
  uploaded_at: Date;
  uploaded_by_name: string
};

export type DocumentAccepted = {
  id: string;
  status: string
};

export type DownloadGrant = {
  url: string
};

export type EndpointFailure = {
  error: string
};

export type HealthResponse = {
  checks: Array<DependencyCheck>;
  service: string;
  status: string;
  version: string
};

export type HealthResponseChecksItem = DependencyCheck;

export type LoginRequest = {
  email: string;
  password: string;
  totp?: string
};

export type MfaCode = {
  code: string
};

export type MfaEnrolment = {
  otpauth_url: string;
  secret: string
};

export type NewApplication = {
  address: string;
  ein: string;
  industry: string;
  legal_name: string;
  owners: Array<NewOwner>;
  purpose: string;
  requested_amount: number
};

export type NewApplicationOwnersItem = NewOwner;

export type NewDocument = {
  doc_type: string;
  filename: string;
  size_bytes: number
};

export type NewOwner = {
  date_of_birth: string;
  full_name: string;
  ownership_percent: number;
  ssn: string
};

export type NewUser = {
  email: string;
  full_name: string;
  password: string;
  role: string
};

export type Ok = {
  ok: boolean
};

export type Owner = {
  date_of_birth: string;
  full_name: string;
  id: string;
  ownership_percent: number;
  ssn_last4: string
};

export type Permissions = {
  can_decide: boolean;
  can_upload: boolean;
  is_creator: boolean
};

export type RevealedSsn = {
  ssn: string
};

export type Risk = {
  annual_debt_service?: number | null;
  dscr?: number | null;
  flag?: string | null;
  leverage?: number | null;
  missing: Array<string>
};

export type SearchRequest = {
  cursor?: string;
  limit?: number;
  q: string
};

export type SessionUser = {
  email: string;
  full_name: string;
  id: string;
  mfa_enabled: boolean;
  role: string
};

export type SetRole = {
  role: string
};

export type UploadGrant = {
  fields: Record<string, string>;
  id: string;
  max_bytes: number;
  url: string
};

export type User = {
  created_at: Date;
  disabled_at?: Date | null;
  email: string;
  full_name: string;
  id: string;
  last_login_at?: Date | null;
  role: string
};

export type ListApplicationsRequest = ForkLaunchSdkRequest<
  never,
  {
    "limit"?: number;
    "cursor"?: string
  },
  never,
  ForkLaunchHeaders
>;

export type ListApplicationsResponse =
  | {
      code: 200;
      status: 200;
      contentType: 'application/json';
      response: Array<Application>;
      headers: {
        'x-next-cursor'?: string | null
      };
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 200 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type SearchApplicationsRequest = ForkLaunchSdkRequest<
  never,
  never,
  SearchRequest,
  ForkLaunchHeaders
>;

export type SearchApplicationsResponse =
  | {
      code: 200;
      status: 200;
      contentType: 'application/json';
      response: Array<Application>;
      headers: {
        'x-next-cursor'?: string | null
      };
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 200 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type CreateApplicationRequest = ForkLaunchSdkRequest<
  never,
  never,
  NewApplication,
  {
    "idempotency-key"?: string
  } & ForkLaunchHeaders
>;

export type CreateApplicationResponse =
  | {
      code: 201;
      status: 201;
      contentType: 'application/json';
      response: Created;
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 201 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type GetApplicationRequest = ForkLaunchSdkRequest<
  {
    "id": string
  },
  never,
  never,
  ForkLaunchHeaders
>;

export type GetApplicationResponse =
  | {
      code: 200;
      status: 200;
      contentType: 'application/json';
      response: ApplicationDetail;
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 200 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type DecideApplicationRequest = ForkLaunchSdkRequest<
  {
    "id": string
  },
  never,
  Decision,
  ForkLaunchHeaders
>;

export type DecideApplicationResponse =
  | {
      code: 200;
      status: 200;
      contentType: 'application/json';
      response: Decided;
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 200 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type RevealOwnerSsnRequest = ForkLaunchSdkRequest<
  {
    "id": string;
    "owner_id": string
  },
  never,
  never,
  ForkLaunchHeaders
>;

export type RevealOwnerSsnResponse =
  | {
      code: 200;
      status: 200;
      contentType: 'application/json';
      response: RevealedSsn;
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 200 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type ExportApplicationRequest = ForkLaunchSdkRequest<
  {
    "id": string
  },
  never,
  never,
  ForkLaunchHeaders
>;

export type ExportApplicationResponse =
  | {
      code: 200;
      status: 200;
      contentType: 'application/json';
      response: ApplicationExport;
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 200 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type SignInRequest = ForkLaunchSdkRequest<
  never,
  never,
  LoginRequest,
  ForkLaunchHeaders
>;

export type SignInResponse =
  | {
      code: 200;
      status: 200;
      contentType: 'application/json';
      response: SessionUser;
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 200 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type SignOutRequest = ForkLaunchSdkRequest<
  never,
  never,
  never,
  ForkLaunchHeaders
>;

export type SignOutResponse =
  | {
      code: 200;
      status: 200;
      contentType: 'application/json';
      response: Ok;
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 200 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type CurrentUserRequest = ForkLaunchSdkRequest<
  never,
  never,
  never,
  ForkLaunchHeaders
>;

export type CurrentUserResponse =
  | {
      code: 200;
      status: 200;
      contentType: 'application/json';
      response: SessionUser;
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 200 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type StartTwoFactorEnrolmentRequest = ForkLaunchSdkRequest<
  never,
  never,
  never,
  ForkLaunchHeaders
>;

export type StartTwoFactorEnrolmentResponse =
  | {
      code: 200;
      status: 200;
      contentType: 'application/json';
      response: MfaEnrolment;
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 200 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type ConfirmTwoFactorEnrolmentRequest = ForkLaunchSdkRequest<
  never,
  never,
  MfaCode,
  ForkLaunchHeaders
>;

export type ConfirmTwoFactorEnrolmentResponse =
  | {
      code: 200;
      status: 200;
      contentType: 'application/json';
      response: Ok;
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 200 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type ChangePasswordRequest = ForkLaunchSdkRequest<
  never,
  never,
  ChangePassword,
  ForkLaunchHeaders
>;

export type ChangePasswordResponse =
  | {
      code: 200;
      status: 200;
      contentType: 'application/json';
      response: Ok;
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 200 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type ReportBrowserErrorRequest = ForkLaunchSdkRequest<
  never,
  never,
  BrowserErrorReport,
  ForkLaunchHeaders
>;

export type ReportBrowserErrorResponse =
  | {
      code: 200;
      status: 200;
      contentType: 'application/json';
      response: Ok;
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 200 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type RequestDocumentUploadRequest = ForkLaunchSdkRequest<
  {
    "id": string
  },
  never,
  NewDocument,
  ForkLaunchHeaders
>;

export type RequestDocumentUploadResponse =
  | {
      code: 201;
      status: 201;
      contentType: 'application/json';
      response: UploadGrant;
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 201 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type CompleteDocumentUploadRequest = ForkLaunchSdkRequest<
  {
    "id": string
  },
  never,
  never,
  ForkLaunchHeaders
>;

export type CompleteDocumentUploadResponse =
  | {
      code: 200;
      status: 200;
      contentType: 'application/json';
      response: DocumentAccepted;
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 200 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type DownloadDocumentRequest = ForkLaunchSdkRequest<
  {
    "id": string
  },
  never,
  never,
  ForkLaunchHeaders
>;

export type DownloadDocumentResponse =
  | {
      code: 200;
      status: 200;
      contentType: 'application/json';
      response: DownloadGrant;
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 200 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type ReadDocumentContentRequest = ForkLaunchSdkRequest<
  {
    "id": string
  },
  never,
  never,
  ForkLaunchHeaders
>;

export type ReadDocumentContentResponse =
  | {
      code: 200;
      status: 200;
      contentType: 'application/pdf';
      response: Uint8Array;
      headers: {
        'content-disposition': 'attachment'
      };
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 200 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type GetReadinessRequest = ForkLaunchSdkRequest<
  never,
  never,
  never,
  ForkLaunchHeaders
>;

export type GetReadinessResponse =
  | {
      code: 200;
      status: 200;
      contentType: 'application/json';
      response: HealthResponse;
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 200 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type GetLivenessRequest = ForkLaunchSdkRequest<
  never,
  never,
  never,
  ForkLaunchHeaders
>;

export type GetLivenessResponse =
  | {
      code: 200;
      status: 200;
      contentType: 'application/json';
      response: HealthResponse;
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 200 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type GetClientSettingsRequest = ForkLaunchSdkRequest<
  never,
  never,
  never,
  ForkLaunchHeaders
>;

export type GetClientSettingsResponse =
  | {
      code: 200;
      status: 200;
      contentType: 'application/json';
      response: ClientSettingsResponse;
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 200 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type ListUsersRequest = ForkLaunchSdkRequest<
  never,
  {
    "limit"?: number;
    "cursor"?: string
  },
  never,
  ForkLaunchHeaders
>;

export type ListUsersResponse =
  | {
      code: 200;
      status: 200;
      contentType: 'application/json';
      response: Array<User>;
      headers: {
        'x-next-cursor'?: string | null
      };
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 200 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type CreateUserRequest = ForkLaunchSdkRequest<
  never,
  never,
  NewUser,
  ForkLaunchHeaders
>;

export type CreateUserResponse =
  | {
      code: 201;
      status: 201;
      contentType: 'application/json';
      response: User;
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 201 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type SetUserRoleRequest = ForkLaunchSdkRequest<
  {
    "id": string
  },
  never,
  SetRole,
  ForkLaunchHeaders
>;

export type SetUserRoleResponse =
  | {
      code: 200;
      status: 200;
      contentType: 'application/json';
      response: User;
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 200 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type DisableUserRequest = ForkLaunchSdkRequest<
  {
    "id": string
  },
  never,
  never,
  ForkLaunchHeaders
>;

export type DisableUserResponse =
  | {
      code: 200;
      status: 200;
      contentType: 'application/json';
      response: User;
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 200 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type EnableUserRequest = ForkLaunchSdkRequest<
  {
    "id": string
  },
  never,
  never,
  ForkLaunchHeaders
>;

export type EnableUserResponse =
  | {
      code: 200;
      status: 200;
      contentType: 'application/json';
      response: User;
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 200 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type AccessReviewRequest = ForkLaunchSdkRequest<
  never,
  never,
  never,
  ForkLaunchHeaders
>;

export type AccessReviewResponse =
  | {
      code: 200;
      status: 200;
      contentType: 'application/json';
      response: AccessReview;
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 200 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type RecordAccessReviewRequest = ForkLaunchSdkRequest<
  never,
  never,
  never,
  ForkLaunchHeaders
>;

export type RecordAccessReviewResponse =
  | {
      code: 201;
      status: 201;
      contentType: 'application/json';
      response: AccessReviewRecorded;
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 201 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type ListAuditLogRequest = ForkLaunchSdkRequest<
  never,
  {
    "limit"?: number;
    "cursor"?: string
  },
  never,
  ForkLaunchHeaders
>;

export type ListAuditLogResponse =
  | {
      code: 200;
      status: 200;
      contentType: 'application/json';
      response: Array<AuditEntry>;
      headers: {
        'x-next-cursor'?: string | null
      };
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 200 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type GetHealthRequest = ForkLaunchSdkRequest<
  never,
  never,
  never,
  ForkLaunchHeaders
>;

export type GetHealthResponse =
  | {
      code: 200;
      status: 200;
      contentType: 'application/json';
      response: HealthResponse;
    }
  | ForkLaunchSdkResponse<
      Record<400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503, EndpointFailure>,
      never,
      Record<number, 'application/json'>
    >
  | ForkLaunchSdkResponse<Omit<ForkLaunchErrorResponses, 200 | 400 | 401 | 403 | 404 | 408 | 409 | 413 | 415 | 429 | 500 | 502 | 503>>;

export type ApiSdkClient = {
  api: {
    listApplications: (input: ListApplicationsRequest) => Promise<ListApplicationsResponse>;
    searchApplications: (input: SearchApplicationsRequest) => Promise<SearchApplicationsResponse>;
    createApplication: (input: CreateApplicationRequest) => Promise<CreateApplicationResponse>;
    getApplication: (input: GetApplicationRequest) => Promise<GetApplicationResponse>;
    decideApplication: (input: DecideApplicationRequest) => Promise<DecideApplicationResponse>;
    revealOwnerSsn: (input: RevealOwnerSsnRequest) => Promise<RevealOwnerSsnResponse>;
    exportApplication: (input: ExportApplicationRequest) => Promise<ExportApplicationResponse>;
    signIn: (input: SignInRequest) => Promise<SignInResponse>;
    signOut: (input?: SignOutRequest) => Promise<SignOutResponse>;
    currentUser: (input?: CurrentUserRequest) => Promise<CurrentUserResponse>;
    startTwoFactorEnrolment: (input?: StartTwoFactorEnrolmentRequest) => Promise<StartTwoFactorEnrolmentResponse>;
    confirmTwoFactorEnrolment: (input: ConfirmTwoFactorEnrolmentRequest) => Promise<ConfirmTwoFactorEnrolmentResponse>;
    changePassword: (input: ChangePasswordRequest) => Promise<ChangePasswordResponse>;
    reportBrowserError: (input: ReportBrowserErrorRequest) => Promise<ReportBrowserErrorResponse>;
    requestDocumentUpload: (input: RequestDocumentUploadRequest) => Promise<RequestDocumentUploadResponse>;
    completeDocumentUpload: (input: CompleteDocumentUploadRequest) => Promise<CompleteDocumentUploadResponse>;
    downloadDocument: (input: DownloadDocumentRequest) => Promise<DownloadDocumentResponse>;
    readDocumentContent: (input: ReadDocumentContentRequest) => Promise<ReadDocumentContentResponse>;
    getReadiness: (input?: GetReadinessRequest) => Promise<GetReadinessResponse>;
    getLiveness: (input?: GetLivenessRequest) => Promise<GetLivenessResponse>;
    getClientSettings: (input?: GetClientSettingsRequest) => Promise<GetClientSettingsResponse>;
    listUsers: (input: ListUsersRequest) => Promise<ListUsersResponse>;
    createUser: (input: CreateUserRequest) => Promise<CreateUserResponse>;
    setUserRole: (input: SetUserRoleRequest) => Promise<SetUserRoleResponse>;
    disableUser: (input: DisableUserRequest) => Promise<DisableUserResponse>;
    enableUser: (input: EnableUserRequest) => Promise<EnableUserResponse>;
    accessReview: (input?: AccessReviewRequest) => Promise<AccessReviewResponse>;
    recordAccessReview: (input?: RecordAccessReviewRequest) => Promise<RecordAccessReviewResponse>;
    listAuditLog: (input: ListAuditLogRequest) => Promise<ListAuditLogResponse>;
  };
  health: {
    getHealth: (input?: GetHealthRequest) => Promise<GetHealthResponse>;
  };
};
