import { createFileRoute, redirect } from "@tanstack/react-router";
import RestrictedLayout from "@/components/layouts/RestrictedLayout";
import { getAccessToken } from "@/lib/rpc/utilities";

export const Route = createFileRoute("/_restricted")({
  beforeLoad: ({ context, location }) => {
    if (!context.auth.isAuthenticated && !getAccessToken()) {
      throw redirect({
        to: "/auth/login",
        search: { redirect: location.pathname },
      });
    }
  },
  component: RestrictedLayout,
});
