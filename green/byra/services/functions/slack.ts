import { IncomingWebhook } from "@slack/webhook";

export interface CloudWatchAlarmDetail {
  state: {
    value: string;
  };
  message: string;
  alarmName: string;
}

const BEER_EMOJI = "🍺";
const WARN_EMOJI = "🔥";

const SLACK_WEBHOOK_URL =
  "https://hooks.slack.com/services/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";

export const handler = async (event: any) => {
  console.log(`Received event: ${JSON.stringify(event)}`);

  if (
    event.source === "aws.cloudwatch" &&
    event.detailType === "Alarm State Change"
  ) {
    const alarmDetail = event.detail as CloudWatchAlarmDetail;
    if (alarmDetail.state.value === "ALARM") {
      const webhook = new IncomingWebhook(SLACK_WEBHOOK_URL);
      const result = await webhook.send({
        text: `${WARN_EMOJI} CRITICAL: ${BEER_EMOJI} Beer count is low`,
      });
      console.log(result);
    }
  }
};
