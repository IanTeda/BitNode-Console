import { useEffect, useMemo } from "react";
import { Logger } from "tslog";

const logger = new Logger({
  name: "BitNode-Console",
  minLevel: import.meta.env.PROD ? 4 : 2,
  type: import.meta.env.PROD ? "json" : "pretty",
  hideLogPositionForProduction: true,
});

// Returns a logger instance for the given name, with a "Mounted" log message when the component mounts
export function useLogger(name: string) {
  const log = useMemo(() => logger.getSubLogger({ name }), [name]);

  useEffect(() => {
    log.info("Mounted");
  }, [log]);

  return log;
}

export default logger;
