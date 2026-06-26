import { createFileRoute } from "@tanstack/react-router";
import logger from "@/lib/logger";

const log = logger.getSubLogger({ name: "SettingsRoute" });

export const Route = createFileRoute("/_restricted/settings")({
  component: RouteComponent,
});

function RouteComponent() {
  log.info("Settings page rendered");
  return <div>Hello "/_authenticated/settings"!</div>;
}
