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
  commandSaveFullscreenScreenshot,
  commandSendRightClick,
  commandTogglePauseTracking,
  commandGetTrackingState,
} from "@/lib/command";
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
import { getOrCreateMobileCompanionRoomId } from "@/lib/mobileCompanionRoom";
import { sidebarCollectionElements } from "@/store/sidebarCollectionElements";
import type { CollectionElement } from "@/lib/types";

const createSkyWay = () => {
  const roomId = getOrCreateMobileCompanionRoomId(uuidV4);
  const sentImagePathSet = new Set<string>();
  let libraryThumbnailPathSet = new Set<string>();
  let imageSendQueue = Promise.resolve();
  const { createChunks } = useChunk();

  const sendImagesAsChunks = async (
    imagePaths: string[],
    options: { skipSent?: boolean; rememberSent?: boolean } = {},
  ) => {
    const { skipSent = true, rememberSent = true } = options;
    const uniqueImagePaths = Array.from(
      new Set(
        imagePaths.filter(
          (path) => path && (!skipSent || !sentImagePathSet.has(path)),
        ),
      ),
    );
    for (const path of uniqueImagePaths) {
      try {
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
        if (rememberSent) {
          sentImagePathSet.add(path);
        }
        await wait(16);
      } catch (error) {
        reportError("skyway.image.send", error);
      }
    }
  };

  const queueImagesAsChunks = (
    imagePaths: string[],
    options?: { skipSent?: boolean; rememberSent?: boolean },
  ) => {
    imageSendQueue = imageSendQueue
      .catch((error) => {
        reportError("skyway.image.queue", error);
      })
      .then(() => sendImagesAsChunks(imagePaths, options));
    return imageSendQueue;
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
    await queueImagesAsChunks(imagePaths);

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
    thumbnailPath: element.thumbnail && element.thumbnail.trim() !== "" ? element.thumbnail.trim() : null,
    thumbnailWidth: element.thumbnailWidth,
    thumbnailHeight: element.thumbnailHeight,
  });

  const createLibraryResponseMessage =
    async (): Promise<LibraryResponseMessage> => {
      await sidebarCollectionElements.refetch();
      const games = sidebarCollectionElements.value().map(toRemoteGameSummary);
      libraryThumbnailPathSet = new Set(
        games
          .map((game) => game.thumbnailPath)
          .filter((path): path is string => !!path),
      );
      return {
        type: "library_response",
        games,
      };
    };

  const sendRequestedThumbnails = async (paths: string[]) => {
    const thumbnailPaths = paths.filter((path) =>
      libraryThumbnailPathSet.has(path),
    );
    await queueImagesAsChunks(thumbnailPaths, {
      skipSent: false,
      rememberSent: false,
    });
  };

  const wait = (milliseconds: number) =>
    new Promise((resolve) => setTimeout(resolve, milliseconds));

  const createControlStatusMessage =
    async (error?: string): Promise<ControlStatusMessage> => {
      const state = await commandGetTrackingState();
      return {
        type: "control_status",
        isPaused: state.isPaused,
        isTracking: state.isTracking,
        activeGameId: state.activeGameId,
        activeProcessId: state.activeProcessId,
        error,
      };
    };

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

      sentImagePathSet.clear();
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
          case "thumbnail_request": {
            void sendRequestedThumbnails(message.paths);
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
              const state = await commandGetTrackingState();
              sendMessage({
                type: "control_status",
                isPaused,
                isTracking: state.isTracking,
                activeGameId: state.activeGameId,
                activeProcessId: state.activeProcessId,
              });
            } catch (e) {
              reportError("skyway.pause.toggle", e);
              const response = await createControlStatusMessage(
                getFriendlyErrorMessage(e, "Pauseの切り替えに失敗しました"),
              );
              sendMessage(response);
            }
            break;
          case "take_screenshot": {
            let didHideText = false;
            try {
              if (message.hideText) {
                await commandSendRightClick();
                didHideText = true;
                await wait(400);
              }
              const imagePath = await commandSaveFullscreenScreenshot(message.gameId);
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
            } finally {
              if (didHideText) {
                await wait(120);
                try {
                  await commandSendRightClick();
                } catch (e) {
                  reportError("skyway.screenshot.restore_text", e);
                }
              }
            }
            break;
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
      mode: "controller",
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

    dataStream.write(
      message.buffer.slice(
        message.byteOffset,
        message.byteOffset + message.byteLength,
      ),
    );
  };

  const syncMemo = async (workId: number, text: string) => {
    if (!dataStream) return;
    const imagePaths = getMemoImagePaths(text);
    const notSharedImages = imagePaths.filter(
      (path) => !sentImagePathSet.has(path)
    );
    await queueImagesAsChunks(notSharedImages);

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
