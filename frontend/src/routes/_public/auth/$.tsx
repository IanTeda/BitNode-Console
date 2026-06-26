import { createFileRoute, redirect } from "@tanstack/react-router";

// Catch any unauthenticated auth routes and redirect to login

export const Route = createFileRoute("/_public/auth/$")({
  beforeLoad: () => {
    throw redirect({ to: "/auth/login" });
  },
});
