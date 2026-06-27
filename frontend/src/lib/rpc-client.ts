import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";
import type { RpcInterceptor } from "@protobuf-ts/runtime-rpc";
import { UtilitiesServiceClient } from "@/lib/generated_protos/bitnode_console/v1/utilities.client";
import logger from "@/lib/logger";

const log = logger.getSubLogger({ name: "RPC Client" });

const DEFAULT_BASE_URL = "http://127.0.0.1:50051";
const DEFAULT_DEADLINE_MS = 30_000;

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
    const baseUrl = DEFAULT_BASE_URL;

    const transport = new GrpcWebFetchTransport({
      baseUrl,
      deadline: DEFAULT_DEADLINE_MS,
      interceptors: [authInterceptor(accessToken)],
      fetchInit: {},
    });

    log.debug("gRPC-web transport created for", baseUrl);

    const utilities = new UtilitiesServiceClient(transport);

    return new RpcClient(utilities);
  }
}
