import http from "k6/http";
import { check, sleep } from "k6";

export const options = {
  vus: 1,
  duration: "30s",
  thresholds: {
    http_req_failed: ["rate<0.01"],
    http_req_duration: ["p(95)<750"],
  },
};

const baseUrl = __ENV.VERDYX_API_BASE_URL || "http://localhost:8080";

export default function () {
  const res = http.get(`${baseUrl}/health`);

  check(res, {
    "health returns 2xx": (r) => r.status >= 200 && r.status < 300,
  });

  sleep(1);
}
