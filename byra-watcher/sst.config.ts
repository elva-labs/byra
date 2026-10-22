import { SSTConfig } from "sst";
import { Aspects } from "aws-cdk-lib";
import { CfnFunction } from "aws-cdk-lib/aws-lambda";
import { Function } from "sst/constructs";
import { ByraStack } from "./stacks/ByraStack";

export default {
  config(_input) {
    return {
      name: "byra-watcher",
      region: "eu-north-1",
    };
  },
  stacks(app) {
    // SST v2 can't select nodejs22.x and Node 20 and older are EOL and unpatched (CVE-2026-111497): rewrite the deployed runtime.
    Aspects.of(app).add({
      visit(node) {
        if (
          node instanceof CfnFunction &&
          node.node.scope instanceof Function &&
          typeof node.runtime === "string" &&
          node.runtime.startsWith("nodejs")
        ) {
          node.runtime = "nodejs22.x";
        }
      },
    });
    app.stack(ByraStack);
  }
} satisfies SSTConfig;
