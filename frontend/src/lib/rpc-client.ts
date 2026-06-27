import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";
import type { RpcInterceptor } from "@protobuf-ts/runtime-rpc";
import { UtilitiesServiceClient } from "@/lib/generated_protos/bitnode_console/v1/utilities.client";
import logger from "@/lib/logger";

const log = logger.getSubLogger({ name: "RPC Client" });

const RPC_BASE_URL = import.meta.env.VITE_RPC_BASE_URL;
const RPC_DEADLINE_MS = Number(import.meta.env.VITE_RPC_DEADLINE_MS);

function authInterceptor(accessToken?: string): RpcInterceptor {
  return {
    interceptUnary(next, method, input, options) {
      if (accessToken) {
        options.meta ??= {};
        options.meta["Authorization"] = `Bearer ${accessToken}`;
      }
      return next(method, input, options);
    },
  };
}

export type UtilitiesClientType = UtilitiesServiceClient;

export class RpcClient {
  private static instance: RpcClient;

  private readonly utilities: UtilitiesClientType;

  private constructor(utilities: UtilitiesClientType) {
    this.utilities = utilities;
  }

  public static async getInstance(): Promise<RpcClient> {
    if (!RpcClient.instance) {
      RpcClient.instance = await RpcClient.create();
    }
    return RpcClient.instance;
  }

  public utilitiesClient(): UtilitiesClientType {
    return this.utilities;
  }

  public static async create(accessToken?: string): Promise<RpcClient> {
    const transport = new GrpcWebFetchTransport({
      baseUrl: RPC_BASE_URL,
      deadline: RPC_DEADLINE_MS,
      interceptors: [authInterceptor(accessToken)],
      fetchInit: {},
    });

    log.debug("gRPC-web transport created for", RPC_BASE_URL);

    const utilities = new UtilitiesServiceClient(transport);

    return new RpcClient(utilities);
  }
}
