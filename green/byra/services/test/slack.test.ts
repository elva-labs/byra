import { CloudWatchAlarmDetail, handler } from "./../functions/slack";
import { describe, it, expect } from "vitest";

describe("send to slack", () => {
  it("on state alarm", async () => {
    const alarm: CloudWatchAlarmDetail = {
      state: {
        value: "ALARM",
      },
      message: "Threshhold reached",
      alarmName: "Beer Alarm",
    };
    const event = {
      source: "aws.cloudwatch",
      detailType: "Alarm State Change",
      detail: alarm,
    };

    await handler(event);

    expect(true).toBe(true);
  });
});
