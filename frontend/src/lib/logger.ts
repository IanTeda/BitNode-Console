import { Logger } from "tslog";

const logger = new Logger({
  name: "BitNode-Console",
  minLevel: import.meta.env.PROD ? 4 : 2,
  type: import.meta.env.PROD ? "json" : "pretty",
  hideLogPositionForProduction: true,
});

export default logger;
