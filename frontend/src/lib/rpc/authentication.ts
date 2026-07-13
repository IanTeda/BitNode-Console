import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";
import { AuthenticationServiceClient } from "@/lib/generated_protos/bitnode_console/authentication/authentication.client";
import logger from "@/lib/logger";

const log = logger.getSubLogger({ name: "Authentication RPC" });

const RPC_BASE_URL = import.meta.env.BITNODE_RPC_BASE_URL as string;
const RPC_DEADLINE_MS = Number(import.meta.env.BITNODE_RPC_DEADLINE_MS);

let client: AuthenticationServiceClient | undefined;

/// Returns the shared AuthenticationServiceClient, creating it on first call.
/// No auth interceptor is attached — login is unauthenticated and refresh/logout
/// pass their tokens explicitly via call-site metadata.
export function authenticationClient(): AuthenticationServiceClient {
  if (!client) {
    const transport = new GrpcWebFetchTransport({
      baseUrl: RPC_BASE_URL,
      deadline: RPC_DEADLINE_MS,
      fetchInit: {},
    });
    log.debug("gRPC-web transport created for", RPC_BASE_URL);
    client = new AuthenticationServiceClient(transport);
  }
  return client;
}
