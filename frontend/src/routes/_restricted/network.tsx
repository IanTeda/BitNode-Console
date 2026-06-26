import { createFileRoute } from "@tanstack/react-router";
import logger from "@/lib/logger";

const log = logger.getSubLogger({ name: "NetworkRoute" });

export const Route = createFileRoute("/_restricted/network")({
  component: RouteComponent,
});

function RouteComponent() {
  log.info("Network page rendered");
  return <div>Hello "/_authenticated/network"!</div>;
}
