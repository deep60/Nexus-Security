import http from "k6/http";
import { check, sleep } from "k6";

export const options = {
  stages: [
    { duration: "1m", target: 25 },
    { duration: "2m", target: 50 },
    { duration: "2m", target: 75 },
    { duration: "1m", target: 0 },
  ],
  thresholds: {
    http_req_failed: ["rate<0.10"],
    http_req_duration: ["p(99)<2000"],
  },
};

const baseUrl = __ENV.VERDYX_API_BASE_URL || "http://localhost:8080";

export default function () {
  const res = http.get(`${baseUrl}/health`);
  check(res, {
    "health survives stress": (r) => r.status >= 200 && r.status < 500,
  });

  sleep(1);
}
