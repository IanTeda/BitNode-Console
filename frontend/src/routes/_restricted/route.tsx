import { createFileRoute } from "@tanstack/react-router";
import RestrictedLayout from "@/components/layouts/RestrictedLayout";

export const Route = createFileRoute("/_restricted")({
  component: RestrictedLayout,
});
