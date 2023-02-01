import { StackContext, Api } from "@serverless-stack/resources";
import { Metric } from "aws-cdk-lib/aws-cloudwatch";

export function MyStack({ stack }: StackContext) {
  const metric = new Metric({
    namespace: 'elva-labs',
    metricName: 'beerWeight',
  });

  const alarm = metric.createAlarm(stack, 'BeerAlarm', {
    alarmName: 'BeerAlarm',
    alarmDescription: 'Alarm when beer weight is too low',
    // comparisonOperator: 'LessThanThreshold',
    evaluationPeriods: 1,
    threshold: 1000,
    // treatMissingData: 'notBreaching',
  });
}
