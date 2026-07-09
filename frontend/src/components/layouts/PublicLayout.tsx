import { Outlet } from "@tanstack/react-router";

export default function PublicLayout() {
  return (
    <div>
      <main>
        <Outlet />
      </main>
    </div>
  );
}
