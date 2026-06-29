import { createFileRoute } from "@tanstack/react-router";
import { LoggedOutCard } from "@/components/LoggedOutCard";
import logger from "@/lib/logger";

const log = logger.getSubLogger({ name: "LoggedOutRoute" });

export const Route = createFileRoute("/_public/auth/logged-out")({
  beforeLoad: async ({ context }) => {
    await context.auth.handleLogout();
  },
  component: RouteComponent,
});

function RouteComponent() {
  log.info("Logged out page rendered");
  return <LoggedOutCard />;
}
