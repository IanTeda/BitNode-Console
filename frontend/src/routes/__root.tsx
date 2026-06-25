import { createRootRoute, Outlet } from "@tanstack/react-router";
import TanStackDevTools from "../components/TanStackDevTools";

export const Route = createRootRoute({
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
