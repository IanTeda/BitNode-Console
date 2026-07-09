import { createFileRoute } from "@tanstack/react-router";
import { usePingQuery } from "@/queries/ping";
import { useLogger } from "@/lib/logger";

export const Route = createFileRoute("/_restricted/node")({
  component: RouteComponent,
});

function RouteComponent() {
  const log = useLogger("NodeRoute");
  const { data, isLoading, isError, error } = usePingQuery();

  if (isLoading) {
    return <div>Pinging backend...</div>;
  }

  if (isError) {
    log.error("Ping failed:", error);
    return <div>Ping failed: {error.message}</div>;
  }

  return <div>Backend response: {data?.pong}</div>;
}
