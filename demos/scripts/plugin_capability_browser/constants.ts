export const LOCAL_PROBE_TIMEOUT_SECONDS = 8;
export const LOCAL_PROBE_SUCCESS_LIMIT = 6;
export const LOCAL_PROBE_ATTEMPT_LIMIT = 18;

export const INTERACTIVE_SERVER_FOLLOWUP_LIMITS: Record<string, number> = {
  clap: 4,
  vst3: 6,
  au: 4,
  lv2: 4,
};
