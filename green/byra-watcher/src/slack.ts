import { IncomingWebhook } from "@slack/webhook";
import { EventBridgeHandler } from "aws-lambda";
import { Config } from "sst/node/config";

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


export const handler: EventBridgeHandler<'msg', CloudWatchAlarmDetail, void> = async (event) => {
  console.info(`Received event: ${JSON.stringify(event)}`);

  await new IncomingWebhook(Config.SLACK_URL).send({
    text:
      event.detail.state.value === "ALARM"
        ? `${WARN_EMOJI} CRITICAL: ${BEER_EMOJI} Beer count is low`
        : `${HAPPY_EMOJI} ALL GOOD: ${BEER_EMOJI} We have beer!`,
  });
};
