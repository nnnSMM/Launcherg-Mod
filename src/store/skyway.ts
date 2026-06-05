import {
  SkyWayContext,
  SkyWayRoom,
  SkyWayStreamFactory,
  uuidV4,
  type RoomPublication,
  type LocalStream,
  type LocalDataStream,
  type RemoteRoomMember,
} from "@skyway-sdk/room";
import { memo } from "../store/memo";
import { createWritable } from "@/lib/utils";
import { fetch } from "@tauri-apps/plugin-http";
import { readFile } from "@tauri-apps/plugin-fs";
import {
  commandGetPauseState,
  commandSaveScreenshotByPid,
  commandTogglePauseTracking,
} from "@/lib/command";
import { getStartProcessMap } from "@/store/startProcessMap";
import { showErrorToast } from "@/lib/toast";
import { getFriendlyErrorMessage, reportError } from "@/lib/errors";
import { useChunk } from "@/lib/chunk";
import {
  parseRemoteMessage,
  type ControlStatusMessage,
  type ImageMetadataMessage,
  type InitResponseMessage,
  type LibraryResponseMessage,
  type LocalMessage,
  type MemoMessage,
  type PingMessage,
  type RemoteGameSummary,
} from "@/store/skywayMessage";
import {
  createMobileCompanionUrl,
  SKYWAY_CONNECT_ENDPOINT,
} from "@/lib/mobileCompanionUrl";
import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
import type { CollectionElement } from "@/lib/types";

