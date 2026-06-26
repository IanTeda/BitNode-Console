import { createFileRoute } from "@tanstack/react-router";
import logger from "@/lib/logger";

const log = logger.getSubLogger({ name: "LogsRoute" });

export const Route = createFileRoute("/_restricted/logs")({
  component: RouteComponent,
});

function RouteComponent() {
  log.info("Logs page rendered");
  return <div>Hello "/_authenticated/logs"!</div>;
}
