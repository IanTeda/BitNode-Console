import { useNavigate } from "@tanstack/react-router";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";

export function LoggedOutCard({
  className,
  ...props
}: React.ComponentProps<"div">) {
  const navigate = useNavigate();

  return (
    <div className={cn("flex flex-col gap-6", className)} {...props}>
      <div className="flex flex-col items-center gap-1 text-center">
        <h1 className="text-2xl font-bold">Logged Out</h1>
        <p className="text-sm text-balance text-muted-foreground">
          You have been logged out. You can close this window or log back in.
        </p>
      </div>
      <Button onClick={() => navigate({ to: "/auth/login", search: { redirect: undefined } })}>
        Log Back In
      </Button>
    </div>
  );
}
