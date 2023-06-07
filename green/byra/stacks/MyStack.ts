import * as events from "aws-cdk-lib/aws-events";
import { StackContext, EventBus, Function } from "@serverless-stack/resources";
import {
  ComparisonOperator,
  Metric,
  TreatMissingData,
} from "aws-cdk-lib/aws-cloudwatch";

export function MyStack({ stack }: StackContext) {
  const metric = new Metric({
    namespace: "elva-labs",
    metricName: "beerWeight",
    dimensionsMap: {
      service: "byra",
    },
  });

  metric.createAlarm(stack, "BeerAlarm", {
    alarmName: "BeerAlarm",
    alarmDescription: "Alarm when beer weight is too low",
    comparisonOperator: ComparisonOperator.LESS_THAN_THRESHOLD,
    evaluationPeriods: 1,
    threshold: 1000,
    treatMissingData: TreatMissingData.NOT_BREACHING,
  });

  new Function(stack, "PushMetricLambda", {
    handler: "../services/functions/lambda.handler",
  });

  new EventBus(stack, "DefaultEventBusSlackHandler", {
    cdk: {
      eventBus: events.EventBus.fromEventBusName(
        stack,
        "default-bus",
        "default"
      ),
    },
    rules: {
      myRule: {
        pattern: {
          source: ["aws.cloudwatch"],
          detailType: ["CloudWatch Alarm State Change"],
        },
        targets: {
          slackService: "../services/functions/slack.handler",
        },
      },
    },
  });
}
