export type AppErrorCategory =
  | "permission"
  | "notFound"
  | "invalidShortcut"
  | "busy"
  | "network"
  | "database"
  | "canceled"
  | "unknown";

export const getErrorDetail = (error: unknown): string => {
  if (typeof error === "string") {
    return error;
  }

  if (error instanceof Error) {
    return error.message || error.name;
  }

  if (error === null || error === undefined) {
    return "";
  }

  try {
    return JSON.stringify(error);
  } catch {
    return String(error);
  }
};

const includesAny = (value: string, patterns: string[]) =>
  patterns.some((pattern) => value.includes(pattern));

export const classifyError = (error: unknown): AppErrorCategory => {
  const detail = getErrorDetail(error).toLowerCase();

  if (
    includesAny(detail, [
      "cancelled",
      "canceled",
      "operation canceled",
      "operation cancelled",
    ])
  ) {
    return "canceled";
  }

  if (
    includesAny(detail, [
      "invalid shortcut",
      "invalid accelerator",
      "accelerator parse",
      "failed to parse shortcut",
    ])
  ) {
    return "invalidShortcut";
  }

  if (
    includesAny(detail, [
      "permission denied",
      "access denied",
      "access is denied",
      "eacces",
      "eperm",
      "権限",
      "アクセスが拒否",
    ])
  ) {
    return "permission";
  }

  if (
    includesAny(detail, [
      "executable not found",
      "not found",
      "no such file",
      "path does not exist",
      "指定されたファイルが見つかりません",
      "指定されたパスが見つかりません",
      "見つかりません",
    ])
  ) {
    return "notFound";
  }

  if (
    includesAny(detail, [
      "already registered",
      "already exists",
      "in use",
      "being used",
      "使用中",
      "既に登録",
    ])
  ) {
    return "busy";
  }

  if (
    includesAny(detail, [
      "network",
      "fetch",
      "timeout",
      "timed out",
      "connection",
      "dns",
      "http",
      "request",
    ])
  ) {
    return "network";
  }

  if (
    includesAny(detail, [
      "sqlite",
      "sqlx",
      "database",
      "db error",
      "データベース",
    ])
  ) {
    return "database";
  }

  return "unknown";
};

export const getFriendlyErrorMessage = (
  error: unknown,
  fallbackMessage: string,
): string => {
  switch (classifyError(error)) {
    case "permission":
      return `${fallbackMessage}。ファイルやフォルダへのアクセス権限を確認してください。`;
    case "notFound":
      return `${fallbackMessage}。対象のファイルまたはフォルダが見つかりません。パスを設定し直してください。`;
    case "invalidShortcut":
      return `${fallbackMessage}。ショートカットキーの形式を確認してください。例: Ctrl+Shift+L`;
    case "busy":
      return `${fallbackMessage}。同じショートカットやファイルが別の機能・アプリで使用中の可能性があります。`;
    case "network":
      return `${fallbackMessage}。ネットワーク接続や外部サービスの状態を確認してください。`;
    case "database":
      return `${fallbackMessage}。データベースの読み書きに失敗しました。アプリを再起動して再度試してください。`;
    case "canceled":
      return "操作をキャンセルしました。";
    case "unknown":
      return `${fallbackMessage}。詳細は開発者ログに記録しました。`;
  }
};

const writeDiagnosticLog = (context: string, detail: string) => {
  if (typeof window === "undefined" || !(window as any).__TAURI_INTERNALS__) {
    return;
  }

  void import("@tauri-apps/api/core")
    .then(({ invoke }) =>
      invoke("app_log", {
        level: "error",
        message: `[frontend.${context}] ${detail || "Unknown error"}`,
      }),
    )
    .catch((logError) => {
      console.warn("[app_log] failed to write diagnostic log", logError);
    });
};

export const reportError = (context: string, error: unknown) => {
  const detail = getErrorDetail(error);
  console.error(`[${context}] ${detail || "Unknown error"}`, error);
  writeDiagnosticLog(context, detail);
};

export const setupGlobalErrorLogging = () => {
  const handleError = (event: ErrorEvent) => {
    reportError("window.error", event.error ?? event.message);
  };

  const handleUnhandledRejection = (event: PromiseRejectionEvent) => {
    reportError("window.unhandledrejection", event.reason);
  };

  window.addEventListener("error", handleError);
  window.addEventListener("unhandledrejection", handleUnhandledRejection);

  return () => {
    window.removeEventListener("error", handleError);
    window.removeEventListener("unhandledrejection", handleUnhandledRejection);
  };
};
