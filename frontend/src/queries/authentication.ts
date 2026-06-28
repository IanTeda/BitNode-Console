import { useMutation, useQueryClient } from "@tanstack/react-query";
import logger from "@/lib/logger";
import type { LoginResponse } from "@/lib/generated_protos/bitnode_console/v1/authentication/authentication";
import { authenticationClient } from "@/lib/rpc/authentication";

const log = logger.getSubLogger({ name: "Authentication Query" });

const AUTHENTICATION_QUERY_KEY = "authentication";

export function useAuthenticationMutation() {
  log.debug("Use authentication mutation hook");

  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: [AUTHENTICATION_QUERY_KEY],
    mutationFn: async ({ password }: { password: string }): Promise<LoginResponse> => {
      const { response } = await authenticationClient().login({ password });
      return response;
    },
    onSuccess: (data) => {
      queryClient.setQueryData([AUTHENTICATION_QUERY_KEY], data);
    },
  });
}
