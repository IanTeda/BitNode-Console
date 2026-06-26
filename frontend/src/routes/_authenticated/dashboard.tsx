import { createFileRoute } from "@tanstack/react-router";
import logger from "@/lib/logger";

const log = logger.getSubLogger({ name: "DashboardRoute" });

export const Route = createFileRoute("/_authenticated/dashboard")({
  component: RouteComponent,
});

function RouteComponent() {
  log.info("Dashboard page rendered");
  return <div>Hello "/_authenticated/dashboard"!</div>;
}
