import { createFileRoute, redirect } from "@tanstack/react-router";
import { LoginForm } from "@/components/LoginForm";
import logger from "@/lib/logger";

const log = logger.getSubLogger({ name: "LoginRoute" });

export const Route = createFileRoute("/_public/auth/login")({
  // Validate search parameters for redirect
  validateSearch: (search: Record<string, unknown>) => ({
    redirect: typeof search.redirect === "string" ? search.redirect : undefined,
  }),

  // Check for refresh token on load, and redirect to dashboard if it succeeds
  beforeLoad: async ({ context, search }) => {
    log.debug("Login route: checking for refresh token");
    const refreshed = await context.auth.handleRefresh();
    if (refreshed) {
      log.info("Silent refresh succeeded, redirecting");
      throw redirect({ to: search.redirect ?? "/dashboard" });
    }
  },

  // Render the login form
  component: RouteComponent,
});

// Render the login form
function RouteComponent() {
  log.info("Login page rendered");
  return <LoginForm />;
}
