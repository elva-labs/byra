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
  console.log(`Received event: ${JSON.stringify(event)}`);

  const alarmDetail = event.detail as CloudWatchAlarmDetail;

  await new IncomingWebhook(SLACK_WEBHOOK_URL).send({
    text:
      alarmDetail.state.value === "ALARM"
        ? `${WARN_EMOJI} CRITICAL: ${BEER_EMOJI} Beer count is low`
        : `${HAPPY_EMOJI} ALL GOOD: ${BEER_EMOJI} We have beer!`,
  });
};
