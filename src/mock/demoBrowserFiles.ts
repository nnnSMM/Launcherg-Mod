import { isSupportedGamePath } from "@/mock/demoGameMatching";
import { rememberDemoFile } from "@/mock/demoIconExtraction";

type DemoFileHandle = {
  kind: "file";
  name: string;
  getFile?: () => Promise<File>;
};

type DemoDirectoryHandle = {
  kind: "directory";
  name: string;
  entries: () => AsyncIterableIterator<[string, DemoFileHandle | DemoDirectoryHandle]>;
};

type DemoWindow = Window & {
  showDirectoryPicker?: (options?: { mode?: "read" | "readwrite" }) => Promise<DemoDirectoryHandle>;
  showOpenFilePicker?: (options?: unknown) => Promise<DemoFileHandle[]>;
};

export type DemoScannedFile = {
  path: string;
};

const directoryHandles = new Map<string, DemoDirectoryHandle>();
const directoryFileLists = new Map<string, string[]>();
const pickedFiles = new Set<string>();

const toDemoWindow = () => window as DemoWindow;

const COMMON_GAME_EXECUTABLE_NAMES = [
  "SiglusEngine.exe",
  "BGI.exe",
  "game.exe",
  "start.exe",
  "Start.exe",
];

const uniqueVirtualPath = (scheme: string, name: string, existing: Set<string> | Map<string, unknown>) => {
  const safeName = name.replace(/[\\/]/g, "_") || "selected";
  let path = `${scheme}://${safeName}`;
  let index = 2;
  while (existing.has(path)) {
    path = `${scheme}://${safeName}-${index}`;
    index++;
  }
  return path;
};

export const pickDemoDirectory = async () => {
  const picker = toDemoWindow().showDirectoryPicker;
  if (picker) {
    try {
      const handle = await picker({ mode: "read" });
      const virtualPath = uniqueVirtualPath("browser-fs", handle.name, directoryHandles);
      directoryHandles.set(virtualPath, handle);
      return virtualPath;
    } catch (e) {
      console.warn("[Mock Browser FS] directory selection cancelled", e);
      return null;
    }
  }

  return await pickDirectoryWithFileInput();
};

export const pickDemoFile = async () => {
  const picker = toDemoWindow().showOpenFilePicker;
  if (picker) {
    try {
      const [handle] = await picker({
        multiple: false,
        types: [
          {
            description: "Executable or shortcut",
            accept: {
              "application/octet-stream": [".exe", ".lnk", ".url"],
            },
          },
        ],
      });
      if (!handle) {
        return null;
      }
      const virtualPath = uniqueVirtualPath("browser-file", handle.name, pickedFiles);
      pickedFiles.add(virtualPath);
      return `${virtualPath}\\${handle.name}`;
    } catch (e) {
      console.warn("[Mock Browser FS] file selection cancelled", e);
      return null;
    }
  }

  return await pickFileWithInput();
};

const chooseInputFiles = (configure: (input: HTMLInputElement) => void) =>
  new Promise<FileList | null>((resolve) => {
    const input = document.createElement("input");
    let settled = false;
    const finish = (files: FileList | null) => {
      if (settled) {
        return;
      }
      settled = true;
      window.removeEventListener("focus", onFocus);
      input.remove();
      resolve(files?.length ? files : null);
    };
    const onFocus = () => {
      window.setTimeout(() => finish(input.files), 500);
    };

    input.type = "file";
    input.style.position = "fixed";
    input.style.left = "-9999px";
    input.style.top = "-9999px";
    configure(input);
    input.addEventListener("change", () => {
      finish(input.files);
    });
    document.body.append(input);
    window.addEventListener("focus", onFocus);
    input.click();
  });

const pickDirectoryWithFileInput = async () => {
  const files = await chooseInputFiles((input) => {
    input.multiple = true;
    input.setAttribute("webkitdirectory", "");
    input.setAttribute("directory", "");
  });
  if (!files) {
    return null;
  }

  const entries = Array.from(files).map((file) => {
    const relativePath = (file as File & { webkitRelativePath?: string })
      .webkitRelativePath;
    return {
      file,
      path: relativePath || file.name,
    };
  });
  const rootName = entries[0]?.path.split(/[\\/]/)[0] || "selected-folder";
  const virtualPath = uniqueVirtualPath("browser-fs", rootName, directoryFileLists);
  for (const entry of entries) {
    rememberDemoFile(`${virtualPath}\\${entry.path}`, entry.file);
  }
  directoryFileLists.set(
    virtualPath,
    entries.map((entry) => `${virtualPath}\\${entry.path}`),
  );
  return virtualPath;
};

const pickFileWithInput = async () => {
  const files = await chooseInputFiles((input) => {
    input.multiple = false;
    input.accept = ".exe,.lnk,.url";
  });
  const file = files?.[0];
  if (!file) {
    return null;
  }
  const virtualPath = uniqueVirtualPath("browser-file", file.name, pickedFiles);
  pickedFiles.add(virtualPath);
  return `${virtualPath}\\${file.name}`;
};

const walkDirectory = async (
  handle: DemoDirectoryHandle,
  pathParts: string[],
  files: DemoScannedFile[],
  maxFiles: number,
) => {
  for await (const [name, child] of handle.entries()) {
    if (files.length >= maxFiles) {
      return;
    }
    if (child.kind === "file") {
      const path = [...pathParts, name].join("\\");
      const file = child.getFile ? await child.getFile().catch(() => null) : null;
      rememberDemoFile(path, file);
      if (isSupportedGamePath(path)) {
        files.push({ path });
      }
      continue;
    }
    await walkDirectory(child, [...pathParts, name], files, maxFiles);
  }
};

const splitPath = (path: string) => path.split(/[\\/]/).filter(Boolean);

const getLastPathPart = (path: string) => splitPath(path).at(-1) ?? "";

const getTypedDirectoryScanCandidates = (directoryPath: string) => {
  if (isSupportedGamePath(directoryPath)) {
    return [];
  }

  const directoryName = getLastPathPart(directoryPath);
  const candidateNames = [
    ...COMMON_GAME_EXECUTABLE_NAMES,
    directoryName ? `${directoryName}.exe` : "",
  ].filter(Boolean);

  return candidateNames.map((name) => ({
    path: `${directoryPath.replace(/[\\/]+$/, "")}\\${name}`,
  }));
};

export const scanDemoPaths = async (paths: string[], maxFiles = 5000) => {
  const files: DemoScannedFile[] = [];

  for (const path of paths.filter(Boolean)) {
    const handle = directoryHandles.get(path);
    if (handle) {
      await walkDirectory(handle, [path], files, maxFiles);
      continue;
    }
    const storedFiles = directoryFileLists.get(path);
    if (storedFiles) {
      files.push(
        ...storedFiles
          .filter(isSupportedGamePath)
          .slice(0, Math.max(0, maxFiles - files.length))
          .map((storedPath) => ({ path: storedPath })),
      );
      continue;
    }
    files.push(...getTypedDirectoryScanCandidates(path));
  }

  return files.slice(0, maxFiles);
};
