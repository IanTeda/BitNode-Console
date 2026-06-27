import { useQuery } from "@tanstack/react-query";
import { RpcClient } from "@/lib/rpc-client";
import type { PingResponse } from "@/lib/generated_protos/bitnode_console/v1/utilities";

const PING_QUERY_KEY = ["ping"] as const;

async function ping(): Promise<PingResponse> {
  const client = await RpcClient.getInstance();
  const { response } = await client.utilitiesClient().ping({});
  return response;
}

export function usePingQuery() {
  return useQuery({
    queryKey: PING_QUERY_KEY,
    queryFn: ping,
  });
}
