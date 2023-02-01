import { StackContext, Cron } from "@serverless-stack/resources";
import {
  ComparisonOperator,
  Metric,
  TreatMissingData,
} from "aws-cdk-lib/aws-cloudwatch";

export function MyStack({ stack }: StackContext) {
  const metric = new Metric({
    namespace: "elva-labs",
    metricName: "beerWeight",
  });

  metric.createAlarm(stack, "BeerAlarm", {
    alarmName: "BeerAlarm",
    alarmDescription: "Alarm when beer weight is too low",
    comparisonOperator: ComparisonOperator.LESS_THAN_THRESHOLD,
    evaluationPeriods: 1,
    threshold: 1000,
    treatMissingData: TreatMissingData.NOT_BREACHING,
  });

  new Cron(stack, "Cron", {
    schedule: "rate(1 minute)",
    job: "../services/functions/lambda.handler",
  });
}
