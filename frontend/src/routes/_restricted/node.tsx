import { createFileRoute } from "@tanstack/react-router";
import logger from "@/lib/logger";

const log = logger.getSubLogger({ name: "NodeRoute" });

export const Route = createFileRoute("/_restricted/node")({
  component: RouteComponent,
});

function RouteComponent() {
  log.info("Node page rendered");
  return <div>Hello "/_authenticated/node"!</div>;
}
