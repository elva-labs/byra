import { SSTConfig } from "sst";
import { ByraStack } from "./stacks/ByraStack";

export default {
  config(_input) {
    return {
      name: "byra-watcher",
      region: "eu-north-1",
    };
  },
  stacks(app) {
    app.stack(ByraStack);
  }
} satisfies SSTConfig;
