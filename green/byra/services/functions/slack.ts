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
const HAPPY_EMOJI = "🎉";

const SLACK_WEBHOOK_URL = process.env.SLACK_WEBHOOK_URL ?? "n/a";

export const handler = async (event: any) => {
  const webhook = new IncomingWebhook(SLACK_WEBHOOK_URL);

  console.log(`Received event: ${JSON.stringify(event)}`);

  const alarmDetail = event.detail as CloudWatchAlarmDetail;

  if (alarmDetail.state.value !== "ALARM") {
    await webhook.send({
      text: `${HAPPY_EMOJI} ALL GOOD: ${BEER_EMOJI}, We have beer!`,
    });

    return;
  }

  await webhook.send({
    text: `${WARN_EMOJI} CRITICAL: ${BEER_EMOJI} Beer count is low`,
  });
};
