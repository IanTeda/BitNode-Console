import { createFileRoute, redirect } from "@tanstack/react-router";
import { LoginForm } from "@/components/login-form";
import logger from "@/lib/logger";

const log = logger.getSubLogger({ name: "LoginRoute" });

export const Route = createFileRoute("/_public/auth/login")({
  validateSearch: (search: Record<string, unknown>) => ({
    redirect: typeof search.redirect === "string" ? search.redirect : undefined,
  }),
  beforeLoad: async ({ context, search }) => {
    log.debug("Login route: checking for refresh token");
    const refreshed = await context.auth.handleRefresh();
    if (refreshed) {
      log.info("Silent refresh succeeded, redirecting");
      throw redirect({ to: search.redirect ?? "/dashboard" });
    }
  },
  component: RouteComponent,
});

function RouteComponent() {
  log.info("Login page rendered");
  return <LoginForm />;
}
