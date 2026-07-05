import { useQuery } from "@tanstack/react-query";
import { journalsClient } from "@/lib/rpc/journals";
import type { GetJournalsResponse } from "@/lib/generated_protos/bitnode_console/journals/journals";
import { Priority } from "@/lib/generated_protos/bitnode_console/journals/journals";
import { PageDirection } from "@/lib/generated_protos/bitnode_console/common/pagination";
import logger from "@/lib/logger";

const log = logger.getSubLogger({ name: "Journals Query" });

export const JOURNALS_QUERY_KEY = ["journals"] as const;

export interface JournalsQueryParams {
  timestampFromUs?: string;
  timestampToUs?: string;
  priority?: Priority;
  pageSize?: number;
  pageToken?: string;
}

async function getJournals(params: JournalsQueryParams): Promise<GetJournalsResponse> {
  log.debug("Fetching journals", params);
  const { response } = await journalsClient().getJournals({
    timestampFromUs: params.timestampFromUs,
    timestampToUs: params.timestampToUs,
    priority: params.priority ?? Priority.UNSPECIFIED,
    pagination: {
      pageSize: params.pageSize ?? 100,
      pageToken: params.pageToken,
      pageDirection: PageDirection.UNSPECIFIED,
    },
  });
  return response;
}

export function useJournalsQuery(params: JournalsQueryParams = {}) {
  return useQuery({
    queryKey: [...JOURNALS_QUERY_KEY, params],
    queryFn: () => getJournals(params),
  });
}
