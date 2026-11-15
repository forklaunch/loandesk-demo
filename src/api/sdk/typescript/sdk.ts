import type {
  ForkLaunchSdkRequest,
  ForkLaunchSdkResponse,
  ForkLaunchErrorResponses,
  ForkLaunchHeaders,
} from '@forklaunch/universal-sdk';

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

export type BrowserErrorReport = {
  class: string;
  kind: string;
  release: string;
  screen: string;
  stack: string
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
  email: string;
  full_name: string;
  id: string;
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
    reportBrowserError: (input: ReportBrowserErrorRequest) => Promise<ReportBrowserErrorResponse>;
    requestDocumentUpload: (input: RequestDocumentUploadRequest) => Promise<RequestDocumentUploadResponse>;
    completeDocumentUpload: (input: CompleteDocumentUploadRequest) => Promise<CompleteDocumentUploadResponse>;
    downloadDocument: (input: DownloadDocumentRequest) => Promise<DownloadDocumentResponse>;
    getReadiness: (input?: GetReadinessRequest) => Promise<GetReadinessResponse>;
    getLiveness: (input?: GetLivenessRequest) => Promise<GetLivenessResponse>;
    listUsers: (input: ListUsersRequest) => Promise<ListUsersResponse>;
    createUser: (input: CreateUserRequest) => Promise<CreateUserResponse>;
    setUserRole: (input: SetUserRoleRequest) => Promise<SetUserRoleResponse>;
  };
  health: {
    getHealth: (input?: GetHealthRequest) => Promise<GetHealthResponse>;
  };
};
