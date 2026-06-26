import { createFileRoute } from "@tanstack/react-router";
import { LoggedOutCard } from "@/components/logged-out-card";
import logger from "@/lib/logger";

const log = logger.getSubLogger({ name: "LoggedOutRoute" });

export const Route = createFileRoute("/_public/auth/logged-out")({
  component: RouteComponent,
});

function RouteComponent() {
  log.info("Logged out page rendered");
  return <LoggedOutCard />;
}
