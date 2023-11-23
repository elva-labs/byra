import middy from "@middy/core";
import {
  logMetrics,
  Metrics,
  MetricUnits,
} from "@aws-lambda-powertools/metrics";

const metrics = new Metrics({ namespace: "elva-labs", serviceName: "byra" });

// TODO: check if we can do this transformation directly via iot-core -> cloud watch
export const handler = middy(async (
  event: {
    grams: number
  }) => {
  metrics.addMetric("beerWeight", MetricUnits.Count, event.grams);
}).use(logMetrics(metrics));
