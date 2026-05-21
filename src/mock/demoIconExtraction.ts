const RT_ICON = 3;
const RT_GROUP_ICON = 14;

const fileByPath = new Map<string, File>();
const dataUrls = new Map<string, string>();

const extname = (path: string) => {
  const dot = path.lastIndexOf(".");
  return dot >= 0 ? path.slice(dot + 1).toLowerCase() : "";
};

const dirname = (path: string) => {
  const index = Math.max(path.lastIndexOf("\\"), path.lastIndexOf("/"));
  return index >= 0 ? path.slice(0, index) : "";
};

const isInDirectory = (filePath: string, dir: string) => {
  const prefix = dir ? `${dir}\\` : "";
  return filePath.startsWith(prefix) || filePath.startsWith(prefix.replace(/\\/g, "/"));
};

export const rememberDemoFile = (path: string, file: File | null | undefined) => {
  if (file) {
    fileByPath.set(path, file);
  }
};

const blobToDataUrl = async (blob: Blob) =>
  await new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result ?? ""));
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(blob);
  });

const getDataUrl = async (key: string, blob: Blob) => {
  const existing = dataUrls.get(key);
  if (existing) {
    return existing;
  }
  const url = await blobToDataUrl(blob);
  dataUrls.set(key, url);
  return url;
};

const findSiblingIco = (filePath: string) => {
  const dir = dirname(filePath);
  for (const [path, file] of fileByPath) {
    if (extname(path) === "ico" && isInDirectory(path, dir)) {
      return { path, file };
    }
  }
  return null;
};

const rvaToOffset = (
  rva: number,
  sections: Array<{ virtualAddress: number; virtualSize: number; rawSize: number; rawPointer: number }>,
) => {
  for (const section of sections) {
    const size = Math.max(section.virtualSize, section.rawSize);
    if (rva >= section.virtualAddress && rva < section.virtualAddress + size) {
      return section.rawPointer + (rva - section.virtualAddress);
    }
  }
  return null;
};

const readResourceDirectory = (view: DataView, offset: number) => {
  const namedCount = view.getUint16(offset + 12, true);
  const idCount = view.getUint16(offset + 14, true);
  const entries = [];
  const count = namedCount + idCount;
  for (let i = 0; i < count; i++) {
    const entryOffset = offset + 16 + i * 8;
    entries.push({
      id: view.getUint32(entryOffset, true) & 0xffff,
      dataOffset: view.getUint32(entryOffset + 4, true),
    });
  }
  return entries;
};

const findResourceEntry = (view: DataView, offset: number, id: number) =>
  readResourceDirectory(view, offset).find((entry) => entry.id === id) ?? null;

const collectResourceLeaves = (
  view: DataView,
  resourceBase: number,
  offset: number,
  leaves: number[] = [],
) => {
  for (const entry of readResourceDirectory(view, offset)) {
    const value = entry.dataOffset;
    const relativeOffset = value & 0x7fffffff;
    if (value & 0x80000000) {
      collectResourceLeaves(view, resourceBase, resourceBase + relativeOffset, leaves);
    } else {
      leaves.push(resourceBase + relativeOffset);
    }
  }
  return leaves;
};

const getResourceBytes = (
  view: DataView,
  resourceDataEntryOffset: number,
  sections: Array<{ virtualAddress: number; virtualSize: number; rawSize: number; rawPointer: number }>,
) => {
  const dataRva = view.getUint32(resourceDataEntryOffset, true);
  const size = view.getUint32(resourceDataEntryOffset + 4, true);
  const fileOffset = rvaToOffset(dataRva, sections);
  if (fileOffset === null || fileOffset + size > view.byteLength) {
    return null;
  }
  return new Uint8Array(view.buffer.slice(fileOffset, fileOffset + size));
};