const createSkyWay = () => {
  const roomId = uuidV4();
  const sentImagePathSet = new Set<string>();
  const { createChunks } = useChunk();

  const sendImagesAsChunks = async (imagePaths: string[]) => {
    await Promise.all(
      imagePaths.map(async (path) => {
        const [{ chunkId, mimeType, totalChunkLength }, chunks] =
          await createChunks(path);
        const message: ImageMetadataMessage = {
          type: "image_metadata",
          path,
          key: chunkId,
          totalChunkLength,
          mimeType,
        };
        sendMessage(message);
        chunks.forEach(sendBinaryMessage);
      }),
    );
  };

  const getMemoImagePaths = (text: string) => {
    const regex = /!\[.*?\]\((.*?)\)/g;
    const paths: string[] = [];
    let match: RegExpExecArray | null = null;
    while ((match = regex.exec(text)) !== null) {
      paths.push(match[1]);
    }

    return paths;
  };
  const getMemo = (gameId: number): { value: string; imagePaths: string[] } => {
    const memoKey = `smde_memo-${gameId}`;
    const memo = localStorage.getItem(memoKey) ?? "";

    const paths = getMemoImagePaths(memo);

    return { value: memo, imagePaths: paths };
  };
  const setRemoteMemo = (workId: number, text: string) => {
    const memoKey = `smde_memo-${workId}`;
    localStorage.setItem(memoKey, text);

    memo.update((v) => {
      // 開いてないときはわざわざ store に入れない
      if (!v.find((v) => v.workId === workId)) {
        return v;
      }
      return v.map((v) =>
        v.workId === workId ? { ...v, value: text, lastModified: "remote" } : v
      );
    });
  };

  const createInitResponseMessage = async (workId: number) => {
    const { value, imagePaths } = getMemo(workId);
    imagePaths.forEach((path) => sentImagePathSet.add(path));
    await sendImagesAsChunks(imagePaths);

    const message: InitResponseMessage = {
      type: "init_response",
      gameId: workId,
      initialMemo: {
        type: "memo",
        text: value,
        gameId: workId,
      },
    };
    return message;
  };

  const toRemoteGameSummary = (
    element: CollectionElement,
  ): RemoteGameSummary => ({
    id: element.id,
    title: element.gamename,
    brandName: element.brandname,
    playStatus: element.playStatus,
    totalPlayTimeSeconds: element.totalPlayTimeSeconds,
    lastPlayAt: element.lastPlayAt,
    installed: !!(element.exePath || element.lnkPath),
    liked: !!element.likeAt,
  });

  const createLibraryResponseMessage =
    async (): Promise<LibraryResponseMessage> => {
      await sidebarCollectionElements.refetch();
      return {
        type: "library_response",
        games: sidebarCollectionElements.value().map(toRemoteGameSummary),
      };
    };

  const createControlStatusMessage =
    async (error?: string): Promise<ControlStatusMessage> => ({
      type: "control_status",
      isPaused: await commandGetPauseState(),
      error,
    });

  const cleanupFuncs: (() => void)[] = [];
  const cleanup = () => {
    cleanupFuncs.forEach((func) => func());
  };

  let dataStream: LocalDataStream | undefined = undefined;
  let currentAuthToken: string | undefined = undefined;
  const setDataStream = async () => {
    const response = await fetch(SKYWAY_CONNECT_ENDPOINT, {
      method: "POST",
      headers: {
        "content-type": "application/json",
      },
    });
    const { authToken } = (await response.json()) as { authToken: string };
    currentAuthToken = authToken;

    const context = await SkyWayContext.Create(authToken);
    const room = await SkyWayRoom.FindOrCreate(context, {
      type: "p2p",
      name: roomId,
    });
    const me = await room.join();
    me.onFatalError.add(() => {
      dataStream = undefined;
      cleanup();
      showErrorToast("接続が切断されました。");
    });

    const onPublicate = async (publication: RoomPublication<LocalStream>) => {
      if (publication.publisher.id === me.id) return;
      if (publication.contentType !== "data") return;

      const { stream } = await me.subscribe(publication.id);
      if (stream.contentType !== "data") return;

      const { removeListener } = stream.onData.add(async (data) => {
        if (typeof data !== "string") return;

        const message = parseRemoteMessage(data);
        if (!message) return;

        if (message.type !== "ping") {
          console.log("receive message", message);
        }
        switch (message.type) {
          case "ping":
            return;
          case "memo":
            setRemoteMemo(message.gameId, message.text);
            return;
          case "init": {
            const response = await createInitResponseMessage(message.gameId);
            sendMessage(response);
            break;
          }
          case "library_request": {
            const response = await createLibraryResponseMessage();
            sendMessage(response);
            break;
          }
          case "control_status_request": {
            const response = await createControlStatusMessage();
            sendMessage(response);
            break;
          }
          case "pause_toggle":
            try {
              const isPaused = await commandTogglePauseTracking();
              sendMessage({ type: "control_status", isPaused });
            } catch (e) {
              reportError("skyway.pause.toggle", e);
              const response = await createControlStatusMessage(
                getFriendlyErrorMessage(e, "Pauseの切り替えに失敗しました"),
              );
              sendMessage(response);
            }
            break;
          case "take_screenshot":
            try {
              const processId = getStartProcessMap()[message.gameId];
              if (processId === undefined) {
                throw new Error("対象ゲームの起動プロセスが見つかりません");
              }
              const imagePath = await commandSaveScreenshotByPid(
                message.gameId,
                processId
              );
              const prev = getMemo(message.gameId).value;
              const lines = prev.split("\n");
              const newLines: string[] = [];
              for (let i = 0; i < lines.length; i++) {
                newLines.push(lines[i]);
                if (i === message.cursorLine) {
                  newLines.push(`![](${imagePath})`);
                  newLines.push("");
                }
              }
              const newMemo = newLines.join("\n");
              setRemoteMemo(message.gameId, newMemo);
              await syncMemo(message.gameId, newMemo);
              sendMessage({
                type: "screenshot_result",
                gameId: message.gameId,
                ok: true,
                imagePath,
              });
            } catch (e) {
              reportError("skyway.screenshot.capture", e);
              const error = getFriendlyErrorMessage(
                e,
                "スクリーンショットの取得に失敗しました",
              );
              showErrorToast(error);
              sendMessage({
                type: "screenshot_result",
                gameId: message.gameId,
                ok: false,
                error,
              });
            }
        }
      });
      cleanupFuncs.push(removeListener);

      // PC側の準備が完了したら subscribe させる
      await (publication.publisher as RemoteRoomMember).subscribe(
        myPublication.id
      );
    };

    dataStream = await SkyWayStreamFactory.createDataStream();
    const myPublication = await me.publish(dataStream);

    const pingTimer = setInterval(() => {
      if (!dataStream) return;
      const message: PingMessage = { type: "ping" };
      sendMessage(message);
    }, 10000);
    cleanupFuncs.push(() => clearInterval(pingTimer));

    room.publications.forEach(onPublicate);
    room.onStreamPublished.add((e) => onPublicate(e.publication));
  };

  const connect = async (workId?: number, seiyaUrl = "") => {
    if (!dataStream) {
      await setDataStream();
    }
    return createMobileCompanionUrl({
      roomId,
      gameId: workId,
      seiyaUrl,
      authToken: currentAuthToken,
    });
  };

  const sendMessage = (message: LocalMessage) => {
    if (!dataStream) return;
    if (message.type !== "ping") {
      console.log("send message", message);
    }

    dataStream.write(JSON.stringify(message));
  };
  const sendBinaryMessage = (message: Uint8Array) => {
    if (!dataStream) return;

    dataStream.write(message);
  };

  const syncMemo = async (workId: number, text: string) => {
    if (!dataStream) return;
    const imagePaths = getMemoImagePaths(text);
    const notSharedImages = imagePaths.filter(
      (path) => !sentImagePathSet.has(path)
    );
    await sendImagesAsChunks(notSharedImages);

    const message: MemoMessage = {
      type: "memo",
      text,
      gameId: workId,
    };
    sendMessage(message);
  };

  return { connect, syncMemo, cleanup, roomId };
};

export const skyWay = createSkyWay();
