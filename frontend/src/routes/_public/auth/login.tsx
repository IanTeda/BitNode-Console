import { createFileRoute } from "@tanstack/react-router";
import { LoginForm } from "@/components/login-form";
import logger from "@/lib/logger";

const log = logger.getSubLogger({ name: "LoginRoute" });

export const Route = createFileRoute("/_public/auth/login")({
  component: RouteComponent,
});

function RouteComponent() {
  log.info("Login page rendered");
  return <LoginForm />;
}