const extractIcoFromExe = async (file: File) => {
  const buffer = await file.arrayBuffer();
  const view = new DataView(buffer);
  if (view.byteLength < 0x40 || view.getUint16(0, true) !== 0x5a4d) {
    return null;
  }

  const peOffset = view.getUint32(0x3c, true);
  if (peOffset + 24 >= view.byteLength || view.getUint32(peOffset, true) !== 0x4550) {
    return null;
  }

  const sectionCount = view.getUint16(peOffset + 6, true);
  const optionalHeaderSize = view.getUint16(peOffset + 20, true);
  const optionalHeaderOffset = peOffset + 24;
  const magic = view.getUint16(optionalHeaderOffset, true);
  const dataDirectoryOffset = optionalHeaderOffset + (magic === 0x20b ? 112 : 96);
  const resourceRva = view.getUint32(dataDirectoryOffset + 8 * 2, true);
  if (!resourceRva) {
    return null;
  }

  const sectionOffset = optionalHeaderOffset + optionalHeaderSize;
  const sections: Array<{
    virtualAddress: number;
    virtualSize: number;
    rawSize: number;
    rawPointer: number;
  }> = [];
  for (let i = 0; i < sectionCount; i++) {
    const offset = sectionOffset + i * 40;
    sections.push({
      virtualSize: view.getUint32(offset + 8, true),
      virtualAddress: view.getUint32(offset + 12, true),
      rawSize: view.getUint32(offset + 16, true),
      rawPointer: view.getUint32(offset + 20, true),
    });
  }

  const resourceBase = rvaToOffset(resourceRva, sections);
  if (resourceBase === null) {
    return null;
  }

  const groupTypeEntry = findResourceEntry(view, resourceBase, RT_GROUP_ICON);
  const iconTypeEntry = findResourceEntry(view, resourceBase, RT_ICON);
  if (!groupTypeEntry || !iconTypeEntry) {
    return null;
  }

  const groupLeaves = collectResourceLeaves(
    view,
    resourceBase,
    resourceBase + (groupTypeEntry.dataOffset & 0x7fffffff),
  );
  const groupBytes = groupLeaves
    .map((offset) => getResourceBytes(view, offset, sections))
    .find((bytes): bytes is Uint8Array => Boolean(bytes));
  if (!groupBytes || groupBytes.length < 6) {
    return null;
  }

  const groupView = new DataView(groupBytes.buffer);
  const count = groupView.getUint16(4, true);
  if (!count) {
    return null;
  }

  const iconLeaves = collectResourceLeaves(
    view,
    resourceBase,
    resourceBase + (iconTypeEntry.dataOffset & 0x7fffffff),
  );
  const iconBytesById = new Map<number, Uint8Array>();
  for (const iconIdEntry of readResourceDirectory(
    view,
    resourceBase + (iconTypeEntry.dataOffset & 0x7fffffff),
  )) {
    const leaves = collectResourceLeaves(
      view,
      resourceBase,
      resourceBase + (iconIdEntry.dataOffset & 0x7fffffff),
    );
    const bytes = leaves
      .map((offset) => getResourceBytes(view, offset, sections))
      .find((value): value is Uint8Array => Boolean(value));
    if (bytes) {
      iconBytesById.set(iconIdEntry.id, bytes);
    }
  }

  let bestEntryOffset = 6;
  let bestScore = -1;
  for (let i = 0; i < count; i++) {
    const offset = 6 + i * 14;
    const width = groupView.getUint8(offset) || 256;
    const height = groupView.getUint8(offset + 1) || 256;
    const bytesInRes = groupView.getUint32(offset + 8, true);
    const score = width * height * 1024 + bytesInRes;
    if (score > bestScore) {
      bestScore = score;
      bestEntryOffset = offset;
    }
  }

  const iconId = groupView.getUint16(bestEntryOffset + 12, true);
  const imageBytes = iconBytesById.get(iconId);
  if (!imageBytes) {
    return null;
  }

  const ico = new Uint8Array(6 + 16 + imageBytes.length);
  const icoView = new DataView(ico.buffer);
  icoView.setUint16(0, 0, true);
  icoView.setUint16(2, 1, true);
  icoView.setUint16(4, 1, true);
  ico.set(groupBytes.slice(bestEntryOffset, bestEntryOffset + 8), 6);
  icoView.setUint16(14, groupView.getUint16(bestEntryOffset + 4, true), true);
  icoView.setUint16(16, groupView.getUint16(bestEntryOffset + 6, true), true);
  icoView.setUint32(18, imageBytes.length, true);
  icoView.setUint32(22, 22, true);
  ico.set(imageBytes, 22);

  return new Blob([ico], { type: "image/x-icon" });
};

export const getDemoIconUrlForPath = async (filePath: string) => {
  const siblingIco = findSiblingIco(filePath);
  if (siblingIco) {
    return await getDataUrl(siblingIco.path, siblingIco.file);
  }

  const file = fileByPath.get(filePath);
  if (!file || extname(filePath) !== "exe") {
    return null;
  }

  const iconBlob = await extractIcoFromExe(file).catch((e) => {
    console.warn("[Mock Browser FS] failed to extract exe icon", e);
    return null;
  });
  return iconBlob ? await getDataUrl(`exe-icon:${filePath}`, iconBlob) : null;
};
