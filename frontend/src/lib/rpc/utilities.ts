import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";
import type { RpcInterceptor } from "@protobuf-ts/runtime-rpc";
import { UtilitiesServiceClient } from "@/lib/generated_protos/bitnode_console/v1/utilities/utilities.client";
import logger from "@/lib/logger";

const log = logger.getSubLogger({ name: "Utilities RPC" });

const RPC_BASE_URL = import.meta.env.VITE_RPC_BASE_URL as string;
const RPC_DEADLINE_MS = Number(import.meta.env.VITE_RPC_DEADLINE_MS);

let currentAccessToken: string | undefined;

/// Sets the access token used by the utilities interceptor for all subsequent calls.
export function setAccessToken(token: string | undefined): void {
  currentAccessToken = token;
}

function accessTokenInterceptor(): RpcInterceptor {
  return {
    interceptUnary(next, method, input, options) {
      if (currentAccessToken) {
        options.meta ??= {};
        options.meta["Authorization"] = `Bearer ${currentAccessToken}`;
      }
      return next(method, input, options);
    },
  };
}

let client: UtilitiesServiceClient | undefined;

/// Returns the shared UtilitiesServiceClient, creating it on first call.
export function utilitiesClient(): UtilitiesServiceClient {
  if (!client) {
    const transport = new GrpcWebFetchTransport({
      baseUrl: RPC_BASE_URL,
      deadline: RPC_DEADLINE_MS,
      interceptors: [accessTokenInterceptor()],
      fetchInit: {},
    });
    log.debug("gRPC-web transport created for", RPC_BASE_URL);
    client = new UtilitiesServiceClient(transport);
  }
  return client;
}
