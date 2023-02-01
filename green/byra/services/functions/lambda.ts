import { APIGatewayProxyHandlerV2 } from "aws-lambda";
import middy from '@middy/core';
import { logMetrics, Metrics, MetricUnits } from "@aws-lambda-powertools/metrics";

const metrics = new Metrics({ namespace: 'elva-labs', serviceName: 'byra' });

export const lambdaHandler: APIGatewayProxyHandlerV2 = async (event) => {
  metrics.addMetric('beerWeight', MetricUnits.Count, 3245);
  return {
    statusCode: 200,
    body: JSON.stringify({ message: 'success' }),
  }
};

export const handler = middy(lambdaHandler)
    .use(logMetrics(metrics));