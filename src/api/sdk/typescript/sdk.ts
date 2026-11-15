import type {
  ForkLaunchSdkRequest,
  ForkLaunchSdkResponse,
  ForkLaunchErrorResponses,
  ForkLaunchHeaders,
} from '@forklaunch/universal-sdk';

export type EndpointFailure = {
  error: string
};

export type HealthResponse = {
  recordCount: number;
  routes: Array<string>;
  service: string;
  status: string
};

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
  health: {
    getHealth: (input?: GetHealthRequest) => Promise<GetHealthResponse>;
  };
};
