import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";
import type { RpcInterceptor } from "@protobuf-ts/runtime-rpc";
import { JournalsServiceClient } from "@/lib/generated_protos/bitnode_console/journals/journals.client";
import logger from "@/lib/logger";

const log = logger.getSubLogger({ name: "Journals RPC" });

const RPC_BASE_URL = import.meta.env.BITNODE_RPC_BASE_URL as string;
const RPC_DEADLINE_MS = Number(import.meta.env.BITNODE_RPC_DEADLINE_MS);

let currentAccessToken: string | undefined;

export function setAccessToken(token: string | undefined): void {
  currentAccessToken = token;
}

function accessTokenInterceptor(): RpcInterceptor {
  return {
    interceptUnary(next, method, input, options) {
      if (currentAccessToken) {
        options.meta ??= {};
        options.meta["access_token"] = currentAccessToken;
      }
      return next(method, input, options);
    },
    interceptServerStreaming(next, method, input, options) {
      if (currentAccessToken) {
        options.meta ??= {};
        options.meta["access_token"] = currentAccessToken;
      }
      return next(method, input, options);
    },
  };
}

let client: JournalsServiceClient | undefined;

export function journalsClient(): JournalsServiceClient {
  if (!client) {
    const transport = new GrpcWebFetchTransport({
      baseUrl: RPC_BASE_URL,
      deadline: RPC_DEADLINE_MS,
      interceptors: [accessTokenInterceptor()],
      fetchInit: {},
    });
    log.debug("gRPC-web transport created for", RPC_BASE_URL);
    client = new JournalsServiceClient(transport);
  }
  return client;
}
