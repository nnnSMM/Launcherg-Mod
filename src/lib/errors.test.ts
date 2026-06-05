import { describe, expect, it, vi } from "vitest";
import {
  classifyError,
  getErrorDetail,
  getFriendlyErrorMessage,
  reportError,
} from "./errors";

describe("classifyError", () => {
  it("実行ファイルなしを notFound に分類する", () => {
    expect(classifyError("executable not found")).toBe("notFound");
  });

  it("ショートカット形式の失敗を invalidShortcut に分類する", () => {
    expect(classifyError("invalid shortcut key `Ctrl+`")).toBe(
      "invalidShortcut",
    );
  });

  it("アクセス拒否を permission に分類する", () => {
    expect(classifyError(new Error("Access is denied"))).toBe("permission");
  });

  it("SQLite の失敗を database に分類する", () => {
    expect(classifyError("sqlite error: database is locked")).toBe("database");
  });
});

describe("getFriendlyErrorMessage", () => {
  it("見つからないエラーではパス再設定を促す", () => {
    expect(
      getFriendlyErrorMessage(
        "executable not found",
        "ゲームの起動に失敗しました",
      ),
    ).toContain("パスを設定し直してください");
  });

  it("未知のエラーでは詳細ログを案内する", () => {
    expect(
      getFriendlyErrorMessage("unexpected failure", "処理に失敗しました"),
    ).toBe("処理に失敗しました。詳細は開発者ログに記録しました。");
  });
});

describe("getErrorDetail", () => {
  it("Error の message を返す", () => {
    expect(getErrorDetail(new Error("failed"))).toBe("failed");
  });

  it("オブジェクトは JSON にして返す", () => {
    expect(getErrorDetail({ code: "E_TEST" })).toBe('{"code":"E_TEST"}');
  });
});

describe("reportError", () => {
  it("文脈と詳細を console.error に残す", () => {
    const spy = vi.spyOn(console, "error").mockImplementation(() => {});

    reportError("test.context", new Error("failed"));

    expect(spy).toHaveBeenCalledWith(
      "[test.context] failed",
      expect.any(Error),
    );
    spy.mockRestore();
  });
});
