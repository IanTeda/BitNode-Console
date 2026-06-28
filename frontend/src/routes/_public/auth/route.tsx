import { createFileRoute, redirect } from "@tanstack/react-router";
import AuthLayout from "@/components/layouts/AuthLayout";

export const Route = createFileRoute("/_public/auth")({
  beforeLoad: ({ context }) => {
    if (context.auth.isAuthenticated) {
      throw redirect({ to: "/dashboard" });
    }
  },
  component: AuthLayout,
});
