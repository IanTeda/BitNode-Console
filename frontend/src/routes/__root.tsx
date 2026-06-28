import { createRootRouteWithContext, Outlet } from "@tanstack/react-router";
import TanStackDevTools from "../components/TanStackDevTools";
import type { AuthContext } from "@/components/AuthenticationProvider";

interface RouterContext {
  auth: AuthContext;
}

export const Route = createRootRouteWithContext<RouterContext>()({
  component: RootComponent,
});

function RootComponent() {
  return (
    <div>
      <Outlet />
      <TanStackDevTools />
    </div>
  );
}
