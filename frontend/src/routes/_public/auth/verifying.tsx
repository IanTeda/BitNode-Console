import { createFileRoute } from "@tanstack/react-router";
import logger from "@/lib/logger";
import { Loader2 } from "lucide-react";

const log = logger.getSubLogger({ name: "AuthenticatingRoute" });

export const Route = createFileRoute("/_public/auth/verifying")({
  component: RouteComponent,
});

function RouteComponent() {
  log.info("Authenticating page rendered");
  return (
    <div className="flex flex-col gap-6">
      <div className="flex flex-col items-center gap-1 text-center">
        <h1 className="text-2xl font-bold">Authenticating</h1>
        <p className="text-sm text-balance text-muted-foreground">
          Verifying the BitNode password, please wait...
        </p>
      </div>
      <div className="flex justify-center">
        <Loader2 className="size-6 animate-spin text-muted-foreground" />
      </div>
    </div>
  );
}
