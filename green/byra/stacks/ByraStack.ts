import * as events from "aws-cdk-lib/aws-events";
import * as iot from "aws-cdk-lib/aws-iot";
import { StackContext, EventBus, Function } from "@serverless-stack/resources";
import { ThingWithCert } from 'cdk-iot-core-certificates';
import {
  ComparisonOperator,
  Metric,
  TreatMissingData,
} from "aws-cdk-lib/aws-cloudwatch";
import { CfnOutput, Duration } from "aws-cdk-lib";

export function ByraStack({ stack }: StackContext) {
  const metricPushLambda = new Function(stack, "PushMetricLambda", {
    handler: "../services/functions/lambda.handler",
  })

  new iot.CfnTopicRule(stack, 'ByraIotHandler', {
    ruleName: 'weightListener', topicRulePayload: {
      actions: [
        {
          lambda: {
            functionArn: metricPushLambda.functionArn
          }
        },
      ],
      ruleDisabled: false,
      sql: "SELECT * FROM 'byra/weight'",
    },
  });

  const { thingArn, certId, certPem, privKey } = new ThingWithCert(stack, 'ByraScale01', {
    thingName: 'byra-01',
    saveToParamStore: true,
    paramPrefix: 'devices',
  });

  const ByraCrtPolicy = new iot.CfnPolicy(stack, 'ByraIotPolicy', {
    policyName: 'byra-iot-commnication-policy',
    policyDocument: {
      Version: '2012-10-17',
      Statement: [
        {
          Effect: 'Allow',
          // OBS: make more specific if you'd like, cba atm.
          Action: 'iot:*',
          Resource: '*',
        },
      ],
    }
  })

  // Attach the policy to the certificate
  new iot.CfnPolicyPrincipalAttachment(
    stack,
    'ByraPolicyPrincipalAttachment',
    {
      policyName: ByraCrtPolicy.policyName!,
      principal: `arn:aws:iot:${stack.region}:${stack.account}:cert/${certId}`,
    }
  );

  const metric = new Metric({
    namespace: "elva-labs",
    metricName: "beerWeight",
    dimensionsMap: {
      service: "byra",
    },
    period: Duration.minutes(5)
  });

  metric.createAlarm(stack, "BeerAlarm", {
    alarmName: "BeerAlarm",
    alarmDescription: "Alarm when beer weight is too low",
    comparisonOperator: ComparisonOperator.LESS_THAN_THRESHOLD,
    evaluationPeriods: 3,
    threshold: 10000,
    treatMissingData: TreatMissingData.IGNORE,
    datapointsToAlarm: 3
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


  new CfnOutput(stack, 'Output-ThingArn', {
    value: thingArn,
  });

  new CfnOutput(stack, 'Output-CertId', {
    value: certId,
  });

  new CfnOutput(stack, 'Output-CertPem', {
    value: certPem,
  });

  new CfnOutput(stack, 'Output-PrivKey', {
    value: privKey,
  });
}
