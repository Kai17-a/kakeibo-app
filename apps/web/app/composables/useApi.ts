export function apiErrorMessage(error: unknown): string {
  if (error && typeof error === "object" && "data" in error) {
    const data = error.data;
    if (data && typeof data === "object" && "message" in data && typeof data.message === "string") {
      return data.message;
    }
  }
  return "通信に失敗しました。接続を確認して、もう一度お試しください。";
}
