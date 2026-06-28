import { useQuery } from "@tanstack/react-query";
import { utilitiesClient } from "@/lib/rpc/utilities";
import type { PingResponse } from "@/lib/generated_protos/bitnode_console/v1/utilities/utilities";

const PING_QUERY_KEY = ["ping"] as const;

async function ping(): Promise<PingResponse> {
  const { response } = await utilitiesClient().ping({});
  return response;
}

export function usePingQuery() {
  return useQuery({
    queryKey: PING_QUERY_KEY,
    queryFn: ping,
  });
}
